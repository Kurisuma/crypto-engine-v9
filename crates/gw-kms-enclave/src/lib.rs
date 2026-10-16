#![forbid(unsafe_code)]
#![deny(
    clippy::float_arithmetic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::unwrap_used,
    clippy::indexing_slicing
)]

pub mod enclave;
pub mod key_storage;
pub mod parser;
pub mod policy;

pub use enclave::SovereignEnclaveKeySignerEngine;
pub use key_storage::{EnclaveKeyStore, ZeroizedKeyBuffer};
pub use parser::{ParsedPayloadDetails, PayloadAstParser};
pub use policy::StatelessPreSignPolicyGuardEngine;
