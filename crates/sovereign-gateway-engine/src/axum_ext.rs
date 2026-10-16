#![forbid(unsafe_code)]
#![deny(
    clippy::float_arithmetic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::unwrap_used,
    clippy::indexing_slicing
)]

use crate::di_container::SovereignDiContainer;
use axum::extract::FromRef;
use gw_enterprise_traits::{
    CrossNetworkLiquidityRouterEngine, InvoiceVerificationEngine, ObererOrmStorageEngine,
    PriorityRedundantCacheEngine, SovereignGatewayRuntimeEngine, TreasuryPortfolioEngine,
};
use std::sync::Arc;

impl FromRef<SovereignDiContainer> for Arc<dyn ObererOrmStorageEngine> {
    fn from_ref(container: &SovereignDiContainer) -> Self {
        container.storage_engine.clone()
    }
}

impl FromRef<SovereignDiContainer> for Arc<dyn PriorityRedundantCacheEngine> {
    fn from_ref(container: &SovereignDiContainer) -> Self {
        container.cache_engine.clone()
    }
}

impl FromRef<SovereignDiContainer> for Arc<dyn InvoiceVerificationEngine> {
    fn from_ref(container: &SovereignDiContainer) -> Self {
        container.verification_engine.clone()
    }
}

impl FromRef<SovereignDiContainer> for Arc<dyn TreasuryPortfolioEngine> {
    fn from_ref(container: &SovereignDiContainer) -> Self {
        container.treasury_engine.clone()
    }
}

impl FromRef<SovereignDiContainer> for Arc<dyn CrossNetworkLiquidityRouterEngine> {
    fn from_ref(container: &SovereignDiContainer) -> Self {
        container.liquidity_router_engine.clone()
    }
}

impl FromRef<SovereignDiContainer> for Arc<SovereignGatewayRuntimeEngine> {
    fn from_ref(container: &SovereignDiContainer) -> Self {
        container.runtime_engine.clone()
    }
}
