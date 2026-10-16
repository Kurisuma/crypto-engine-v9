#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Security execution zones enforcing Axiom 1 and Axiom 3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SecurityZone {
    /// Zone A: Public DMZ, Quote Engine, IMPRCE, oberer-orm HybridConn (Zero Private Keys).
    ZoneAPublicWatchOnly,
    /// Zone B: Isolated Native Blockchain Node & Indexing Plane (Zero Private Keys).
    ZoneBPrivateNodePlane,
    /// Zone C: Air-Gapped / Hardware-Attested Signing Enclave (Stateless Policy Guard + Zeroized Keys).
    ZoneCZeroTrustEnclave,
}

/// Comprehensive cryptographic and cell integrity audit report for any Sovereign engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EngineIntegrityReport {
    pub engine_id: String,
    pub security_zone: SecurityZone,
    pub verified_records_or_cells: u64,
    pub detected_corruptions_or_tampers: u64,
    pub autonomous_self_heals: u64,
    pub hash_chain_head_hex: String,
    pub audit_passed: bool,
}

/// Real-time health, latency, and worker snapshot for any Sovereign engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EngineHealthSnapshot {
    pub engine_id: String,
    pub security_zone: SecurityZone,
    pub is_healthy: bool,
    pub active_workers_or_shards: usize,
    pub total_operations_processed: u64,
    pub p99_operation_latency_us: u64,
    pub memory_allocated_bytes: usize,
}
