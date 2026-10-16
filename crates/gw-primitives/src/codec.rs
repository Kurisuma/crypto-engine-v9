#![forbid(unsafe_code)]

use crate::error::SovereignError;
use alloy_primitives::U256;

/// Bijective Order-Preserving 256-Bit Integer Codec for `oberer-orm` `Vec<u8>` (`BLOB`) and `String` (`TEXT`).
/// Guarantees SQLite `memcmp(a, b)` in B-Trees is isomorphic to `a < b` in \mathbb{Z}_{2^{256}}.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObererU256Codec;

impl ObererU256Codec {
    /// Encodes U256 into a 32-byte Big-Endian `Vec<u8>` for `oberer-orm` `BLOB` fields.
    #[inline]
    pub fn to_be_blob_vec(value: U256) -> Vec<u8> {
        value.to_be_bytes::<32>().to_vec()
    }

    /// Decodes a 32-byte Big-Endian `oberer-orm` `BLOB` slice back into U256.
    #[inline]
    pub fn from_be_blob_slice(bytes: &[u8]) -> Result<U256, SovereignError> {
        let arr: [u8; 32] = bytes.try_into().map_err(|_| SovereignError::ObererOrmError {
            detail: format!("Invalid U256 BLOB(32) byte length: {}", bytes.len()),
        })?;
        Ok(U256::from_be_bytes(arr))
    }

    /// Encodes U256 into a fixed 78-character zero-padded base-10 `String` for `oberer-orm` `TEXT` fields.
    #[inline]
    pub fn to_padded_dec78(value: U256) -> String {
        format!("{value:0>78}")
    }

    /// Parses a 78-character zero-padded decimal string back into U256.
    #[inline]
    pub fn from_padded_dec78(s: &str) -> Result<U256, SovereignError> {
        let trimmed = s.trim_start_matches('0');
        if trimmed.is_empty() {
            return Ok(U256::ZERO);
        }
        U256::from_str_radix(trimmed, 10).map_err(|e| SovereignError::ObererOrmError {
            detail: format!("Invalid U256 TEXT(78) decimal string '{s}': {e}"),
        })
    }
}
