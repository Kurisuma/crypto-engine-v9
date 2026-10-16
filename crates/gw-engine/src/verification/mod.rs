#![forbid(unsafe_code)]
#![deny(
    clippy::float_arithmetic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::unwrap_used,
    clippy::indexing_slicing
)]

pub mod engine;
pub mod gates;

pub use engine::SovereignInvoiceVerificationEngine;
pub use gates::FiveGateEvaluator;
