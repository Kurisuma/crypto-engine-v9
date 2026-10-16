#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic, clippy::unwrap_used)]

use async_trait::async_trait;
pub use gw_primitives::{EngineHealthSnapshot, EngineIntegrityReport, SecurityZone, SovereignError};
use oberer_orm::conn::{HybridConn, OrmConn};
use oberer_orm::prelude::*;
use subtle::ConstantTimeEq;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Root Object-Oriented Lifecycle & Security Trait implemented by every subsystem engine.
#[async_trait]
pub trait SovereignEngine: Send + Sync {
    /// Unique static identifier of the engine (e.g., `"OBERER_ORM_HYBRID_STORAGE_ENGINE"`).
    fn engine_id(&self) -> &'static str;

    /// Physical/logical security zone in which this engine is permitted to execute.
    fn security_zone(&self) -> SecurityZone;

    /// Initializes internal state, runs migrations, and warms caches.
    async fn initialize_engine(&self) -> Result<(), SovereignError>;

    /// Executes a deep cryptographic integrity audit (verifying CRC32C, Row MACs, or Hash Chains).
    async fn audit_cryptographic_integrity(&self) -> Result<EngineIntegrityReport, SovereignError>;

    /// Returns a standardized operational telemetry and health snapshot.
    async fn health_snapshot(&self) -> Result<EngineHealthSnapshot, SovereignError>;

    /// Flushes caches, syncs storage, and zeroes ephemeral keys.
    async fn shutdown_gracefully(&self) -> Result<(), SovereignError>;
}

/// Per-boot / HSM-unsealed 256-bit Row-Authentication Key for `oberer-orm` entities.
/// Protects SQLite `.db` and `.db-wal` files against offline tampering (Axiom 8).
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct ObererRowMacSigner {
    row_mac_key: [u8; 32],
}

impl ObererRowMacSigner {
    pub const fn new(row_mac_key: [u8; 32]) -> Self {
        Self { row_mac_key }
    }

    /// Computes a 32-byte keyed Blake3 MAC binding `(table || pk || version || canonical_bytes)`.
    pub fn compute_row_mac(
        &self,
        table_name: &'static str,
        primary_key: &str,
        version: i64,
        canonical_payload: &[u8],
    ) -> Vec<u8> {
        let mut hasher = blake3::Hasher::new_keyed(&self.row_mac_key);
        hasher.update(b"OBERER_ORM_V9_ROW_MAC");
        hasher.update(table_name.as_bytes());
        hasher.update(&(primary_key.len() as u64).to_le_bytes());
        hasher.update(primary_key.as_bytes());
        hasher.update(&version.to_le_bytes());
        hasher.update(&(canonical_payload.len() as u64).to_le_bytes());
        hasher.update(canonical_payload);
        hasher.finalize().as_bytes().to_vec()
    }

    /// Verifies the stored `row_mac: Vec<u8>` column in constant time (`subtle::ConstantTimeEq`).
    pub fn verify_row_mac(
        &self,
        table_name: &'static str,
        primary_key: &str,
        version: i64,
        canonical_payload: &[u8],
        stored_mac: &[u8],
    ) -> Result<(), SovereignError> {
        let expected =
            self.compute_row_mac(table_name, primary_key, version, canonical_payload);
        if expected.len() == stored_mac.len() && bool::from(expected.ct_eq(stored_mac)) {
            Ok(())
        } else {
            Err(SovereignError::SqliteRowTamperingDetected {
                table: table_name,
                primary_key: primary_key.to_string(),
            })
        }
    }
}

/// Trait implemented by security-critical `#[derive(OrmTable)]` models to bind `#[orm(version)]`
/// and `row_mac: Vec<u8>` verification into `ObererOrmStorageEngine`.
pub trait TamperEvidentOrmModel: OrmTable + Clone + Send + Sync + 'static {
    const TABLE_LABEL: &'static str;
    fn pk_string(&self) -> String;
    fn current_version(&self) -> i64;
    fn canonical_security_bytes(&self) -> Vec<u8>;
    fn row_mac_slice(&self) -> &[u8];
    fn assign_row_mac(&mut self, mac: Vec<u8>);
}

/// Master `oberer-orm` Dual-Tier Storage Engine Trait (`HybridConn` + V001/V002 Platform Services).
#[async_trait]
pub trait ObererOrmStorageEngine: SovereignEngine {
    /// Returns a reference to the dual-tier `HybridConn` (`:memory:` working DB + disk WAL DB).
    fn hybrid_conn(&self) -> &HybridConn;

    /// Returns a cloned handle to the fast `:memory:` `OrmConn` actor.
    fn memory_conn(&self) -> OrmConn {
        self.hybrid_conn().mem().clone()
    }

    /// Returns a cloned handle to the persistent disk WAL `OrmConn` actor.
    fn disk_conn(&self) -> OrmConn {
        self.hybrid_conn().file().clone()
    }

    /// Returns the cryptographic Row-MAC signer used to seal and verify `#[derive(OrmTable)]` rows.
    fn row_mac_signer(&self) -> &ObererRowMacSigner;

    /// Verifies and records an atomic idempotency barrier via `oberer_orm::runtime::IdempotencyService`.
    async fn acquire_idempotency_barrier(
        &self,
        scope: &str,
        request_key: &str,
        payload_sha256_hex: &str,
    ) -> Result<(), SovereignError>;

    /// Appends an immutable audit trail entry via `oberer_orm::runtime::AuditRecordBuilder`.
    async fn record_platform_audit_log(
        &self,
        action: &str,
        actor_id: &str,
        detail_json: &str,
    ) -> Result<(), SovereignError>;

    /// Triggers an explicit row-level delta flush (`DirtyTracker`) from `:memory:` L1 to Disk WAL L2.
    async fn sync_hybrid_to_disk(&self) -> Result<(), SovereignError>;
}

/// Extension trait providing generic closure-based ACID transactions on any `ObererOrmStorageEngine`.
pub trait ObererOrmStorageEngineExt: ObererOrmStorageEngine {
    fn execute_atomic_synced_tx<F, R>(&self, f: F) -> Result<R, SovereignError>
    where
        F: FnOnce(&OrmConn) -> Result<R, OrmError>,
    {
        let res = self.hybrid_conn().mem().transaction(f)?;
        self.hybrid_conn().sync()?;
        Ok(res)
    }
}

impl<T: ?Sized + ObererOrmStorageEngine> ObererOrmStorageEngineExt for T {}
