#![forbid(unsafe_code)]
#![deny(
    clippy::float_arithmetic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::unwrap_used,
    clippy::indexing_slicing
)]

use crate::indexing::dag_continuity::{find_lowest_common_ancestor, verify_parent_hash_continuity};
use crate::indexing::scanner::{BlockTransactionScanner, RawCandidateTransfer};
use async_trait::async_trait;
use gw_enterprise_traits::{
    ConsensusBlockHeader, ConsensusIndexingEngine, ContinuityCheckOutcome,
    EngineHealthSnapshot, EngineIntegrityReport, NormalizedTransferEvent,
    ObererOrmStorageEngine, SecurityZone, SovereignEngine,
    TamperEvidentOrmModel,
};
use gw_primitives::{ChainId, SovereignError};
use gw_storage_oberer::ConsensusBlockHeaderRow;
use oberer_orm::{OrmResultExt, Table, TableOps};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

use gw_imprce::BlockedBloomFilter;

type SimulatedBlockFeed = Arc<RwLock<Vec<(ConsensusBlockHeader, Vec<RawCandidateTransfer>)>>>;

/// Implementation of the Sovereign Consensus Indexing Engine.
pub struct SovereignConsensusIndexingEngine {
    chain_id: ChainId,
    storage: Arc<dyn ObererOrmStorageEngine>,
    scanner: BlockTransactionScanner,
    sliding_window: Arc<RwLock<Vec<ConsensusBlockHeader>>>,
    max_window_depth: usize,
    simulated_block_feed: SimulatedBlockFeed,
}

impl SovereignConsensusIndexingEngine {
    pub fn new(
        chain_id: ChainId,
        storage: Arc<dyn ObererOrmStorageEngine>,
        bloom_filter: Option<Arc<BlockedBloomFilter>>,
        max_window_depth: usize,
    ) -> Self {
        let scanner = BlockTransactionScanner::new(chain_id, bloom_filter);
        Self {
            chain_id,
            storage,
            scanner,
            sliding_window: Arc::new(RwLock::new(Vec::with_capacity(max_window_depth))),
            max_window_depth,
            simulated_block_feed: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Injects a block and its transfers into the engine's ingestion feed.
    pub async fn inject_block_feed(
        &self,
        header: ConsensusBlockHeader,
        transfers: Vec<RawCandidateTransfer>,
    ) {
        let mut feed = self.simulated_block_feed.write().await;
        feed.push((header, transfers));
    }

    /// Stores a canonical block header into the SQLite consensus_block_ring with Row MAC sealing.
    pub async fn persist_block_header(
        &self,
        header: &ConsensusBlockHeader,
    ) -> Result<(), SovereignError> {
        let conn = self.storage.memory_conn();
        let table = Table::<ConsensusBlockHeaderRow>::new(conn);

        let mut row = ConsensusBlockHeaderRow {
            id: 0,
            chain_id: self.chain_id.as_i64(),
            block_number: header.block_number,
            block_hash: header.block_hash.clone(),
            parent_block_hash: header.parent_block_hash.clone(),
            state_root: header.state_root_or_merkle_root.clone(),
            timestamp_unix: header.timestamp_unix,
            is_finalized: if header.is_finalized { 1 } else { 0 },
            version: 1,
            row_mac: Vec::new(),
        };

        let mac = self.storage.row_mac_signer().compute_row_mac(
            ConsensusBlockHeaderRow::TABLE_LABEL,
            &row.pk_string(),
            1,
            &row.canonical_security_bytes(),
        );
        row.assign_row_mac(mac);

        table.insert(&row).unenvelope().map_err(|e| {
            SovereignError::ObererOrmError {
                detail: format!("Failed to persist block header {}: {e}", header.block_number),
            }
        })?;

        Ok(())
    }
}

#[async_trait]
impl SovereignEngine for SovereignConsensusIndexingEngine {
    fn engine_id(&self) -> &'static str {
        "SOVEREIGN_CONSENSUS_INDEXING_ENGINE"
    }

    fn security_zone(&self) -> SecurityZone {
        SecurityZone::ZoneBPrivateNodePlane
    }

    async fn initialize_engine(&self) -> Result<(), SovereignError> {
        // Hydrate sliding window ring buffer from SQLite
        let conn = self.storage.memory_conn();
        let table = Table::<ConsensusBlockHeaderRow>::new(conn);
        let rows = table.find_all().unenvelope().map_err(|e| {
            SovereignError::ObererOrmError {
                detail: format!("Failed to hydrate consensus headers: {e}"),
            }
        })?;

        let mut window = self.sliding_window.write().await;
        window.clear();
        for r in rows {
            if r.chain_id == self.chain_id.as_i64() {
                window.push(ConsensusBlockHeader {
                    chain_id: self.chain_id,
                    block_number: r.block_number,
                    block_hash: r.block_hash,
                    parent_block_hash: r.parent_block_hash,
                    state_root_or_merkle_root: r.state_root,
                    timestamp_unix: r.timestamp_unix,
                    is_finalized: r.is_finalized != 0,
                });
            }
        }
        window.sort_by_key(|h| h.block_number);

        Ok(())
    }

    async fn audit_cryptographic_integrity(&self) -> Result<EngineIntegrityReport, SovereignError> {
        let window = self.sliding_window.read().await;
        let mut previous: Option<&ConsensusBlockHeader> = None;
        let mut corrupted_blocks = 0_usize;

        for header in window.iter() {
            if let Some(prev) = previous {
                if header.parent_block_hash != prev.block_hash {
                    corrupted_blocks = corrupted_blocks.saturating_add(1);
                }
            }
            previous = Some(header);
        }

        let is_valid = corrupted_blocks == 0;
        Ok(EngineIntegrityReport {
            engine_id: self.engine_id().to_string(),
            security_zone: self.security_zone(),
            verified_records_or_cells: window.len() as u64,
            detected_corruptions_or_tampers: corrupted_blocks as u64,
            autonomous_self_heals: 0,
            hash_chain_head_hex: String::new(),
            audit_passed: is_valid,
        })
    }

    async fn health_snapshot(&self) -> Result<EngineHealthSnapshot, SovereignError> {
        let window = self.sliding_window.read().await;
        let latest_block = window.last().map(|h| h.block_number).unwrap_or(0);
        let _ = latest_block;
        Ok(EngineHealthSnapshot {
            engine_id: self.engine_id().to_string(),
            security_zone: self.security_zone(),
            is_healthy: true,
            active_workers_or_shards: 1,
            total_operations_processed: window.len() as u64,
            p99_operation_latency_us: 10,
            memory_allocated_bytes: window.len().saturating_mul(128),
        })
    }

    async fn shutdown_gracefully(&self) -> Result<(), SovereignError> {
        Ok(())
    }
}

#[async_trait]
impl ConsensusIndexingEngine for SovereignConsensusIndexingEngine {
    fn chain_id(&self) -> ChainId {
        self.chain_id
    }

