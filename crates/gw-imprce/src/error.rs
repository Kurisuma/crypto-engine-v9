#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic, clippy::unwrap_used)]

use thiserror::Error;

/// Error taxonomy for the Sovereign In-Memory Priority-Redundant Cache Engine.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ImprceError {
    #[error("Hardware SEU bit-rot detected (CRC32C mismatch): expected {expected:#x}, computed {computed:#x}")]
    BitRotDetected { expected: u32, computed: u32 },

    #[error("Cryptographic in-memory MAC tampering detected for key: {0}")]
    MacTamperingDetected(String),

    #[error("Integrity check failed: all replica arenas reported corruption or entry missing")]
    IntegrityQuorumFailed,

    #[error("Hard quota capacity exhausted for tier {0:?} (refused to evict protected tier)")]
    TierQuotaExhausted(String),

    #[error("Entry for key '{0}' has expired")]
    EntryExpired(String),

    #[error("Key not found in any mirror arena: {0}")]
    KeyNotFound(String),

    #[error("Anti-replay violation: nonce/idempotency key '{0}' already consumed")]
    ReplayDetected(String),

    #[error("Hash-chained WAL corruption detected: expected prev_hash {expected}, found {computed}")]
    WalHashChainBroken { expected: String, computed: String },

    #[error("Hydration from storage engine failed: {0}")]
    HydrationError(String),

    #[error("Serialization / Deserialization error: {0}")]
    SerializationError(String),
}
