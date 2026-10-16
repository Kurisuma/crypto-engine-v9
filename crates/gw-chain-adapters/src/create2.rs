#![forbid(unsafe_code)]
#![deny(
    clippy::float_arithmetic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::unwrap_used,
    clippy::indexing_slicing
)]

use gw_primitives::{ChainId, RawAssetAmount, SovereignError};
use sha3::{Digest, Keccak256};

/// Computes raw 20-byte EVM CREATE2 counterfactual address.
///
/// $$A_{\text{raw}}(s) = \text{Keccak256}\left(\text{0xff} \parallel A_{\text{factory}} \parallel s \parallel H_{\text{init}}\right)[12..32]$$
pub fn derive_evm_create2_raw(
    factory: [u8; 20],
    salt: [u8; 32],
    init_code_hash: [u8; 32],
) -> [u8; 20] {
    let mut hasher = Keccak256::new();
    hasher.update([0xff]);
    hasher.update(factory);
    hasher.update(salt);
    hasher.update(init_code_hash);
    let digest = hasher.finalize();

    let mut addr = [0u8; 20];
    if let Some(slice) = digest.get(12..32) {
        addr.copy_from_slice(slice);
    }
    addr
}

/// Converts a 20-byte address to an EIP-55 mixed-case checksum string ("0x...").
pub fn to_eip55_checksum(addr: &[u8; 20]) -> String {
    let hex_digits = b"0123456789abcdef";
    let mut hex_ascii = [0u8; 40];
    for (i, byte) in addr.iter().enumerate() {
        let hi = (byte >> 4) as usize;
        let lo = (byte & 0x0f) as usize;
        if let Some(c) = hex_digits.get(hi) {
            if let Some(target) = hex_ascii.get_mut(i * 2) {
                *target = *c;
            }
        }
        if let Some(c) = hex_digits.get(lo) {
            if let Some(target) = hex_ascii.get_mut(i * 2 + 1) {
                *target = *c;
            }
        }
    }

    let hash = Keccak256::digest(hex_ascii);

    let mut result = String::with_capacity(42);
    result.push_str("0x");

    for (i, &c) in hex_ascii.iter().enumerate() {
        if c.is_ascii_digit() {
            result.push(c as char);
        } else {
            let hash_byte = hash.get(i / 2).copied().unwrap_or(0);
            let nibble = if i % 2 == 0 { hash_byte >> 4 } else { hash_byte & 0x0f };
            if nibble >= 8 {
                result.push((c as char).to_ascii_uppercase());
            } else {
                result.push(c as char);
            }
        }
    }

    result
}

/// Derives the full EIP-55 checksummed EVM counterfactual deposit address.
pub fn derive_evm_create2_address(
    factory: [u8; 20],
    salt: [u8; 32],
    init_code_hash: [u8; 32],
) -> String {
    let raw = derive_evm_create2_raw(factory, salt, init_code_hash);
    to_eip55_checksum(&raw)
}

/// Computes raw 20-byte Tron (TVM) CREATE2 counterfactual address.
///
/// $$A_{\text{tvm\_raw}}(s) = \text{Keccak256}\left(\text{0x41} \parallel A_{\text{factory}} \parallel s \parallel H_{\text{init}}\right)[12..32]$$
pub fn derive_tron_create2_raw(
    factory: [u8; 20],
    salt: [u8; 32],
    init_code_hash: [u8; 32],
) -> [u8; 20] {
    let mut hasher = Keccak256::new();
    hasher.update([0x41]);
    hasher.update(factory);
    hasher.update(salt);
    hasher.update(init_code_hash);
    let digest = hasher.finalize();

    let mut addr = [0u8; 20];
    if let Some(slice) = digest.get(12..32) {
        addr.copy_from_slice(slice);
    }
    addr
}

