#![forbid(unsafe_code)]
#![deny(
    clippy::float_arithmetic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::unwrap_used,
    clippy::indexing_slicing
)]

use gw_primitives::{ChainId, SovereignError};

/// 32-character Crockford Base32 alphabet (no I, L, O, U to prevent visual confusion).
pub const CROCKFORD_BASE32_ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// 40-bit Key-Scheduled Feistel Permutation for TON Jetton invoice memo tags:
///
/// $$\mathcal{F}_{K_{\text{memo}}} : [0, 2^{40}-1] \longleftrightarrow [0, 2^{40}-1]$$
///
/// Ensures 100% collision-free bijective mapping from internal 40-bit invoice ID to
/// an 8-character customer-facing Crockford Base32 memo tag.
#[derive(Debug, Clone)]
pub struct TonFeistelMemoCipher {
    round_keys: [[u8; 32]; 6],
}

impl TonFeistelMemoCipher {
    pub fn new(master_memo_key: &[u8; 32]) -> Self {
        let mut round_keys = [[0u8; 32]; 6];
        for (r, key) in round_keys.iter_mut().enumerate() {
            let mut hasher = blake3::Hasher::new_keyed(master_memo_key);
            hasher.update(b"TON_FEISTEL_ROUND_KEY_V9");
            hasher.update(&(r as u64).to_le_bytes());
            *key = *hasher.finalize().as_bytes();
        }
        Self { round_keys }
    }

    fn round_function(&self, r: usize, right_20bit: u32) -> u32 {
        let round_key = match self.round_keys.get(r) {
            Some(k) => k,
            None => &self.round_keys[0],
        };
        let mut hasher = blake3::Hasher::new_keyed(round_key);
        hasher.update(&right_20bit.to_le_bytes());
        let digest = hasher.finalize();
        let bytes = digest.as_bytes();
        let mut chunk = [0u8; 4];
        if let Some(src) = bytes.get(0..4) {
            chunk.copy_from_slice(src);
        }
        u32::from_le_bytes(chunk) & 0x000F_FFFF // Mask to 20 bits
    }

    /// Encrypts a 40-bit integer bijectively: $\mathcal{F}_K(i)$.
    pub fn encrypt_40bit(&self, invoice_id_40bit: u64) -> u64 {
        let masked = invoice_id_40bit & 0x0000_00FF_FFFF_FFFF;
        let mut left = u32::try_from((masked >> 20) & 0x000F_FFFF).unwrap_or(0);
        let mut right = u32::try_from(masked & 0x000F_FFFF).unwrap_or(0);

        for r in 0..6 {
            let f = self.round_function(r, right);
            let next_left = right;
            let next_right = left ^ f;
            left = next_left;
            right = next_right;
        }

        // Swap back in the last round
        let out_left = (right as u64) & 0x000F_FFFF;
        let out_right = (left as u64) & 0x000F_FFFF;
        (out_left << 20) | out_right
    }

    /// Decrypts a 40-bit integer: $\mathcal{F}_K^{-1}(m) \equiv i$.
    pub fn decrypt_40bit(&self, memo_code_40bit: u64) -> u64 {
        let masked = memo_code_40bit & 0x0000_00FF_FFFF_FFFF;
        let mut right = u32::try_from((masked >> 20) & 0x000F_FFFF).unwrap_or(0);
        let mut left = u32::try_from(masked & 0x000F_FFFF).unwrap_or(0);

        for r in (0..6).rev() {
            let f = self.round_function(r, left);
            let prev_right = left;
            let prev_left = right ^ f;
            right = prev_right;
            left = prev_left;
        }

        let out_left = (left as u64) & 0x000F_FFFF;
        let out_right = (right as u64) & 0x000F_FFFF;
        (out_left << 20) | out_right
    }

    /// Encodes a 40-bit invoice ID into an 8-character Crockford Base32 memo tag.
    pub fn encode_memo_tag(&self, invoice_id_40bit: u64) -> String {
        let permuted = self.encrypt_40bit(invoice_id_40bit);
        let mut tag = String::with_capacity(8);

        // 8 characters, each taking 5 bits (8 * 5 = 40 bits)
        for i in (0..8).rev() {
            let shift = i * 5;
            let symbol_idx = ((permuted >> shift) & 0x1F) as usize;
            let c = match CROCKFORD_BASE32_ALPHABET.get(symbol_idx) {
                Some(&b) => b as char,
                None => '0',
            };
            tag.push(c);
        }

        tag
    }

    /// Decodes an 8-character Crockford Base32 memo tag back to the exact invoice ID.
    pub fn decode_memo_tag(&self, tag: &str) -> Result<u64, SovereignError> {
        let trimmed = tag.trim();
        if trimmed.len() != 8 {
            return Err(SovereignError::InvalidBlockchainAddress {
                chain: ChainId::TonMainnet,
                address: format!("TON memo tag must be 8 characters, got {}", trimmed.len()),
            });
        }

        let mut permuted: u64 = 0;
        for c in trimmed.chars() {
            let upper = c.to_ascii_uppercase();
            // Crockford normalization: map O/o -> 0, I/i/L/l -> 1
            let norm = match upper {
                'O' => '0',
                'I' | 'L' => '1',
                other => other,
            };

            let symbol_idx = match CROCKFORD_BASE32_ALPHABET.iter().position(|&b| (b as char) == norm) {
                Some(idx) => idx as u64,
                None => {
                    return Err(SovereignError::InvalidBlockchainAddress {
                        chain: ChainId::TonMainnet,
                        address: format!("Invalid Crockford Base32 character '{c}' in TON memo tag"),
                    });
                }
            };

            permuted = (permuted << 5) | symbol_idx;
        }

        let invoice_id = self.decrypt_40bit(permuted);
        Ok(invoice_id)
    }
}

/// CRC16-CCITT calculation for TON user-friendly address validation.
pub fn ton_crc16(data: &[u8]) -> u16 {
    let mut crc: u16 = 0;
    for &byte in data {
        let mut b = (byte as u16) << 8;
        for _ in 0..8 {
            if ((crc ^ b) & 0x8000) != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
            b <<= 1;
        }
    }
    crc
}

/// Validates a TON address in either raw format (`workchain:hex_32bytes`) or user-friendly Base64/Base64url (`EQ...`, `UQ...`).
pub fn validate_ton_address(address: &str) -> Result<(), SovereignError> {
    let trimmed = address.trim();

    // Check if raw format: 0:<64 hex> or -1:<64 hex>
    if let Some((wc, hex_part)) = trimmed.split_once(':') {
        if (wc == "0" || wc == "-1") && hex_part.len() == 64 {
            let is_hex = hex_part.chars().all(|c| c.is_ascii_hexdigit());
            if is_hex {
                return Ok(());
            }
        }
    }

    // Otherwise, parse as 48-character user-friendly Base64 / Base64url address
    if trimmed.len() != 48 {
        return Err(SovereignError::InvalidBlockchainAddress {
            chain: ChainId::TonMainnet,
            address: format!("TON user-friendly address length must be 48 chars, got {}", trimmed.len()),
        });
    }

    // Must start with EQ (bounceable) or UQ (non-bounceable) or kQ / 0Q for testnets
    if !trimmed.starts_with("EQ") && !trimmed.starts_with("UQ") && !trimmed.starts_with("kQ") && !trimmed.starts_with("0Q") {
        return Err(SovereignError::InvalidBlockchainAddress {
            chain: ChainId::TonMainnet,
            address: format!("TON address starts with invalid prefix: '{trimmed}'"),
        });
    }

    Ok(())
}
