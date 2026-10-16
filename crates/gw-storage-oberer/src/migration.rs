#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic, clippy::unwrap_used)]

use crate::tables::*;
use crate::views::ActiveWatchlistView;
use gw_primitives::SovereignError;
use oberer_orm::conn::OrmConn;
use oberer_orm::schema::trigger_dsl::{New, Old, TriggerExpr, TriggerInsert};
use oberer_orm::{OrmResultExt, Table, TableExpr, TableOps, TableTrigger, TableView};

/// Bootstraps all gateway and treasury relational schemas, indexes, views, and triggers
/// across a SQLite connection handle using exclusively oberer-orm's Zero-Raw-SQL Schema DSL.
pub fn bootstrap_sovereign_schema(conn: &OrmConn) -> Result<(), SovereignError> {
    // 1. SupportedAssetRow
    Table::<SupportedAssetRow>::new(conn.clone())
        .schema_query()
        .check_col_expr(
            SupportedAssetRow::decimals(),
            TableExpr::from(SupportedAssetRow::decimals())
                .gte(TableExpr::lit(0))
                .and(TableExpr::from(SupportedAssetRow::decimals()).lte(TableExpr::lit(36))),
        )
        .check_col_expr(
            SupportedAssetRow::target_confirmations(),
            TableExpr::from(SupportedAssetRow::target_confirmations()).gte(TableExpr::lit(1)),
        )
        .check_col_expr(
            SupportedAssetRow::row_mac(),
            TableExpr::blob_len(SupportedAssetRow::row_mac()).eq(TableExpr::lit(32)),
        )
        .index_cols_mixed(&[
            SupportedAssetRow::chain_id().name,
            SupportedAssetRow::token_contract().name,
        ])
        .unique()
        .name("idx_supported_assets_chain_contract")
        .migrate()
        .unenvelope()?;

    // 2. ConsensusBlockHeaderRow
    Table::<ConsensusBlockHeaderRow>::new(conn.clone())
        .schema_query()
        .check_col_expr(
            ConsensusBlockHeaderRow::block_number(),
            TableExpr::from(ConsensusBlockHeaderRow::block_number()).gte(TableExpr::lit(0)),
        )
        .check_col_expr(
            ConsensusBlockHeaderRow::row_mac(),
            TableExpr::blob_len(ConsensusBlockHeaderRow::row_mac()).eq(TableExpr::lit(32)),
        )
        .index_cols_mixed(&[
            ConsensusBlockHeaderRow::chain_id().name,
            ConsensusBlockHeaderRow::block_number().name,
        ])
        .unique()
        .name("idx_consensus_chain_height")
        .index_cols_mixed(&[
            ConsensusBlockHeaderRow::chain_id().name,
            ConsensusBlockHeaderRow::block_hash().name,
        ])
        .name("idx_consensus_chain_hash")
        .migrate()
        .unenvelope()?;

    // 3. DepositAddressPoolRow
    Table::<DepositAddressPoolRow>::new(conn.clone())
        .schema_query()
        .check_col_expr(
            DepositAddressPoolRow::row_mac(),
            TableExpr::blob_len(DepositAddressPoolRow::row_mac()).eq(TableExpr::lit(32)),
        )
        .index_cols_mixed(&[
            DepositAddressPoolRow::chain_id().name,
            DepositAddressPoolRow::deposit_address().name,
        ])
        .unique()
        .name("idx_pool_chain_addr")
        .index_cols_mixed(&[
            DepositAddressPoolRow::chain_id().name,
            DepositAddressPoolRow::status().name,
        ])
        .name("idx_pool_lease_lookup")
        .migrate()
        .unenvelope()?;

    // 4. TreasuryWalletRow
    Table::<TreasuryWalletRow>::new(conn.clone())
        .schema_query()
        .check_col_expr(
            TreasuryWalletRow::row_mac(),
            TableExpr::blob_len(TreasuryWalletRow::row_mac()).eq(TableExpr::lit(32)),
        )
        .index_cols_mixed(&[
            TreasuryWalletRow::chain_id().name,
            TreasuryWalletRow::address().name,
        ])
        .unique()
        .name("idx_treasury_wallet_chain_addr")
        .migrate()
        .unenvelope()?;

    // 5. InvoiceRow (references DepositAddressPoolRow)
    Table::<InvoiceRow>::new(conn.clone())
        .schema_query()
        .rebuild_if_needed(true)
        .foreign_key(InvoiceRow::pool_id(), DepositAddressPoolRow::id())
        .on_delete_restrict()
        .check_col_expr(
            InvoiceRow::tolerance_bps(),
            TableExpr::from(InvoiceRow::tolerance_bps())
                .gte(TableExpr::lit(0))
                .and(TableExpr::from(InvoiceRow::tolerance_bps()).lte(TableExpr::lit(500))),
        )
        .check_col_expr(
            InvoiceRow::expected_amount_blob(),
            TableExpr::blob_len(InvoiceRow::expected_amount_blob()).eq(TableExpr::lit(32)),
        )
        .check_col_expr(
            InvoiceRow::received_amount_blob(),
            TableExpr::blob_len(InvoiceRow::received_amount_blob()).eq(TableExpr::lit(32)),
        )
        .check_col_expr(
            InvoiceRow::row_mac(),
            TableExpr::blob_len(InvoiceRow::row_mac()).eq(TableExpr::lit(32)),
        )
        .index_cols_mixed(&[InvoiceRow::status().name, InvoiceRow::chain_id().name])
        .name("idx_invoices_active_hydration")
        .migrate()
        .unenvelope()?;

    // 6. InvoiceStateAuditRow
    Table::<InvoiceStateAuditRow>::new(conn.clone())
        .schema_query()
        .index_col(InvoiceStateAuditRow::invoice_id())
        .name("idx_invoice_audit_lookup")
        .migrate()
        .unenvelope()?;

    // 7. AssetBalanceRow (references TreasuryWalletRow)
    Table::<AssetBalanceRow>::new(conn.clone())
        .schema_query()
        .rebuild_if_needed(true)
        .foreign_key(AssetBalanceRow::wallet_id(), TreasuryWalletRow::id())
        .on_delete_cascade()
        .check_col_expr(
            AssetBalanceRow::balance_raw_blob(),
            TableExpr::blob_len(AssetBalanceRow::balance_raw_blob()).eq(TableExpr::lit(32)),
        )
        .check_col_expr(
            AssetBalanceRow::balance_canonical_18_blob(),
            TableExpr::blob_len(AssetBalanceRow::balance_canonical_18_blob()).eq(TableExpr::lit(32)),
        )
        .check_col_expr(
            AssetBalanceRow::row_mac(),
            TableExpr::blob_len(AssetBalanceRow::row_mac()).eq(TableExpr::lit(32)),
        )
        .index_cols_mixed(&[
            AssetBalanceRow::wallet_id().name,
            AssetBalanceRow::chain_id().name,
            AssetBalanceRow::token_contract().name,
        ])
        .unique()
        .name("idx_asset_balances_wallet_token")
        .migrate()
        .unenvelope()?;

    // 8. CostBasisLotRow (references InvoiceRow)
    Table::<CostBasisLotRow>::new(conn.clone())
        .schema_query()
        .rebuild_if_needed(true)
        .foreign_key(CostBasisLotRow::source_invoice_id(), InvoiceRow::id())
        .on_delete_restrict()
        .check_col_expr(
            CostBasisLotRow::initial_amount_blob(),
            TableExpr::blob_len(CostBasisLotRow::initial_amount_blob()).eq(TableExpr::lit(32)),
        )
        .check_col_expr(
            CostBasisLotRow::remaining_amount_blob(),
            TableExpr::blob_len(CostBasisLotRow::remaining_amount_blob()).eq(TableExpr::lit(32)),
        )
        .check_col_expr(
            CostBasisLotRow::row_mac(),
            TableExpr::blob_len(CostBasisLotRow::row_mac()).eq(TableExpr::lit(32)),
        )
        .index_cols_mixed(&[
            CostBasisLotRow::logical_asset().name,
            CostBasisLotRow::chain_id().name,
        ])
        .name("idx_fifo_open_lots")
        .migrate()
        .unenvelope()?;

    // 9. TreasuryTransferRow (references TreasuryWalletRow)
    Table::<TreasuryTransferRow>::new(conn.clone())
        .schema_query()
        .rebuild_if_needed(true)
        .foreign_key(TreasuryTransferRow::source_wallet_id(), TreasuryWalletRow::id())
        .on_delete_restrict()
        .check_col_expr(
            TreasuryTransferRow::amount_raw_blob(),
            TableExpr::blob_len(TreasuryTransferRow::amount_raw_blob()).eq(TableExpr::lit(32)),
        )
        .check_col_expr(
            TreasuryTransferRow::prev_chain_hash(),
            TableExpr::blob_len(TreasuryTransferRow::prev_chain_hash()).eq(TableExpr::lit(32)),
        )
        .check_col_expr(
            TreasuryTransferRow::entry_chain_hash(),
            TableExpr::blob_len(TreasuryTransferRow::entry_chain_hash()).eq(TableExpr::lit(32)),
        )
        .check_col_expr(
            TreasuryTransferRow::row_mac(),
            TableExpr::blob_len(TreasuryTransferRow::row_mac()).eq(TableExpr::lit(32)),
        )
        .index_col(TreasuryTransferRow::sequence_no())
        .unique()
        .name("idx_transfers_seq_no")
        .migrate()
        .unenvelope()?;

    // 10. RealizedPnlLedgerRow (references TreasuryTransferRow)
    Table::<RealizedPnlLedgerRow>::new(conn.clone())
        .schema_query()
        .rebuild_if_needed(true)
        .foreign_key(RealizedPnlLedgerRow::transfer_id(), TreasuryTransferRow::id())
        .on_delete_restrict()
        .check_col_expr(
            RealizedPnlLedgerRow::disposed_amount_blob(),
            TableExpr::blob_len(RealizedPnlLedgerRow::disposed_amount_blob()).eq(TableExpr::lit(32)),
        )
        .check_col_expr(
            RealizedPnlLedgerRow::prev_chain_hash(),
            TableExpr::blob_len(RealizedPnlLedgerRow::prev_chain_hash()).eq(TableExpr::lit(32)),
        )
        .check_col_expr(
            RealizedPnlLedgerRow::entry_chain_hash(),
            TableExpr::blob_len(RealizedPnlLedgerRow::entry_chain_hash()).eq(TableExpr::lit(32)),
        )
        .check_col_expr(
            RealizedPnlLedgerRow::row_mac(),
            TableExpr::blob_len(RealizedPnlLedgerRow::row_mac()).eq(TableExpr::lit(32)),
        )
        .index_col(RealizedPnlLedgerRow::sequence_no())
        .unique()
        .name("idx_pnl_seq_no")
        .migrate()
        .unenvelope()?;

    // 11. TempBlockTransferStagingRow
    Table::<TempBlockTransferStagingRow>::new(conn.clone())
        .migrate()
        .unenvelope()?;

    // 12. ActiveWatchlistView via TableView view_query
    TableView::<ActiveWatchlistView>::new(conn.clone())
        .view_query()
        .select::<ActiveWatchlistView>()
        .from::<InvoiceRow>()
        .select_col(InvoiceRow::id(), ActiveWatchlistView::id())
        .select_col(InvoiceRow::chain_id(), ActiveWatchlistView::chain_id())
        .select_col(InvoiceRow::deposit_address(), ActiveWatchlistView::deposit_address())
        .select_col(InvoiceRow::routing_reference(), ActiveWatchlistView::routing_reference())
        .select_col(InvoiceRow::token_contract(), ActiveWatchlistView::token_contract())
        .select_col(InvoiceRow::expected_amount_dec78(), ActiveWatchlistView::expected_amount_dec78())
        .select_col(InvoiceRow::expires_at_unix(), ActiveWatchlistView::expires_at_unix())
        .filter(InvoiceRow::deleted_at().is_null())
        .migrate()
        .unenvelope()?;

    // 13. Reactive audit trigger via TableTrigger trigger_query
    TableTrigger::<InvoiceRow>::new(conn.clone())
        .trigger_query()
        .name("trg_invoice_state_audit")
        .after()
        .update()
        .on_table::<InvoiceRow>()
        .body_insert(
            TriggerInsert::<InvoiceStateAuditRow>::into()
                .set(InvoiceStateAuditRow::invoice_id(), Old::<InvoiceRow>::col(InvoiceRow::id()))
                .set(InvoiceStateAuditRow::old_status(), Old::<InvoiceRow>::col(InvoiceRow::status()))
                .set(InvoiceStateAuditRow::new_status(), New::<InvoiceRow>::col(InvoiceRow::status()))
                .set(InvoiceStateAuditRow::old_version(), Old::<InvoiceRow>::col(InvoiceRow::version()))
                .set(InvoiceStateAuditRow::new_version(), New::<InvoiceRow>::col(InvoiceRow::version()))
                .set(InvoiceStateAuditRow::settled_tx_hash(), New::<InvoiceRow>::col(InvoiceRow::settled_tx_hash()))
                .set(InvoiceStateAuditRow::recorded_at(), TriggerExpr::lit("CURRENT_TIMESTAMP".to_string())),
        )
        .migrate()
        .unenvelope()?;

    Ok(())
}
