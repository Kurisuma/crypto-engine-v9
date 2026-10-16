#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic, clippy::unwrap_used)]

use crate::oberer_storage_engine::SovereignEngine;
use async_trait::async_trait;
use gw_primitives::{
    ChainArchitecture, ChainId, PhysicalAssetId, RawAssetAmount, SovereignError, WalletTier,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Deterministic unsigned transaction envelope persisted in storage and sent to Zone C.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnsignedTransactionEnvelope {
    pub envelope_id: Uuid,
    pub chain_id: ChainId,
    pub architecture: ChainArchitecture,
    pub source_tier: WalletTier,
    pub destination_tier: WalletTier,
    pub source_addresses: Vec<String>,
    pub expected_destination_address: String,
    pub physical_asset: PhysicalAssetId,
    pub net_transfer_amount: RawAssetAmount,
    pub max_network_fee_native: RawAssetAmount,
    pub nonce_or_sequence: u64,
    pub raw_unsigned_payload: Vec<u8>,
    pub payload_sha256_hex: String,
    pub signer_key_locators: Vec<String>,
}

/// Pre-broadcast local node dry-run simulation report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DryRunSimulationReport {
    pub simulation_succeeded: bool,
    pub exact_gas_or_energy_used: u64,
    pub verified_recipient_delta: RawAssetAmount,
    pub reclaimed_rent_or_dust_saved: u64,
    pub revert_reason: Option<String>,
}

/// Engine Trait V: Unsigned Sweep, Batch Consolidation, Fee-Bump & Broadcast Engine.
#[async_trait]
pub trait UnsignedSweepAssemblerEngine: SovereignEngine {
    fn chain_id(&self) -> ChainId;

    /// Assembles an unsigned Tier 0 -> Tier 2/3 customer deposit sweep envelope.
    async fn assemble_tier0_deposit_sweep(
        &self,
        deposit_locators: &[String],
        destination_vault_address: &str,
        destination_tier: WalletTier,
        asset: &PhysicalAssetId,
        relayer_address: &str,
    ) -> Result<UnsignedTransactionEnvelope, SovereignError>;

    /// Assembles an unsigned institutional treasury transfer.
    #[allow(clippy::too_many_arguments)]
    async fn assemble_treasury_transfer(
        &self,
        source_wallet_address: &str,
        source_key_locator: &str,
        destination_address: &str,
        source_tier: WalletTier,
        destination_tier: WalletTier,
        asset: &PhysicalAssetId,
        amount: RawAssetAmount,
    ) -> Result<UnsignedTransactionEnvelope, SovereignError>;

    /// Executes a deterministic dry-run simulation against the local native node.
    async fn simulate_unsigned_envelope(
        &self,
        envelope: &UnsignedTransactionEnvelope,
    ) -> Result<DryRunSimulationReport, SovereignError>;

    /// Constructs an RBF / EIP-1559 same-nonce gas bump or CPFP replacement envelope.
    async fn assemble_fee_bump_replacement(
        &self,
        stuck_envelope: &UnsignedTransactionEnvelope,
        fee_multiplier_bps: u32,
    ) -> Result<UnsignedTransactionEnvelope, SovereignError>;

    /// Broadcasts a Zone-C cryptographically signed transaction payload to the local native node.
    async fn broadcast_signed_transaction(
        &self,
        envelope_id: Uuid,
        signed_raw_bytes: &[u8],
    ) -> Result<String, SovereignError>;
}
