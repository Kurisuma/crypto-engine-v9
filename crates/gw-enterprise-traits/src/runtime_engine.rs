#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic, clippy::unwrap_used)]

use crate::cache_engine_trait::PriorityRedundantCacheEngine;
use crate::enclave_signer_engine_trait::EnclaveKeySignerEngine;
use crate::liquidity_router_engine_trait::CrossNetworkLiquidityRouterEngine;
use crate::oberer_storage_engine::{
    EngineHealthSnapshot, EngineIntegrityReport, ObererOrmStorageEngine, SovereignEngine,
};
use crate::policy_guard_engine_trait::PreSignPolicyGuardEngine;
use crate::treasury_engine_trait::TreasuryPortfolioEngine;
use crate::unified_adapter_engine_trait::UnifiedNetworkAdapterEngine;
use crate::verification_engine_trait::InvoiceVerificationEngine;
use async_trait::async_trait;
use gw_primitives::{ChainId, SecurityZone, SovereignError};
use std::collections::BTreeMap;
use std::sync::Arc;

/// Master OOP Composition Root & Runtime Engine.
#[derive(Clone)]
pub struct SovereignGatewayRuntimeEngine {
    pub storage_engine: Arc<dyn ObererOrmStorageEngine>,
    pub cache_engine: Arc<dyn PriorityRedundantCacheEngine>,
    pub network_adapters: BTreeMap<ChainId, Arc<dyn UnifiedNetworkAdapterEngine>>,
    pub verification_engine: Arc<dyn InvoiceVerificationEngine>,
    pub treasury_engine: Arc<dyn TreasuryPortfolioEngine>,
    pub liquidity_router_engine: Arc<dyn CrossNetworkLiquidityRouterEngine>,
    pub enclave_policy_guard: Option<Arc<dyn PreSignPolicyGuardEngine>>,
    pub enclave_signer: Option<Arc<dyn EnclaveKeySignerEngine>>,
}

#[async_trait]
impl SovereignEngine for SovereignGatewayRuntimeEngine {
    fn engine_id(&self) -> &'static str {
        "SOVEREIGN_GATEWAY_MASTER_RUNTIME_ENGINE_V9"
    }

    fn security_zone(&self) -> SecurityZone {
        SecurityZone::ZoneAPublicWatchOnly
    }

    async fn initialize_engine(&self) -> Result<(), SovereignError> {
        self.storage_engine.initialize_engine().await?;
        self.cache_engine.initialize_engine().await?;
        self.cache_engine
            .hydrate_from_oberer_hybrid(self.storage_engine.as_ref())
            .await?;

        for adapter in self.network_adapters.values() {
            adapter.initialize_engine().await?;
        }

        self.verification_engine.initialize_engine().await?;
        self.treasury_engine.initialize_engine().await?;
        self.liquidity_router_engine.initialize_engine().await?;

        Ok(())
    }

    async fn audit_cryptographic_integrity(&self) -> Result<EngineIntegrityReport, SovereignError> {
        let storage_audit = self.storage_engine.audit_cryptographic_integrity().await?;
        let cache_audit = self.cache_engine.audit_cryptographic_integrity().await?;

        Ok(EngineIntegrityReport {
            engine_id: self.engine_id().to_string(),
            security_zone: self.security_zone(),
            verified_records_or_cells: storage_audit
                .verified_records_or_cells
                .saturating_add(cache_audit.verified_records_or_cells),
            detected_corruptions_or_tampers: storage_audit
                .detected_corruptions_or_tampers
                .saturating_add(cache_audit.detected_corruptions_or_tampers),
            autonomous_self_heals: storage_audit
                .autonomous_self_heals
                .saturating_add(cache_audit.autonomous_self_heals),
            hash_chain_head_hex: storage_audit.hash_chain_head_hex,
            audit_passed: storage_audit.audit_passed && cache_audit.audit_passed,
        })
    }

    async fn health_snapshot(&self) -> Result<EngineHealthSnapshot, SovereignError> {
        let storage_health = self.storage_engine.health_snapshot().await?;
        let cache_health = self.cache_engine.health_snapshot().await?;

        Ok(EngineHealthSnapshot {
            engine_id: self.engine_id().to_string(),
            security_zone: self.security_zone(),
            is_healthy: storage_health.is_healthy && cache_health.is_healthy,
            active_workers_or_shards: storage_health
                .active_workers_or_shards
                .saturating_add(cache_health.active_workers_or_shards),
            total_operations_processed: storage_health
                .total_operations_processed
                .saturating_add(cache_health.total_operations_processed),
            p99_operation_latency_us: storage_health
                .p99_operation_latency_us
                .max(cache_health.p99_operation_latency_us),
            memory_allocated_bytes: storage_health
                .memory_allocated_bytes
                .saturating_add(cache_health.memory_allocated_bytes),
        })
    }

    async fn shutdown_gracefully(&self) -> Result<(), SovereignError> {
        self.liquidity_router_engine.shutdown_gracefully().await?;
        self.treasury_engine.shutdown_gracefully().await?;
        self.verification_engine.shutdown_gracefully().await?;
        for adapter in self.network_adapters.values() {
            adapter.shutdown_gracefully().await?;
        }
        self.cache_engine.shutdown_gracefully().await?;
        self.storage_engine.shutdown_gracefully().await?;
        Ok(())
    }
}