    async fn query_consensus_heights(&self) -> Result<(i64, i64), SovereignError> {
        let window = self.sliding_window.read().await;
        let tip = window.last().map(|h| h.block_number).unwrap_or(0);
        let finalized = window
            .iter()
            .rev()
            .find(|h| h.is_finalized)
            .map(|h| h.block_number)
            .unwrap_or(0);
        Ok((tip, finalized))
    }

    async fn fetch_and_verify_block_continuity(
        &self,
        block_height: i64,
        previous_cached_header: Option<&ConsensusBlockHeader>,
    ) -> Result<(ConsensusBlockHeader, ContinuityCheckOutcome), SovereignError> {
        let feed = self.simulated_block_feed.read().await;
        let (header, _) = feed
            .iter()
            .find(|(h, _)| h.block_number == block_height)
            .cloned()
            .ok_or_else(|| SovereignError::InvalidBlockchainAddress {
                chain: self.chain_id,
                address: format!("Block {block_height} not found in ingestion stream"),
            })?;

        let outcome = verify_parent_hash_continuity(&header, previous_cached_header);
        match &outcome {
            ContinuityCheckOutcome::CanonicalAppend => {
                let mut window = self.sliding_window.write().await;
                window.push(header.clone());
                if window.len() > self.max_window_depth {
                    let _ = window.remove(0);
                }
                self.persist_block_header(&header).await?;
            }
            ContinuityCheckOutcome::ReorgDetected { lca_height, .. } => {
                // Truncate sliding window back to LCA
                let mut window = self.sliding_window.write().await;
                window.retain(|h| h.block_number <= *lca_height);
            }
        }

        Ok((header, outcome))
    }

    async fn find_reorg_lowest_common_ancestor(
        &self,
        divergent_height: i64,
        cached_window: &[ConsensusBlockHeader],
    ) -> Result<i64, SovereignError> {
        let feed = self.simulated_block_feed.read().await;
        let fork_branch: Vec<ConsensusBlockHeader> = feed
            .iter()
            .filter(|(h, _)| h.block_number <= divergent_height)
            .map(|(h, _)| h.clone())
            .collect();

        find_lowest_common_ancestor(cached_window, &fork_branch)
    }

    async fn scan_block_for_watched_transfers(
        &self,
        block_header: &ConsensusBlockHeader,
        current_tip_height: i64,
        active_watchlist_keys: &HashSet<String>,
    ) -> Result<Vec<NormalizedTransferEvent>, SovereignError> {
        let feed = self.simulated_block_feed.read().await;
        let candidates = feed
            .iter()
            .find(|(h, _)| h.block_number == block_header.block_number && h.block_hash == block_header.block_hash)
            .map(|(_, txs)| txs.as_slice())
            .unwrap_or(&[]);

        self.scanner.scan_candidates(
            block_header,
            current_tip_height,
            candidates,
            active_watchlist_keys,
        )
    }

    async fn scan_block_range_batch(
        &self,
        start_height: i64,
        end_height: i64,
        active_watchlist_keys: &HashSet<String>,
    ) -> Result<Vec<NormalizedTransferEvent>, SovereignError> {
        let feed = self.simulated_block_feed.read().await;
        let mut all_events = Vec::new();

        for (header, txs) in feed.iter() {
            if header.block_number >= start_height && header.block_number <= end_height {
                let events = self.scanner.scan_candidates(
                    header,
                    end_height,
                    txs,
                    active_watchlist_keys,
                )?;
                all_events.extend(events);
            }
        }

        Ok(all_events)
    }
}
