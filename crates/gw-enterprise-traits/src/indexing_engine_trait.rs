#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic, clippy::unwrap_used)]

use crate::oberer_storage_engine::SovereignEngine;
use async_trait::async_trait;
use gw_primitives::{ChainArchitecture, ChainId, RawAssetAmount, SovereignError};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Canonical block header stored in P0 IMPRCE and consensus storage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsensusBlockHeader {
    pub chain_id: ChainId,
    pub block_number: i64,
    pub block_hash: String,
    pub parent_block_hash: String,
    pub state_root_or_merkle_root: String,
    pub timestamp_unix: i64,
    pub is_finalized: bool,
}

/// Normalized incoming value transfer extracted from any blockchain's native block/slot/checkpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedTransferEvent {
    pub chain_id: ChainId,
    pub architecture: ChainArchitecture,
    pub tx_hash: String,
    pub event_index: u32,
    pub block_header: ConsensusBlockHeader,
    pub watchlist_lookup_key: String,
    pub sender_address: String,
    pub recipient_address: String,
    pub token_contract: String,
    /// Strict balance differential: post_balance.checked_sub(pre_balance) on recipient vault.
    pub net_credited_amount: RawAssetAmount,
    /// Gate 1 indicator: true iff execution succeeded without revert.
    pub execution_succeeded: bool,
    pub confirmations: u32,
}

/// Result of verifying parent-hash continuity \Lambda_{cont}(b_h, \mathcal{B}_C).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContinuityCheckOutcome {
    CanonicalAppend,
    ReorgDetected {
        orphaned_from_height: i64,
        orphaned_to_height: i64,
        lca_height: i64,
        lca_block_hash: String,
    },
}

/// Engine Trait III: Native Node Block Ingestion, DAG Continuity & O(1) Watchlist Matching Engine.
#[async_trait]
pub trait ConsensusIndexingEngine: SovereignEngine {
    fn chain_id(&self) -> ChainId;

    /// Queries the latest head height and finalized height from the local native node quorum.
    async fn query_consensus_heights(&self) -> Result<(i64, i64), SovereignError>;

    /// Fetches the canonical block header at `block_height` and verifies `parent_block_hash` continuity.
    async fn fetch_and_verify_block_continuity(
        &self,
        block_height: i64,
        previous_cached_header: Option<&ConsensusBlockHeader>,
    ) -> Result<(ConsensusBlockHeader, ContinuityCheckOutcome), SovereignError>;

    /// Walks backward to find the Lowest Common Ancestor (LCA) height h_{LCA} upon fork detection.
    async fn find_reorg_lowest_common_ancestor(
        &self,
        divergent_height: i64,
        cached_window: &[ConsensusBlockHeader],
    ) -> Result<i64, SovereignError>;

    /// Scans a single block/slot/checkpoint and extracts all transfer events matching `active_watchlist_keys`.
    async fn scan_block_for_watched_transfers(
        &self,
        block_header: &ConsensusBlockHeader,
        current_tip_height: i64,
        active_watchlist_keys: &HashSet<String>,
    ) -> Result<Vec<NormalizedTransferEvent>, SovereignError>;

    /// High-throughput range catch-up scanner.
    async fn scan_block_range_batch(
        &self,
        start_height: i64,
        end_height: i64,
        active_watchlist_keys: &HashSet<String>,
    ) -> Result<Vec<NormalizedTransferEvent>, SovereignError>;
}
