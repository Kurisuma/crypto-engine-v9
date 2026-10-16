#![forbid(unsafe_code)]
#![deny(
    clippy::float_arithmetic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::unwrap_used,
    clippy::indexing_slicing
)]

use gw_enterprise_traits::{ConsensusBlockHeader, ContinuityCheckOutcome};
use gw_primitives::SovereignError;

/// Verifies DAG parent-hash continuity $\Lambda_{\text{cont}}(b_h, \mathcal{B})$:
///
/// $$\Lambda_{\text{cont}}(b_h, \mathcal{B}) \iff P(b_h) \equiv H(b_{h-1})$$
pub fn verify_parent_hash_continuity(
    incoming: &ConsensusBlockHeader,
    previous: Option<&ConsensusBlockHeader>,
) -> ContinuityCheckOutcome {
    let prev = match previous {
        Some(p) => p,
        None => return ContinuityCheckOutcome::CanonicalAppend,
    };

    // Sequential next block: height must be prev + 1 and parent hash must equal prev hash
    if incoming.block_number == prev.block_number.saturating_add(1) {
        if incoming.parent_block_hash == prev.block_hash {
            ContinuityCheckOutcome::CanonicalAppend
        } else {
            // Parent hash mismatch at height h: fork detected!
            ContinuityCheckOutcome::ReorgDetected {
                orphaned_from_height: prev.block_number,
                orphaned_to_height: prev.block_number,
                lca_height: prev.block_number.saturating_sub(1),
                lca_block_hash: prev.parent_block_hash.clone(),
            }
        }
    } else if incoming.block_number <= prev.block_number {
        // Incoming block height is at or behind previous tip -> competing fork or re-org branch
        ContinuityCheckOutcome::ReorgDetected {
            orphaned_from_height: incoming.block_number,
            orphaned_to_height: prev.block_number,
            lca_height: incoming.block_number.saturating_sub(1),
            lca_block_hash: String::new(),
        }
    } else {
        // Gap in block ingestion (incoming.block_number > prev.block_number + 1)
        ContinuityCheckOutcome::ReorgDetected {
            orphaned_from_height: prev.block_number.saturating_add(1),
            orphaned_to_height: incoming.block_number,
            lca_height: prev.block_number,
            lca_block_hash: prev.block_hash.clone(),
        }
    }
}

/// Backtracks backwards through the sliding window to find the Lowest Common Ancestor (LCA):
///
/// $$h_{\text{LCA}} = \max \{ k < h \mid H_{\text{canonical}}(k) \equiv H_{\text{fork}}(k) \}$$
pub fn find_lowest_common_ancestor(
    canonical_window: &[ConsensusBlockHeader],
    fork_branch: &[ConsensusBlockHeader],
) -> Result<i64, SovereignError> {
    if canonical_window.is_empty() {
        return Err(SovereignError::ConsensusReorgExceedsSlidingWindow {
            depth: 0,
            max_depth: 0,
        });
    }

    // 1. Compare headers from highest common height downward
    for fork_block in fork_branch.iter().rev() {
        for canon_block in canonical_window.iter().rev() {
            if canon_block.block_number == fork_block.block_number
                && canon_block.block_hash == fork_block.block_hash
            {
                return Ok(canon_block.block_number);
            }
        }
    }

    // 2. Check if the parent of any fork block matches a canonical block (fork root connection)
    for fork_block in fork_branch.iter() {
        for canon_block in canonical_window.iter().rev() {
            if canon_block.block_hash == fork_block.parent_block_hash {
                return Ok(canon_block.block_number);
            }
        }
    }

    // If no matching block hash found in window, reorg exceeds window depth
    Err(SovereignError::ConsensusReorgExceedsSlidingWindow {
        depth: fork_branch.len() as u64,
        max_depth: canonical_window.len() as u64,
    })
}
