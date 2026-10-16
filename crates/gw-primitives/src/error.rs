#![forbid(unsafe_code)]

use crate::asset::{CachePriorityTier, ChainId};
use crate::zone::SecurityZone;
use oberer_orm::error::OrmError;
use oberer_orm::result::OrmFailure;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Exhaustive algebraic, `oberer-orm`, cache, consensus, DI, and enclave security error taxonomy.
#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize, Deserialize)]
pub enum SovereignError {
    #[error("Checked 256-bit or 64-bit integer overflow in context: {context}")]
    Overflow { context: &'static str },
    #[error("Checked 256-bit or 64-bit integer underflow in context: {context}")]
    Underflow { context: &'static str },
    #[error("Division or modulo by zero in fixed-point calculation")]
    DivisionByZero,
    #[error("Physical decimal mismatch: expected={expected}, actual={actual}")]
    DecimalMismatch { expected: u8, actual: u8 },
    #[error("oberer-orm execution error: {detail}")]
    ObererOrmError { detail: String },
    #[error("oberer-orm CAS optimistic lock conflict (`#[orm(version)]`) on table={table}, pk={pk}")]
    ObererOptimisticLockConflict { table: &'static str, pk: String },
    #[error("oberer-orm safety violation: unfiltered UPDATE/DELETE blocked (`NoFilterOnMutation`)")]
    ObererUnfilteredMutationBlocked,
    #[error("CRITICAL SECURITY ALERT: SQLite row MAC or hash-chain tampering detected in table={table}, pk={primary_key}")]
    SqliteRowTamperingDetected {
        table: &'static str,
        primary_key: String,
    },
    #[error("IMPRCE cache integrity violation: CRC32C/MAC mismatch on key={key}, self_healed={self_healed}")]
    CacheIntegrityCorrupted { key: String, self_healed: bool },
    #[error("IMPRCE priority quota ceiling exhausted for non-evictable tier {tier:?}")]
    CacheQuotaExhausted { tier: CachePriorityTier },
    #[error("Replay attack blocked by oberer-orm IdempotencyService / IMPRCE: digest '{digest}'")]
    ReplayAttackBlocked { digest: String },
    #[error("Dual-oracle divergence {divergence_bps} bps exceeds circuit breaker {max_bps} bps")]
    OracleDivergenceCircuitBreaker { divergence_bps: u32, max_bps: u32 },
    #[error("Blockchain reorganization detected at height {height}: rolled back to LCA {lca_height}")]
    ConsensusReorgDetected { height: u64, lca_height: u64 },
    #[error("Blockchain reorganization depth {depth} exceeds sliding window depth {max_depth}")]
    ConsensusReorgExceedsSlidingWindow { depth: u64, max_depth: u64 },
    #[error("5-Gate verification failed at Gate {gate}: {reason}")]
    GateVerificationFailed { gate: u8, reason: String },
    #[error("Zone C Enclave policy violation: {reason}")]
    EnclavePolicyViolation { reason: String },
    #[error("Insufficient M-of-N hardware quorum: required={required}, verified={verified}")]
    QuorumInsufficient { required: usize, verified: usize },
    #[error("Native node transport failure on {chain:?}: {detail}")]
    NodeTransportFailure { chain: ChainId, detail: String },
    #[error("Rotating deposit address pool exhausted on {chain:?}")]
    AddressPoolExhausted { chain: ChainId },
    #[error("Invalid blockchain address or syntax error on {chain:?}: {address}")]
    InvalidBlockchainAddress { chain: ChainId, address: String },
    #[error("Dependency Injection (DI) configuration error: missing or invalid dependency '{dependency}'")]
    DependencyInjectionError { dependency: &'static str },
    #[error("Security Zone isolation violation: engine '{engine_id}' in {actual:?} cannot be injected into {expected:?}")]
    SecurityZoneMismatch {
        engine_id: &'static str,
        expected: SecurityZone,
        actual: SecurityZone,
    },
}

impl From<OrmError> for SovereignError {
    fn from(err: OrmError) -> Self {
        match err {
            OrmError::OptimisticLockConflict { table, pk, .. } => Self::ObererOptimisticLockConflict {
                table: Box::leak(table.into_boxed_str()),
                pk: pk.to_string(),
            },
            OrmError::NoFilterOnMutation => Self::ObererUnfilteredMutationBlocked,
            other => Self::ObererOrmError {
                detail: other.to_string(),
            },
        }
    }
}

impl From<OrmFailure> for SovereignError {
    fn from(failure: OrmFailure) -> Self {
        Self::ObererOrmError {
            detail: format!("{failure:?}"),
        }
    }
}
