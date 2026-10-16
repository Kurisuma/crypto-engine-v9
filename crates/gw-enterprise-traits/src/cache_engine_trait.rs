#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic, clippy::unwrap_used)]

use crate::oberer_storage_engine::{ObererOrmStorageEngine, SovereignEngine};
use async_trait::async_trait;
use gw_primitives::{CachePriorityTier, ChainId, SovereignError};
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashSet;

/// Tamper-evident raw in-memory cell returned by a quorum-verified read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedRawCacheCell {
    pub namespace: String,
    pub key: String,
    pub payload_bytes: Vec<u8>,
    pub priority_tier: CachePriorityTier,
    pub version: u64,
    pub created_offset_ms: u64,
    pub expires_offset_ms: Option<u64>,
    pub crc32c_tag: u32,
    pub mac_tag_128: [u8; 16],
    pub surviving_replicas: u8,
}

/// Typed wrapper decoded from a `VerifiedRawCacheCell`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedCacheCell<T> {
    pub namespace: String,
    pub key: String,
    pub value: T,
    pub priority_tier: CachePriorityTier,
    pub version: u64,
    pub created_offset_ms: u64,
    pub expires_offset_ms: Option<u64>,
    pub crc32c_tag: u32,
    pub mac_tag_128: [u8; 16],
    pub surviving_replicas: u8,
}

/// Engine Trait I: Object-Safe Sovereign In-Memory Priority-Redundant Cache Engine (`IMPRCE`).
#[async_trait]
pub trait PriorityRedundantCacheEngine: SovereignEngine {
    /// Hydrates P0, P1, and P2 in under 10ms from storage.
    async fn hydrate_from_oberer_hybrid(
        &self,
        storage_engine: &dyn ObererOrmStorageEngine,
    ) -> Result<usize, SovereignError>;

    /// Writes raw bytes across k(priority_tier) disjoint mirror arenas.
    async fn put_redundant_raw(
        &self,
        namespace: &'static str,
        key: &str,
        payload_bytes: &[u8],
        priority_tier: CachePriorityTier,
        ttl_ms: Option<u64>,
    ) -> Result<u64, SovereignError>;

    /// Performs an O(1) read from Primary Arena 0 with instant failover and repair write.
    async fn get_quorum_verified_raw(
        &self,
        namespace: &'static str,
        key: &str,
    ) -> Result<Option<VerifiedRawCacheCell>, SovereignError>;

    /// Atomic Compare-And-Swap (CAS) across all k mirror arenas.
    async fn compare_and_swap_redundant_raw(
        &self,
        namespace: &'static str,
        key: &str,
        expected_version: u64,
        new_payload_bytes: &[u8],
        priority_tier: CachePriorityTier,
        new_ttl_ms: Option<u64>,
    ) -> Result<bool, SovereignError>;

    /// Atomically registers an active checkout lookup key into the P1 watchlist.
    async fn register_active_watchlist_key(
        &self,
        chain_id: ChainId,
        lookup_key: &str,
        invoice_id: i64,
        ttl_ms: u64,
    ) -> Result<(), SovereignError>;

    /// Wait-free O(1) batch filter for block indexers via L1 Bloom Filter.
    async fn filter_watched_keys_batch(
        &self,
        chain_id: ChainId,
        candidate_keys: &[String],
    ) -> Result<HashSet<String>, SovereignError>;

    /// Atomically removes a settled or expired key and zeroizes its memory.
    async fn remove_and_zeroize(
        &self,
        namespace: &'static str,
        key: &str,
    ) -> Result<bool, SovereignError>;

    /// Enforces hard per-tier byte quota ceilings by evicting entries.
    async fn enforce_quota_fence_eviction(&self) -> Result<usize, SovereignError>;
}

/// Ergonomic typed extension trait for `PriorityRedundantCacheEngine`.
#[async_trait]
pub trait PriorityRedundantCacheEngineExt: PriorityRedundantCacheEngine {
    async fn put_redundant<T>(
        &self,
        namespace: &'static str,
        key: &str,
        value: &T,
        priority_tier: CachePriorityTier,
        ttl_ms: Option<u64>,
    ) -> Result<u64, SovereignError>
    where
        T: Serialize + Send + Sync,
    {
        let bytes = serde_json::to_vec(value).map_err(|e| SovereignError::ObererOrmError {
            detail: format!("Cache serialization error: {e}"),
        })?;
        self.put_redundant_raw(namespace, key, &bytes, priority_tier, ttl_ms)
            .await
    }

    async fn get_quorum_verified<T>(
        &self,
        namespace: &'static str,
        key: &str,
    ) -> Result<Option<VerifiedCacheCell<T>>, SovereignError>
    where
        T: DeserializeOwned + Send + Sync,
    {
        let Some(raw) = self.get_quorum_verified_raw(namespace, key).await? else {
            return Ok(None);
        };
        let value: T =
            serde_json::from_slice(&raw.payload_bytes).map_err(|e| SovereignError::ObererOrmError {
                detail: format!("Cache deserialization error: {e}"),
            })?;
        Ok(Some(VerifiedCacheCell {
            namespace: raw.namespace,
            key: raw.key,
            value,
            priority_tier: raw.priority_tier,
            version: raw.version,
            created_offset_ms: raw.created_offset_ms,
            expires_offset_ms: raw.expires_offset_ms,
            crc32c_tag: raw.crc32c_tag,
            mac_tag_128: raw.mac_tag_128,
            surviving_replicas: raw.surviving_replicas,
        }))
    }
}

impl<E: ?Sized + PriorityRedundantCacheEngine> PriorityRedundantCacheEngineExt for E {}
