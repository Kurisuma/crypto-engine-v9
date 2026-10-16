#![forbid(unsafe_code)]
#![deny(
    clippy::float_arithmetic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::unwrap_used,
    clippy::indexing_slicing
)]

pub mod amount;
pub mod asset;
pub mod bridge;
pub mod codec;
pub mod error;
pub mod zone;

pub use amount::RawAssetAmount;
pub use asset::{
    CachePriorityTier, ChainArchitecture, ChainId, LogicalAsset, PhysicalAssetId, WalletTier,
};
pub use bridge::MicroCreditsMathBridge;
pub use codec::ObererU256Codec;
pub use error::SovereignError;
pub use zone::{EngineHealthSnapshot, EngineIntegrityReport, SecurityZone};
