#![forbid(unsafe_code)]
#![deny(
    clippy::float_arithmetic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::unwrap_used,
    clippy::indexing_slicing
)]

use crate::indexing::net_balance::calculate_net_credited_amount;
use gw_enterprise_traits::{ConsensusBlockHeader, NormalizedTransferEvent};
use gw_imprce::BlockedBloomFilter;
use gw_primitives::{ChainArchitecture, ChainId, RawAssetAmount, SovereignError};
use std::collections::HashSet;
use std::sync::Arc;

/// Raw incoming candidate transfer in an unparsed native block.
#[derive(Debug, Clone)]
pub struct RawCandidateTransfer {
    pub tx_hash: String,
    pub event_index: u32,
    pub sender: String,
    pub recipient: String,
    pub routing_reference: Option<String>,
    pub token_contract: String,
    pub declared_amount: RawAssetAmount,
    pub pre_balance: Option<RawAssetAmount>,
    pub post_balance: Option<RawAssetAmount>,
    pub execution_succeeded: bool,
}

/// 2-Stage High-Performance Block Scanner:
/// Stage 1: Lock-Free Blocked Bloom Filter rejection (~4.2 ns rejection).
/// Stage 2: O(1) P1 Active Watchlist hit verification.
pub struct BlockTransactionScanner {
    chain_id: ChainId,
    architecture: ChainArchitecture,
    bloom_filter: Option<Arc<BlockedBloomFilter>>,
}

impl BlockTransactionScanner {
    pub fn new(
        chain_id: ChainId,
        bloom_filter: Option<Arc<BlockedBloomFilter>>,
    ) -> Self {
        Self {
            chain_id,
            architecture: chain_id.architecture(),
            bloom_filter,
        }
    }

    /// Scans a batch of candidate transfers against active watchlist keys.
    pub fn scan_candidates(
        &self,
        block_header: &ConsensusBlockHeader,
        current_tip_height: i64,
        candidates: &[RawCandidateTransfer],
        active_watchlist_keys: &HashSet<String>,
    ) -> Result<Vec<NormalizedTransferEvent>, SovereignError> {
        let mut matched_events = Vec::new();
        let conf_diff = current_tip_height.saturating_sub(block_header.block_number).saturating_add(1);
        let confirmations = u32::try_from(conf_diff.max(0)).unwrap_or(0);

        for cand in candidates {
            // Check lookup keys:
            // 1. Direct deposit address
            // 2. Routing reference (Solana reference pubkey or TON memo tag)
            let recipient_key = cand.recipient.to_lowercase();
            let memo_key = cand.routing_reference.as_ref().map(|r| r.to_lowercase());

            // Stage 1: Optional Blocked Bloom Filter pre-filter
            if let Some(bloom) = &self.bloom_filter {
                let recipient_hit = bloom.contains(&recipient_key);
                let memo_hit = memo_key
                    .as_ref()
                    .map(|m| bloom.contains(m))
                    .unwrap_or(false);

                // If neither recipient nor memo is present in the bloom filter, reject immediately!
                if !recipient_hit && !memo_hit {
                    continue;
                }
            }

            // Stage 2: O(1) exact match in active watchlist keys
            let is_recipient_watched = active_watchlist_keys.contains(&recipient_key)
                || active_watchlist_keys.contains(&cand.recipient);
            let is_memo_watched = memo_key
                .as_ref()
                .map(|m| active_watchlist_keys.contains(m))
                .unwrap_or(false);

            if !is_recipient_watched && !is_memo_watched {
                continue;
            }

            let watchlist_key = if is_recipient_watched {
                cand.recipient.clone()
            } else {
                cand.routing_reference.clone().unwrap_or_default()
            };

            let net_credited = match (cand.pre_balance, cand.post_balance) {
                (Some(pre), Some(post)) => {
                    calculate_net_credited_amount(pre, post, cand.declared_amount)?
                }
                _ => cand.declared_amount,
            };

            matched_events.push(NormalizedTransferEvent {
                chain_id: self.chain_id,
                architecture: self.architecture,
                tx_hash: cand.tx_hash.clone(),
                event_index: cand.event_index,
                block_header: block_header.clone(),
                watchlist_lookup_key: watchlist_key,
                sender_address: cand.sender.clone(),
                recipient_address: cand.recipient.clone(),
                token_contract: cand.token_contract.clone(),
                net_credited_amount: net_credited,
                execution_succeeded: cand.execution_succeeded,
                confirmations,
            });
        }

        Ok(matched_events)
    }
}
