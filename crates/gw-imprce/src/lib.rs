#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic, clippy::unwrap_used)]

//! Sovereign In-Memory Priority-Redundant Cache Engine (`gw-imprce`).
//! Enterprise-grade, zero-trust, self-healing, priority-partitioned in-memory state engine in pure safe Rust.

pub mod arena;
pub mod bloom;
pub mod cell;
pub mod crypto;
pub mod engine;
pub mod error;
pub mod priority;
pub mod wal;

pub use arena::{MirrorArena, NUM_SHARDS};
pub use bloom::BlockedBloomFilter;
pub use cell::{SecureCacheCell, ZeroizedPayload};
pub use crypto::ImprceCryptoEngine;
pub use engine::SovereignImprceEngine;
pub use error::ImprceError;
pub use priority::{PriorityTier, compute_retention_score};
pub use wal::{HashChainedWal, WalFrame};
