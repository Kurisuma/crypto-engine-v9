#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic, clippy::unwrap_used)]

use oberer_orm_derive::OrmTable;
use serde::{Deserialize, Serialize};

/// Read-Only Database View model for P1 Watchlist hydration.
/// Zero-Raw-SQL: Migrated via `ViewSchema<ActiveWatchlistView>`.
#[derive(OrmTable, Debug, Clone, Serialize, Deserialize)]
#[orm(table_name = "v_active_watchlist_invoices", schema_version = 1)]
pub struct ActiveWatchlistView {
    #[orm(primary_key)]
    pub id: i64,
    pub chain_id: i64,
    pub deposit_address: String,
    pub routing_reference: Option<String>,
    pub token_contract: String,
    pub expected_amount_dec78: String,
    pub expires_at_unix: i64,
}
