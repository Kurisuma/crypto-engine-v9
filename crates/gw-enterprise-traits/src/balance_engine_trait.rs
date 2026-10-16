#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic, clippy::unwrap_used)]

use crate::oberer_storage_engine::SovereignEngine;
use async_trait::async_trait;
use gw_primitives::{ChainId, PhysicalAssetId, RawAssetAmount, SovereignError};
use serde::{Deserialize, Serialize};

/// Chain-specific execution resource and gas relayer health telemetry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelayerResourceHealth {
    pub chain_id: ChainId,
    pub relayer_address: String,
    pub native_gas_balance: RawAssetAmount,
    pub min_required_gas_balance: RawAssetAmount,
    /// For Tron (TVM): Available delegated Stake 2.0 Energy & Bandwidth units.
    pub available_tvm_energy: u64,
    pub available_tvm_bandwidth: u64,
    /// For Solana (SVM): Count of open temporary PDA Token Accounts holding reclaimable rent.
    pub reclaimable_svm_rent_lamports: u64,
    pub next_pending_nonce: u64,
    pub needs_immediate_refill: bool,
}

/// Unspent Transaction Output (UTXO) or discrete Move Coin<T> object descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpendableDiscreteOutput {
    pub output_id: String,
    pub owner_address: String,
    pub derivation_locator: String,
    pub amount: RawAssetAmount,
    pub confirmations: u32,
}

/// Engine Trait IV: On-Chain State Balance, UTXO/Object Set & Relayer Resource Reader Engine.
#[async_trait]
pub trait StateBalanceReaderEngine: SovereignEngine {
    fn chain_id(&self) -> ChainId;

    /// Queries the exact raw balance of `(address, asset)` at the latest finalized block.
    async fn get_finalized_balance(
        &self,
        address: &str,
        asset: &PhysicalAssetId,
    ) -> Result<RawAssetAmount, SovereignError>;

    /// Multicall / pipelined batch balance query across N wallet addresses and assets.
    async fn get_balances_batch(
        &self,
        queries: &[(String, PhysicalAssetId)],
    ) -> Result<Vec<RawAssetAmount>, SovereignError>;

    /// Lists all spendable UTXOs (Bitcoin) or discrete Coin<T> objects (Sui) for batch consolidation.
    async fn list_spendable_discrete_outputs(
        &self,
        addresses_with_locators: &[(String, String)],
        asset: &PhysicalAssetId,
        min_confirmations: u32,
    ) -> Result<Vec<SpendableDiscreteOutput>, SovereignError>;

    /// Inspects the Tier 1 Gas Station Relayer's resources.
    async fn inspect_gas_relayer_health(
        &self,
        relayer_address: &str,
        min_required_gas: RawAssetAmount,
    ) -> Result<RelayerResourceHealth, SovereignError>;

    /// Estimates the current optimal network fee rate (sat/vB, EIP-1559 wei, micro-lamports, Energy).
    async fn estimate_optimal_fee_rate(&self, target_blocks: u8) -> Result<u128, SovereignError>;
}
