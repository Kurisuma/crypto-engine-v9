#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic, clippy::unwrap_used)]

use crate::oberer_storage_engine::SovereignEngine;
use crate::verification_engine_trait::{StateVerifiedComplete, TypestateInvoice};
use alloy_primitives::U256;
use async_trait::async_trait;
use gw_primitives::{
    ChainId, LogicalAsset, PhysicalAssetId, RawAssetAmount, SovereignError, WalletTier,
};
use oberer_orm::MicroCredits;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

/// Immutable FIFO Tax Lot domain model backed by `CostBasisLotRow`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FifoCostBasisLot {
    pub lot_id: i64,
    pub lot_uuid: String,
    pub source_invoice_id: i64,
    pub asset: PhysicalAssetId,
    pub initial_amount: RawAssetAmount,
    pub remaining_amount: RawAssetAmount,
    pub acquisition_price_mc: MicroCredits,
    pub acquired_at_unix: i64,
    pub version: i64,
    pub row_mac: Vec<u8>,
}

/// Unified cross-network portfolio rollup for a single LogicalAsset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnifiedAssetPortfolioSummary {
    pub logical_asset: LogicalAsset,
    pub total_canonical_18_units: U256,
    pub total_market_valuation_mc: MicroCredits,
    pub total_fifo_cost_basis_mc: MicroCredits,
    pub unrealized_pnl_microcredits: i64,
    pub valuation_by_tier_mc: BTreeMap<WalletTier, MicroCredits>,
    pub valuation_by_chain_mc: BTreeMap<ChainId, MicroCredits>,
}

/// Hash-chained audit record persisted upon FIFO disposal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RealizedPnlSettlementReport {
    pub settlement_id: i64,
    pub settlement_uuid: Uuid,
    pub sequence_no: i64,
    pub logical_asset: LogicalAsset,
    pub chain_id: ChainId,
    pub disposed_amount: RawAssetAmount,
    pub consumed_lot_slices: Vec<(i64, RawAssetAmount, MicroCredits)>,
    pub total_cost_basis_mc: MicroCredits,
    pub gross_proceeds_mc: MicroCredits,
    pub network_fee_burned_mc: MicroCredits,
    pub net_realized_pnl_microcredits: i64,
    pub prev_chain_hash_hex: String,
    pub entry_chain_hash_hex: String,
}

/// Engine Trait X: Unified Cross-Network Asset Aggregation & FIFO Cost-Basis Calculus Engine.
#[async_trait]
pub trait TreasuryPortfolioEngine: SovereignEngine {
    /// Creates and seals an immutable `CostBasisLotRow` inside storage upon invoice completion.
    async fn record_verified_invoice_fifo_lot(
        &self,
        verified_invoice: &TypestateInvoice<StateVerifiedComplete>,
    ) -> Result<FifoCostBasisLot, SovereignError>;

    /// Aggregates all wallet holdings into a unified per-LogicalAsset balance sheet with Unrealized P&L.
    async fn compute_unified_global_portfolio(
        &self,
        spot_prices_mc: &BTreeMap<LogicalAsset, MicroCredits>,
    ) -> Result<BTreeMap<LogicalAsset, UnifiedAssetPortfolioSummary>, SovereignError>;

    /// Consumes open FIFO tax lots in strict chronological order and generates a hash-chained settlement report.
    async fn execute_fifo_lot_consumption(
        &self,
        asset: &PhysicalAssetId,
        disposed_amount: RawAssetAmount,
        execution_price_mc: MicroCredits,
        network_fee_mc: MicroCredits,
    ) -> Result<RealizedPnlSettlementReport, SovereignError>;
}
