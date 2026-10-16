#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic, clippy::unwrap_used)]

use crate::mac::TamperEvidentOrmModel;
use crate::tables::*;

impl TamperEvidentOrmModel for InvoiceRow {
    const TABLE_LABEL: &'static str = "invoices";

    fn pk_string(&self) -> String {
        self.id.to_string()
    }

    fn current_version(&self) -> i64 {
        self.version
    }

    fn canonical_security_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(self.deposit_address.as_bytes());
        buf.extend_from_slice(self.token_contract.as_bytes());
        buf.extend_from_slice(&self.fiat_amount_usd.get().to_le_bytes());
        buf.extend_from_slice(&self.expected_amount_blob);
        buf
    }

    fn row_mac_slice(&self) -> &[u8] {
        &self.row_mac
    }

    fn assign_row_mac(&mut self, mac: Vec<u8>) {
        self.row_mac = mac;
    }
}

impl TamperEvidentOrmModel for CostBasisLotRow {
    const TABLE_LABEL: &'static str = "cost_basis_lots";

    fn pk_string(&self) -> String {
        self.id.to_string()
    }

    fn current_version(&self) -> i64 {
        self.version
    }

    fn canonical_security_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(self.lot_uuid.as_bytes());
        buf.extend_from_slice(&self.initial_amount_blob);
        buf.extend_from_slice(&self.remaining_amount_blob);
        buf.extend_from_slice(&self.acquisition_price_usd.get().to_le_bytes());
        buf
    }

    fn row_mac_slice(&self) -> &[u8] {
        &self.row_mac
    }

    fn assign_row_mac(&mut self, mac: Vec<u8>) {
        self.row_mac = mac;
    }
}

impl TamperEvidentOrmModel for TreasuryWalletRow {
    const TABLE_LABEL: &'static str = "treasury_wallets";

    fn pk_string(&self) -> String {
        self.id.to_string()
    }

    fn current_version(&self) -> i64 {
        self.version
    }

    fn canonical_security_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(self.address.as_bytes());
        buf.extend_from_slice(&(self.chain_id).to_le_bytes());
        buf
    }

    fn row_mac_slice(&self) -> &[u8] {
        &self.row_mac
    }

    fn assign_row_mac(&mut self, mac: Vec<u8>) {
        self.row_mac = mac;
    }
}

impl TamperEvidentOrmModel for AssetBalanceRow {
    const TABLE_LABEL: &'static str = "asset_balances";

    fn pk_string(&self) -> String {
        self.id.to_string()
    }

    fn current_version(&self) -> i64 {
        self.version
    }

    fn canonical_security_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.wallet_id.to_le_bytes());
        buf.extend_from_slice(&self.balance_raw_blob);
        buf
    }

    fn row_mac_slice(&self) -> &[u8] {
        &self.row_mac
    }

    fn assign_row_mac(&mut self, mac: Vec<u8>) {
        self.row_mac = mac;
    }
}

impl TamperEvidentOrmModel for TreasuryTransferRow {
    const TABLE_LABEL: &'static str = "treasury_transfers";

    fn pk_string(&self) -> String {
        self.id.to_string()
    }

    fn current_version(&self) -> i64 {
        self.version
    }

    fn canonical_security_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.sequence_no.to_le_bytes());
        buf.extend_from_slice(&self.amount_raw_blob);
        buf.extend_from_slice(self.unsigned_payload_sha256.as_bytes());
        buf
    }

    fn row_mac_slice(&self) -> &[u8] {
        &self.row_mac
    }

    fn assign_row_mac(&mut self, mac: Vec<u8>) {
        self.row_mac = mac;
    }
}

impl TamperEvidentOrmModel for RealizedPnlLedgerRow {
    const TABLE_LABEL: &'static str = "realized_pnl_ledger";

    fn pk_string(&self) -> String {
        self.id.to_string()
    }

    fn current_version(&self) -> i64 {
        self.version
    }

    fn canonical_security_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.sequence_no.to_le_bytes());
        buf.extend_from_slice(&self.disposed_amount_blob);
        buf.extend_from_slice(&self.cost_basis_usd.get().to_le_bytes());
        buf.extend_from_slice(&self.gross_proceeds_usd.get().to_le_bytes());
        buf
    }

    fn row_mac_slice(&self) -> &[u8] {
        &self.row_mac
    }

    fn assign_row_mac(&mut self, mac: Vec<u8>) {
        self.row_mac = mac;
    }
}

impl TamperEvidentOrmModel for DepositAddressPoolRow {
    const TABLE_LABEL: &'static str = "deposit_address_pool";

    fn pk_string(&self) -> String {
        self.id.to_string()
    }

    fn current_version(&self) -> i64 {
        self.version
    }

    fn canonical_security_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.chain_id.to_le_bytes());
        buf.extend_from_slice(self.deposit_address.as_bytes());
        buf.extend_from_slice(self.derivation_locator.as_bytes());
        buf.extend_from_slice(self.counterfactual_hash.as_bytes());
        buf
    }

    fn row_mac_slice(&self) -> &[u8] {
        &self.row_mac
    }

    fn assign_row_mac(&mut self, mac: Vec<u8>) {
        self.row_mac = mac;
    }
}

impl TamperEvidentOrmModel for ConsensusBlockHeaderRow {
    const TABLE_LABEL: &'static str = "consensus_block_ring";

    fn pk_string(&self) -> String {
        self.id.to_string()
    }

    fn current_version(&self) -> i64 {
        self.version
    }

    fn canonical_security_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.chain_id.to_le_bytes());
        buf.extend_from_slice(&self.block_number.to_le_bytes());
        buf.extend_from_slice(self.block_hash.as_bytes());
        buf.extend_from_slice(self.parent_block_hash.as_bytes());
        buf
    }

    fn row_mac_slice(&self) -> &[u8] {
        &self.row_mac
    }

    fn assign_row_mac(&mut self, mac: Vec<u8>) {
        self.row_mac = mac;
    }
}
