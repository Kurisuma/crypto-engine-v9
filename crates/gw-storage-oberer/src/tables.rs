#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic, clippy::unwrap_used)]

use crate::enums::*;
use chrono::{DateTime, Utc};
use oberer_orm::MicroCredits;
use oberer_orm_derive::OrmTable;
use serde::{Deserialize, Serialize};

/// 1. Whitelisted Network & Asset Registry (P0 Consensus Tier)
///
/// Zero-Raw-SQL: All checks, indexes, and constraints defined via `Schema<SupportedAssetRow>`.
#[derive(OrmTable, Debug, Clone, Serialize, Deserialize)]
#[orm(table_name = "supported_assets", schema_version = 1)]
pub struct SupportedAssetRow {
    #[orm(primary_key, auto_increment)]
    pub id: i64,
    pub chain_id: i64,
    #[orm(enum)]
    pub architecture: OrmChainArchitecture,
    #[orm(enum)]
    pub logical_asset: OrmLogicalAsset,
    #[orm(default = "'NATIVE'")]
    pub token_contract: String,
    pub decimals: i64,
    #[orm(micro_credits, non_negative)]
    pub min_sweep_usd: MicroCredits,
    pub target_confirmations: i64,
    pub keep_one_unit_dust: i64,
    pub requires_memo_tag: i64,
    pub is_enabled: i64,
    #[orm(version)]
    pub version: i64,
    pub row_mac: Vec<u8>,
}

/// 2. P0 Consensus Parent-Hash Sliding Window Ring Buffer (Reorg LCA Detection)
#[derive(OrmTable, Debug, Clone, Serialize, Deserialize)]
#[orm(table_name = "consensus_block_ring", schema_version = 1)]
pub struct ConsensusBlockHeaderRow {
    #[orm(primary_key, auto_increment)]
    pub id: i64,
    pub chain_id: i64,
    pub block_number: i64,
    pub block_hash: String,
    pub parent_block_hash: String,
    pub state_root: String,
    pub timestamp_unix: i64,
    pub is_finalized: i64,
    #[orm(version)]
    pub version: i64,
    pub row_mac: Vec<u8>,
}

/// 3. Rotating Guest Deposit Address Pool (P2 Pool Lease & Cooldown Tier)
#[derive(OrmTable, Debug, Clone, Serialize, Deserialize)]
#[orm(table_name = "deposit_address_pool", schema_version = 1)]
pub struct DepositAddressPoolRow {
    #[orm(primary_key, auto_increment)]
    pub id: i64,
    pub chain_id: i64,
    pub deposit_address: String,
    pub derivation_locator: String,
    pub counterfactual_hash: String,
    #[orm(enum)]
    pub status: PoolLeaseStatus,
    pub leased_until_unix: Option<i64>,
    pub cooldown_until_unix: Option<i64>,
    #[orm(version)]
    pub version: i64,
    pub row_mac: Vec<u8>,
}

/// 4. Typestate Invoices Ledger (P1 Active Watchlist Tier)
#[derive(OrmTable, Debug, Clone, Serialize, Deserialize)]
#[orm(table_name = "invoices", schema_version = 1)]
pub struct InvoiceRow {
    #[orm(primary_key, auto_increment)]
    pub id: i64,
    pub customer_id_uuid: Option<String>,
    pub pool_id: Option<i64>,
    pub chain_id: i64,
    #[orm(enum)]
    pub logical_asset: OrmLogicalAsset,
    pub token_contract: String,
    pub decimals: i64,
    pub deposit_address: String,
    pub routing_reference: Option<String>,
    pub derivation_locator: String,
    #[orm(micro_credits, non_negative)]
    pub fiat_amount_usd: MicroCredits,
    #[orm(micro_credits, non_negative)]
    pub locked_price_usd: MicroCredits,
    pub expected_amount_blob: Vec<u8>,
    pub expected_amount_dec78: String,
    pub received_amount_blob: Vec<u8>,
    pub received_amount_dec78: String,
    pub tolerance_bps: i64,
    #[orm(enum)]
    pub status: InvoiceLifecycleStatus,
    pub detected_at_block: Option<i64>,
    pub current_confirmations: i64,
    pub target_confirmations: i64,
    pub expires_at_unix: i64,
    pub settled_tx_hash: Option<String>,
    pub fulfillment_idempotency_key: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[orm(version)]
    pub version: i64,
    #[orm(soft_delete)]
    pub deleted_at: Option<String>,
    pub row_mac: Vec<u8>,
}

/// 5. Reactive Invoice State Transition Audit Trail
#[derive(OrmTable, Debug, Clone, Serialize, Deserialize)]
#[orm(table_name = "invoice_state_audit", schema_version = 1)]
pub struct InvoiceStateAuditRow {
    #[orm(primary_key, auto_increment)]
    pub id: i64,
    pub invoice_id: i64,
    #[orm(enum)]
    pub old_status: InvoiceLifecycleStatus,
    #[orm(enum)]
    pub new_status: InvoiceLifecycleStatus,
    pub old_version: i64,
    pub new_version: i64,
    pub settled_tx_hash: Option<String>,
    pub recorded_at: String,
}

