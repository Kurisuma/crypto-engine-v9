#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic, clippy::unwrap_used)]

use crate::oberer_storage_engine::SovereignEngine;
use crate::sweep_engine_trait::UnsignedTransactionEnvelope;
use async_trait::async_trait;
use gw_primitives::{ChainId, LogicalAsset, PhysicalAssetId, RawAssetAmount, SovereignError};
use oberer_orm::MicroCredits;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Per-network liquidity threshold and split allocation policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkLiquidityPolicy {
    pub asset: PhysicalAssetId,
    pub tier1_relayer_address: String,
    pub tier2_hot_wallet_address: String,
    pub tier2_key_locator: String,
    pub tier3_cold_vault_address: String,
    pub tier4_market_exchange_address: String,
    pub hot_wallet_ceiling_mc: MicroCredits,
    pub cold_vault_split_bps: u16,
    pub market_exchange_split_bps: u16,
    pub min_relayer_gas_raw: RawAssetAmount,
    pub target_relayer_gas_raw: RawAssetAmount,
}

/// Global routing execution mode across all chains.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GlobalRoutingDirective {
    EmergencyOrScheduledFlushToCold,
    FlushAllToMarketExchange,
    PolicyDrivenProportionalSplit,
}

/// Engine Trait XI: Cross-Network Liquidity Routing, Relayer Auto-Refill & Consolidation Engine.
#[async_trait]
pub trait CrossNetworkLiquidityRouterEngine: SovereignEngine {
    /// Evaluates all Tier 1 Gas Relayers across all networks and generates unsigned top-up envelopes.
    async fn plan_tier1_relayer_gas_refills(
        &self,
        policies: &[NetworkLiquidityPolicy],
    ) -> Result<Vec<UnsignedTransactionEnvelope>, SovereignError>;

    /// Plans optimal cross-network transfers to Cold Storage and/or Market Exchanges.
    async fn plan_global_liquidity_transfers(
        &self,
        filter_asset: Option<LogicalAsset>,
        directive: GlobalRoutingDirective,
        policies: &[NetworkLiquidityPolicy],
        spot_prices_mc: &BTreeMap<LogicalAsset, MicroCredits>,
    ) -> Result<Vec<UnsignedTransactionEnvelope>, SovereignError>;

    /// Computes the minimum economical sweep threshold for Tier 0 customer deposit addresses.
    async fn calculate_economical_sweep_threshold(
        &self,
        chain_id: ChainId,
        asset: &PhysicalAssetId,
        max_fee_to_principal_bps: u16,
    ) -> Result<RawAssetAmount, SovereignError>;
}
