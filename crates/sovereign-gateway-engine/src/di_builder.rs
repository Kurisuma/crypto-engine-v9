#![forbid(unsafe_code)]
#![deny(
    clippy::float_arithmetic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::unwrap_used,
    clippy::indexing_slicing
)]

use crate::di_container::SovereignDiContainer;
use crate::instances::*;
use gw_enterprise_traits::{
    CrossNetworkLiquidityRouterEngine, EnclaveKeySignerEngine, InvoiceVerificationEngine,
    ObererOrmStorageEngine, ObererRowMacSigner, PreSignPolicyGuardEngine,
    PriorityRedundantCacheEngine, SovereignGatewayRuntimeEngine, TreasuryPortfolioEngine,
    UnifiedNetworkAdapterEngine,
};
use gw_primitives::{ChainId, SovereignError};
use std::collections::BTreeMap;
use std::sync::Arc;

/// Fluent Inversion of Control (IoC) Builder for assembling the Sovereign Dependency Injection Container.
pub struct SovereignGatewayDiBuilder {
    storage_engine: Option<Arc<dyn ObererOrmStorageEngine>>,
    cache_engine: Option<Arc<dyn PriorityRedundantCacheEngine>>,
    network_adapters: BTreeMap<ChainId, Arc<dyn UnifiedNetworkAdapterEngine>>,
    verification_engine: Option<Arc<dyn InvoiceVerificationEngine>>,
    treasury_engine: Option<Arc<dyn TreasuryPortfolioEngine>>,
    liquidity_router: Option<Arc<dyn CrossNetworkLiquidityRouterEngine>>,
    enclave_policy_guard: Option<Arc<dyn PreSignPolicyGuardEngine>>,
    enclave_signer: Option<Arc<dyn EnclaveKeySignerEngine>>,
    row_mac_key: [u8; 32],
}

impl Default for SovereignGatewayDiBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SovereignGatewayDiBuilder {
    /// Creates a new fluent DI builder initialized with default security keys.
    pub fn new() -> Self {
        Self {
            storage_engine: None,
            cache_engine: None,
            network_adapters: BTreeMap::new(),
            verification_engine: None,
            treasury_engine: None,
            liquidity_router: None,
            enclave_policy_guard: None,
            enclave_signer: None,
            row_mac_key: [0x5au8; 32],
        }
    }

    /// Sets the 32-byte Keyed Blake3 Row MAC secret key for storage engines.
    pub fn with_row_mac_key(mut self, key: [u8; 32]) -> Self {
        self.row_mac_key = key;
        self
    }

    /// Injects an existing `ObererOrmStorageEngine` implementation.
    pub fn with_storage_engine(mut self, storage: Arc<dyn ObererOrmStorageEngine>) -> Self {
        self.storage_engine = Some(storage);
        self
    }

    /// Injects an in-memory `:memory:` SQLite hybrid storage engine (ideal for unit/integration tests).
    pub fn with_in_memory_storage(mut self) -> Result<Self, SovereignError> {
        let storage = ObererHybridStorageEngineInstance::in_memory_testing(self.row_mac_key)?;
        self.storage_engine = Some(Arc::new(storage));
        Ok(self)
    }

    /// Injects a persistent on-disk WAL SQLite hybrid storage engine.
    pub fn with_persistent_wal(mut self, db_path: &str) -> Result<Self, SovereignError> {
        let storage = ObererHybridStorageEngineInstance::persistent_wal(db_path, self.row_mac_key)?;
        self.storage_engine = Some(Arc::new(storage));
        Ok(self)
    }

    /// Injects an existing `PriorityRedundantCacheEngine` implementation.
    pub fn with_cache_engine(mut self, cache: Arc<dyn PriorityRedundantCacheEngine>) -> Self {
        self.cache_engine = Some(cache);
        self
    }

    /// Injects an in-memory Priority-Redundant Cache Engine (IMPRCE) with specified byte ceiling.
    pub fn with_imprce_cache(mut self, max_bytes: usize) -> Self {
        let cache = PriorityRedundantCacheEngineInstance::new(self.row_mac_key, max_bytes);
        self.cache_engine = Some(Arc::new(cache));
        self
    }

    /// Registers a unified blockchain network adapter for a given `ChainId`.
    pub fn with_chain_adapter(
        mut self,
        chain_id: ChainId,
        adapter: Arc<dyn UnifiedNetworkAdapterEngine>,
    ) -> Self {
        self.network_adapters.insert(chain_id, adapter);
        self
    }

