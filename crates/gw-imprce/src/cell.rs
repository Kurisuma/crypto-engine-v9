#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic, clippy::unwrap_used)]

use crate::crypto::ImprceCryptoEngine;
use crate::error::ImprceError;
use crate::priority::{PriorityTier, compute_retention_score};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Zeroize-on-drop wrapper around raw payload bytes.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct ZeroizedPayload {
    pub bytes: Vec<u8>,
}

impl ZeroizedPayload {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }
}

/// Secure, tamper-evident cache cell with dual-tag integrity (CRC32C + MAC).
#[derive(Clone)]
pub struct SecureCacheCell {
    pub ns: String,
    pub key: String,
    pub priority: PriorityTier,
    pub version: u64,
    pub created_at_ms: u64,
    pub expires_at_ms: u64,
    pub hit_count: u64,
    pub crc32: u32,
    pub mac_tag: [u8; 16],
    pub payload: ZeroizedPayload,
}

impl SecureCacheCell {
    /// Construct a new sealed cell with computed CRC32 and MAC.
    #[allow(clippy::too_many_arguments)]
    pub fn seal(
        crypto: &ImprceCryptoEngine,
        ns: impl Into<String>,
        key: impl Into<String>,
        priority: PriorityTier,
        version: u64,
        created_at_ms: u64,
        expires_at_ms: u64,
        payload_bytes: Vec<u8>,
    ) -> Self {
        let ns = ns.into();
        let key = key.into();
        let crc32 = ImprceCryptoEngine::compute_crc32(&ns, &key, version, &payload_bytes);
        let mac_tag = crypto.compute_mac(
            &ns,
            &key,
            priority as u8,
            version,
            expires_at_ms,
            &payload_bytes,
        );

        Self {
            ns,
            key,
            priority,
            version,
            created_at_ms,
            expires_at_ms,
            hit_count: 0,
            crc32,
            mac_tag,
            payload: ZeroizedPayload::new(payload_bytes),
        }
    }

    /// Verifies dual-tag integrity (fast CRC32 + constant-time MAC).
    pub fn verify(&self, crypto: &ImprceCryptoEngine) -> Result<(), ImprceError> {
        crypto.verify_cell(
            &self.ns,
            &self.key,
            self.priority as u8,
            self.version,
            self.expires_at_ms,
            &self.payload.bytes,
            self.crc32,
            &self.mac_tag,
        )
    }

    /// Approximate physical byte footprint in RAM.
    pub fn byte_footprint(&self) -> usize {
        self.ns.len()
            .saturating_add(self.key.len())
            .saturating_add(self.payload.bytes.len())
            .saturating_add(64) // struct metadata overhead
    }

    /// Computes retention priority score at timestamp `now_ms`.
    pub fn retention_score(&self, now_ms: u64) -> u128 {
        compute_retention_score(
            self.priority,
            now_ms,
            self.expires_at_ms,
            self.hit_count,
            self.byte_footprint(),
        )
    }

    /// Check if entry has expired at timestamp `now_ms`.
    pub fn is_expired(&self, now_ms: u64) -> bool {
        now_ms >= self.expires_at_ms
    }
}