/// Derives the Base58Check Tron address (starts with 'T', 21 bytes prefixed with 0x41).
pub fn derive_tron_create2_address(
    factory: [u8; 20],
    salt: [u8; 32],
    init_code_hash: [u8; 32],
) -> String {
    let raw = derive_tron_create2_raw(factory, salt, init_code_hash);
    let mut payload = [0u8; 21];
    if let Some(first) = payload.first_mut() {
        *first = 0x41; // Tron mainnet prefix
    }
    if let Some(dest) = payload.get_mut(1..21) {
        dest.copy_from_slice(&raw);
    }
    bs58::encode(payload).with_check().into_string()
}

/// Calculates the Tron 1-Sun Dust Keeper sweep amount:
///
/// $$V_{\text{sweep}} = B_{\text{current}} \ominus 1$$
///
/// Leaves 1 Sun in the temporary sweeper contract, preventing TRC-20 storage slot deallocation
/// and saving ~65,000 Energy on all future deposits!
pub fn calculate_tron_1sun_dust_keeper_sweep(
    current_balance: RawAssetAmount,
) -> Result<RawAssetAmount, SovereignError> {
    if current_balance.raw() == alloy_primitives::U256::ZERO {
        return Ok(current_balance);
    }
    let one_unit = alloy_primitives::U256::from(1);
    let sweep_units = current_balance.raw().saturating_sub(one_unit);
    Ok(RawAssetAmount::new(sweep_units, current_balance.decimals()))
}

/// Validates an EVM hex address and asserts EIP-55 checksum correctness if mixed-case.
pub fn validate_evm_address(address: &str) -> Result<[u8; 20], SovereignError> {
    let clean = address.strip_prefix("0x").unwrap_or(address);
    if clean.len() != 40 {
        return Err(SovereignError::InvalidBlockchainAddress {
            chain: ChainId::EthereumMainnet,
            address: format!("EVM address hex length must be 40 chars, got {}", clean.len()),
        });
    }

    let mut raw = [0u8; 20];
    for (i, byte_slot) in raw.iter_mut().enumerate() {
        let start = i.saturating_mul(2);
        let end = start.saturating_add(2);
        let chunk = match clean.get(start..end) {
            Some(s) => s,
            None => {
                return Err(SovereignError::InvalidBlockchainAddress {
                    chain: ChainId::EthereumMainnet,
                    address: "Invalid character index in EVM address".to_string(),
                });
            }
        };
        *byte_slot = u8::from_str_radix(chunk, 16).map_err(|e| {
            SovereignError::InvalidBlockchainAddress {
                chain: ChainId::EthereumMainnet,
                address: format!("Invalid hex byte '{chunk}': {e}"),
            }
        })?;
    }

    // Check if mixed-case: if so, must match EIP-55
    let has_upper = clean.chars().any(|c| c.is_ascii_uppercase());
    let has_lower = clean.chars().any(|c| c.is_ascii_lowercase());
    if has_upper && has_lower {
        let expected = to_eip55_checksum(&raw);
        if address != expected {
            return Err(SovereignError::InvalidBlockchainAddress {
                chain: ChainId::EthereumMainnet,
                address: format!("EIP-55 checksum mismatch: expected {expected}, got {address}"),
            });
        }
    }

    Ok(raw)
}

/// Validates a Tron Base58Check address: starts with 'T', 21-byte payload starting with 0x41.
pub fn validate_tron_address(address: &str) -> Result<[u8; 21], SovereignError> {
    let decoded = bs58::decode(address).with_check(None).into_vec().map_err(|e| {
        SovereignError::InvalidBlockchainAddress {
            chain: ChainId::TronMainnet,
            address: format!("Tron Base58Check decode error: {e}"),
        }
    })?;

    if decoded.len() != 21 {
        return Err(SovereignError::InvalidBlockchainAddress {
            chain: ChainId::TronMainnet,
            address: format!("Tron address must be 21 bytes decoded, got {}", decoded.len()),
        });
    }

    if decoded.first() != Some(&0x41) {
        return Err(SovereignError::InvalidBlockchainAddress {
            chain: ChainId::TronMainnet,
            address: "Tron address must start with prefix byte 0x41 ('T')".to_string(),
        });
    }

    let mut out = [0u8; 21];
    out.copy_from_slice(&decoded);
    Ok(out)
}
