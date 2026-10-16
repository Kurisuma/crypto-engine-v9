#![forbid(unsafe_code)]
#![deny(
    clippy::float_arithmetic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::unwrap_used,
    clippy::indexing_slicing
)]

use gw_enterprise_traits::{
    CrossNetworkLiquidityRouterEngine, EnclaveKeySignerEngine, InvoiceVerificationEngine,
    ObererOrmStorageEngine, PreSignPolicyGuardEngine, PriorityRedundantCacheEngine,
    SovereignEngine, SovereignGatewayRuntimeEngine, TreasuryPortfolioEngine,
    UnifiedNetworkAdapterEngine,
};
use gw_primitives::{ChainId, SovereignError};
use std::collections::BTreeMap;
use std::sync::Arc;

/// Type-Safe Dependency Injection (DI) Container for the Sovereign Gateway & Treasury Engine.
///
/// Holds thread-safe `Arc<dyn Engine>` handles for all 11 enterprise OOP engine traits
/// and the master `SovereignGatewayRuntimeEngine` composition root.
/// Fully implements `Clone + Send + Sync + 'static`.
#[derive(Clone)]
pub struct SovereignDiContainer {
    pub storage_engine: Arc<dyn ObererOrmStorageEngine>,
    pub cache_engine: Arc<dyn PriorityRedundantCacheEngine>,
    pub network_adapters: BTreeMap<ChainId, Arc<dyn UnifiedNetworkAdapterEngine>>,
    pub verification_engine: Arc<dyn InvoiceVerificationEngine>,
    pub treasury_engine: Arc<dyn TreasuryPortfolioEngine>,
    pub liquidity_router_engine: Arc<dyn CrossNetworkLiquidityRouterEngine>,
    pub enclave_policy_guard: Option<Arc<dyn PreSignPolicyGuardEngine>>,
    pub enclave_signer: Option<Arc<dyn EnclaveKeySignerEngine>>,
    pub runtime_engine: Arc<SovereignGatewayRuntimeEngine>,
}

impl SovereignDiContainer {
    /// Returns the active Storage Engine.
    pub fn storage(&self) -> Arc<dyn ObererOrmStorageEngine> {
        self.storage_engine.clone()
    }

    /// Returns the active Priority-Redundant Cache Engine (IMPRCE).
    pub fn cache(&self) -> Arc<dyn PriorityRedundantCacheEngine> {
        self.cache_engine.clone()
    }

    /// Returns the 5-Gate Invoice Verification Engine.
    pub fn verification(&self) -> Arc<dyn InvoiceVerificationEngine> {
        self.verification_engine.clone()
    }

    /// Returns the FIFO Unified Treasury Portfolio Engine.
    pub fn treasury(&self) -> Arc<dyn TreasuryPortfolioEngine> {
        self.treasury_engine.clone()
    }

    /// Returns the Cross-Network Liquidity Router Engine.
    pub fn router(&self) -> Arc<dyn CrossNetworkLiquidityRouterEngine> {
        self.liquidity_router_engine.clone()
    }

    /// Returns the optional Zone C Stateless Pre-Sign Policy Guard.
    pub fn policy_guard(&self) -> Option<Arc<dyn PreSignPolicyGuardEngine>> {
        self.enclave_policy_guard.clone()
    }

    /// Returns the optional Zone C Enclave Key Signer.
    pub fn signer(&self) -> Option<Arc<dyn EnclaveKeySignerEngine>> {
        self.enclave_signer.clone()
    }

    /// Returns the Master Composition Root Runtime Engine.
    pub fn runtime(&self) -> Arc<SovereignGatewayRuntimeEngine> {
        self.runtime_engine.clone()
    }

    /// Returns the registered adapter for a specific blockchain network.
    pub fn adapter(&self, chain_id: ChainId) -> Option<Arc<dyn UnifiedNetworkAdapterEngine>> {
        self.network_adapters.get(&chain_id).cloned()
    }

    /// Executes the full sequential boot sequence across all injected engines:
    /// Storage Engine -> IMPRCE Hydration -> Adapters -> Verification -> Treasury -> Router.
    pub async fn initialize_all(&self) -> Result<(), SovereignError> {
        self.runtime_engine.initialize_engine().await
    }
}