/// 6. Unified Treasury Wallets
#[derive(OrmTable, Debug, Clone, Serialize, Deserialize)]
#[orm(table_name = "treasury_wallets", schema_version = 1)]
pub struct TreasuryWalletRow {
    #[orm(primary_key, auto_increment)]
    pub id: i64,
    pub wallet_uuid: String,
    pub wallet_name: String,
    pub chain_id: i64,
    #[orm(enum)]
    pub architecture: OrmChainArchitecture,
    pub address: String,
    #[orm(enum)]
    pub wallet_tier: OrmWalletTier,
    pub key_locator: String,
    pub is_whitelisted_dest: i64,
    pub is_active: i64,
    pub created_at: DateTime<Utc>,
    #[orm(version)]
    pub version: i64,
    #[orm(soft_delete)]
    pub deleted_at: Option<String>,
    pub row_mac: Vec<u8>,
}

/// 7. Multi-Network Wallet Asset Balances
#[derive(OrmTable, Debug, Clone, Serialize, Deserialize)]
#[orm(table_name = "asset_balances", schema_version = 1)]
pub struct AssetBalanceRow {
    #[orm(primary_key, auto_increment)]
    pub id: i64,
    pub wallet_id: i64,
    pub chain_id: i64,
    #[orm(enum)]
    pub logical_asset: OrmLogicalAsset,
    pub token_contract: String,
    pub decimals: i64,
    pub balance_raw_blob: Vec<u8>,
    pub balance_raw_dec78: String,
    pub balance_canonical_18_blob: Vec<u8>,
    pub balance_canonical_18_dec78: String,
    pub updated_at: DateTime<Utc>,
    #[orm(version)]
    pub version: i64,
    pub row_mac: Vec<u8>,
}

/// 8. FIFO Cost-Basis Tax Lots
#[derive(OrmTable, Debug, Clone, Serialize, Deserialize)]
#[orm(table_name = "cost_basis_lots", schema_version = 1)]
pub struct CostBasisLotRow {
    #[orm(primary_key, auto_increment)]
    pub id: i64,
    pub lot_uuid: String,
    pub source_invoice_id: i64,
    pub chain_id: i64,
    #[orm(enum)]
    pub logical_asset: OrmLogicalAsset,
    pub token_contract: String,
    pub decimals: i64,
    pub initial_amount_blob: Vec<u8>,
    pub initial_amount_dec78: String,
    pub remaining_amount_blob: Vec<u8>,
    pub remaining_amount_dec78: String,
    #[orm(micro_credits, non_negative)]
    pub acquisition_price_usd: MicroCredits,
    pub acquired_at: DateTime<Utc>,
    pub is_exhausted: i64,
    #[orm(version)]
    pub version: i64,
    pub row_mac: Vec<u8>,
}

/// 9. Hash-Chained Institutional Treasury Transfers
#[derive(OrmTable, Debug, Clone, Serialize, Deserialize)]
#[orm(table_name = "treasury_transfers", schema_version = 1)]
pub struct TreasuryTransferRow {
    #[orm(primary_key, auto_increment)]
    pub id: i64,
    pub transfer_uuid: String,
    pub sequence_no: i64,
    pub source_wallet_id: i64,
    pub destination_wallet_id: i64,
    pub chain_id: i64,
    #[orm(enum)]
    pub logical_asset: OrmLogicalAsset,
    pub token_contract: String,
    pub amount_raw_blob: Vec<u8>,
    pub amount_raw_dec78: String,
    pub max_gas_fee_blob: Vec<u8>,
    #[orm(micro_credits, non_negative)]
    pub fiat_value_usd: MicroCredits,
    pub unsigned_payload_sha256: String,
    pub tx_hash: Option<String>,
    #[orm(enum)]
    pub status: TransferGovernanceStatus,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub prev_chain_hash: Vec<u8>,
    pub entry_chain_hash: Vec<u8>,
    #[orm(version)]
    pub version: i64,
    pub row_mac: Vec<u8>,
}

/// 10. Hash-Chained Realized P&L Ledger
#[derive(OrmTable, Debug, Clone, Serialize, Deserialize)]
#[orm(table_name = "realized_pnl_ledger", schema_version = 1)]
pub struct RealizedPnlLedgerRow {
    #[orm(primary_key, auto_increment)]
    pub id: i64,
    pub settlement_uuid: String,
    pub sequence_no: i64,
    pub transfer_id: Option<i64>,
    #[orm(enum)]
    pub logical_asset: OrmLogicalAsset,
    pub chain_id: i64,
    #[orm(enum)]
    pub event_kind: PnlDisposalEventKind,
    pub disposed_amount_blob: Vec<u8>,
    pub disposed_amount_dec78: String,
    #[orm(micro_credits, non_negative)]
    pub cost_basis_usd: MicroCredits,
    #[orm(micro_credits, non_negative)]
    pub gross_proceeds_usd: MicroCredits,
    #[orm(micro_credits, non_negative)]
    pub network_fee_usd: MicroCredits,
    pub net_realized_pnl_micros: i64,
    pub recorded_at: DateTime<Utc>,
    pub prev_chain_hash: Vec<u8>,
    pub entry_chain_hash: Vec<u8>,
    #[orm(version)]
    pub version: i64,
    pub row_mac: Vec<u8>,
}

/// 11. Session-Scoped Temporary Block Ingestion Staging Table
#[derive(OrmTable, Debug, Clone, Serialize, Deserialize)]
#[orm(table_name = "temp_block_transfer_staging", temporary)]
pub struct TempBlockTransferStagingRow {
    #[orm(primary_key, auto_increment)]
    pub id: i64,
    pub chain_id: i64,
    pub block_number: i64,
    pub tx_hash: String,
    pub lookup_key: String,
    pub token_contract: String,
    pub net_amount_dec78: String,
}
