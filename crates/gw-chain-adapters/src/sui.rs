#![forbid(unsafe_code)]
#![deny(
    clippy::float_arithmetic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::unwrap_used,
    clippy::indexing_slicing
)]

use gw_primitives::{ChainId, SovereignError};

/// Derives a Sui 32-byte account address from a master seed and invoice ID.
/// In Sui, standard addresses are 32 bytes formatted as `0x<64 hex digits>`.
pub fn derive_sui_deposit_address(
    master_seed: &[u8; 32],
    invoice_id: i64,
) -> (String, [u8; 32]) {
    let mut hasher = blake3::Hasher::new_keyed(master_seed);
    hasher.update(b"SUI_MOVE_COIN_DEPOSIT_LOCATOR_V9");
    hasher.update(&invoice_id.to_le_bytes());
    let digest = hasher.finalize();

    let mut addr_bytes = [0u8; 32];
    addr_bytes.copy_from_slice(digest.as_bytes());

    let hex_addr = hex_encode_32(&addr_bytes);
    let full = format!("0x{hex_addr}");
    (full, addr_bytes)
}

fn hex_encode_32(bytes: &[u8; 32]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(64);
    for &b in bytes {
        let hi = (b >> 4) as usize;
        let lo = (b & 0x0f) as usize;
        if let Some(&c) = HEX.get(hi) {
            s.push(c as char);
        }
        if let Some(&c) = HEX.get(lo) {
            s.push(c as char);
        }
    }
    s
}

/// Validates that a string is a valid Sui address (`0x` + 64 hex characters).
pub fn validate_sui_address(address: &str) -> Result<[u8; 32], SovereignError> {
    let clean = address.strip_prefix("0x").unwrap_or(address);
    if clean.len() != 64 {
        return Err(SovereignError::InvalidBlockchainAddress {
            chain: ChainId::SuiMainnet,
            address: format!("Sui address must be 64 hex chars (32 bytes), got {}", clean.len()),
        });
    }

    let mut out = [0u8; 32];
    for (i, byte_slot) in out.iter_mut().enumerate() {
        let start = i.saturating_mul(2);
        let end = start.saturating_add(2);
        let chunk = match clean.get(start..end) {
            Some(s) => s,
            None => {
                return Err(SovereignError::InvalidBlockchainAddress {
                    chain: ChainId::SuiMainnet,
                    address: "Invalid hex index in Sui address".to_string(),
                });
            }
        };
        *byte_slot = u8::from_str_radix(chunk, 16).map_err(|e| {
            SovereignError::InvalidBlockchainAddress {
                chain: ChainId::SuiMainnet,
                address: format!("Invalid hex byte '{chunk}' in Sui address: {e}"),
            }
        })?;
    }

    Ok(out)
}
