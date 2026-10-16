#![forbid(unsafe_code)]
#![deny(
    clippy::float_arithmetic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::unwrap_used,
    clippy::indexing_slicing
)]

//! # Sovereign Gateway Engine SDK
//!
//! Standalone library crate exposing the Inversion of Control (IoC) Dependency Injection Container
//! (`SovereignDiContainer`), fluent builder (`SovereignGatewayDiBuilder`), and web framework
//! extractors for Axum.

pub mod di_builder;
pub mod di_container;
pub mod instances;
pub mod prelude;

#[cfg(feature = "axum-integration")]
pub mod axum_ext;

pub use di_builder::SovereignGatewayDiBuilder;
pub use di_container::SovereignDiContainer;
pub use instances::*;
