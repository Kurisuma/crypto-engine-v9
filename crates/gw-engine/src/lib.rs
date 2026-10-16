#![forbid(unsafe_code)]
#![deny(
    clippy::float_arithmetic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::unwrap_used,
    clippy::indexing_slicing
)]

pub mod indexing;
pub mod verification;

pub use indexing::{
    calculate_net_credited_amount, find_lowest_common_ancestor, verify_parent_hash_continuity,
    BlockTransactionScanner, RawCandidateTransfer, SovereignConsensusIndexingEngine,
};
pub use verification::{FiveGateEvaluator, SovereignInvoiceVerificationEngine};
