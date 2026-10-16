#![forbid(unsafe_code)]
#![deny(
    clippy::float_arithmetic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::unwrap_used,
    clippy::indexing_slicing
)]

pub mod dsl;
pub mod engine;
pub mod enums;
pub mod mac;
pub mod migration;
pub mod model_impls;
pub mod tables;
pub mod views;

pub use dsl::{Schema, TriggerSchema, ViewSchema};
pub use engine::{
    ObererHybridStorageEngineInstance, ObererOrmStorageEngine, ObererOrmStorageEngineExt,
    SovereignEngine,
};
pub use gw_primitives::{EngineHealthSnapshot, EngineIntegrityReport};
pub use enums::*;
pub use mac::{ObererRowMacSigner, TamperEvidentOrmModel};
pub use migration::bootstrap_sovereign_schema;
pub use tables::*;
pub use views::*;