    /// Injects an existing `InvoiceVerificationEngine`.
    pub fn with_verification_engine(
        mut self,
        engine: Arc<dyn InvoiceVerificationEngine>,
    ) -> Self {
        self.verification_engine = Some(engine);
        self
    }

    /// Injects an existing `TreasuryPortfolioEngine`.
    pub fn with_treasury_engine(
        mut self,
        engine: Arc<dyn TreasuryPortfolioEngine>,
    ) -> Self {
        self.treasury_engine = Some(engine);
        self
    }

    /// Injects an existing `CrossNetworkLiquidityRouterEngine`.
    pub fn with_liquidity_router(
        mut self,
        engine: Arc<dyn CrossNetworkLiquidityRouterEngine>,
    ) -> Self {
        self.liquidity_router = Some(engine);
        self
    }

    /// Injects Zone C Stateless Pre-Sign Policy Guard and Key Signer instances.
    pub fn with_zone_c_enclave(
        mut self,
        policy_guard: Arc<dyn PreSignPolicyGuardEngine>,
        signer: Arc<dyn EnclaveKeySignerEngine>,
    ) -> Self {
        self.enclave_policy_guard = Some(policy_guard);
        self.enclave_signer = Some(signer);
        self
    }

    /// Builds and validates the complete `SovereignDiContainer`.
    pub fn build(self) -> Result<SovereignDiContainer, SovereignError> {
        let mac_signer = Arc::new(ObererRowMacSigner::new(self.row_mac_key));

        let storage: Arc<dyn ObererOrmStorageEngine> = match self.storage_engine {
            Some(s) => s,
            None => {
                let inst = ObererHybridStorageEngineInstance::in_memory_testing(self.row_mac_key)?;
                Arc::new(inst)
            }
        };

        let cache: Arc<dyn PriorityRedundantCacheEngine> = match self.cache_engine {
            Some(c) => c,
            None => {
                let inst = PriorityRedundantCacheEngineInstance::new(
                    self.row_mac_key,
                    64 * 1024 * 1024, // 64 MB default
                );
                Arc::new(inst)
            }
        };

        let verification: Arc<dyn InvoiceVerificationEngine> = match self.verification_engine {
            Some(v) => v,
            None => {
                let pool_engine = Arc::new(gw_chain_adapters::pool::DepositAddressPoolEngine::new(storage.clone()));
                let derivation = Arc::new(gw_chain_adapters::MultiChainWatchOnlyAdapter::new(
                    ChainId::EthereumMainnet,
                    "0x000000000000000000000000000000000000dEaD",
                    pool_engine,
                    [0x33u8; 32],
                ));
                let bridge = gw_primitives::MicroCreditsMathBridge;
                let inst = InvoiceVerificationEngineInstance::new(
                    storage.clone(),
                    derivation,
                    bridge,
                );
                Arc::new(inst)
            }
        };

        let treasury: Arc<dyn TreasuryPortfolioEngine> = match self.treasury_engine {
            Some(t) => t,
            None => {
                let inst = TreasuryPortfolioEngineInstance::new(self.row_mac_key);
                Arc::new(inst)
            }
        };

        let router: Arc<dyn CrossNetworkLiquidityRouterEngine> = match self.liquidity_router {
            Some(r) => r,
            None => {
                let inst = CrossNetworkLiquidityRouterEngineInstance::new(
                    mac_signer.clone(),
                    Some(storage.clone()),
                );
                Arc::new(inst)
            }
        };

        let runtime_engine = Arc::new(SovereignGatewayRuntimeEngine {
            storage_engine: storage.clone(),
            cache_engine: cache.clone(),
            network_adapters: self.network_adapters.clone(),
            verification_engine: verification.clone(),
            treasury_engine: treasury.clone(),
            liquidity_router_engine: router.clone(),
            enclave_policy_guard: self.enclave_policy_guard.clone(),
            enclave_signer: self.enclave_signer.clone(),
        });

        Ok(SovereignDiContainer {
            storage_engine: storage,
            cache_engine: cache,
            network_adapters: self.network_adapters,
            verification_engine: verification,
            treasury_engine: treasury,
            liquidity_router_engine: router,
            enclave_policy_guard: self.enclave_policy_guard,
            enclave_signer: self.enclave_signer,
            runtime_engine,
        })
    }
}
