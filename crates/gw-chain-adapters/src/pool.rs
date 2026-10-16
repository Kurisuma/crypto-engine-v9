#![forbid(unsafe_code)]
#![deny(
    clippy::float_arithmetic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::unwrap_used,
    clippy::indexing_slicing
)]

use gw_enterprise_traits::{CheckoutDepositRoute, ObererOrmStorageEngine, TamperEvidentOrmModel};
use gw_primitives::{ChainId, PhysicalAssetId, SovereignError};
use gw_storage_oberer::{DepositAddressPoolRow, PoolLeaseStatus};
use oberer_orm::conn::OrmConn;
use oberer_orm::error::OrmError;
use oberer_orm::{OrmResultExt, Table, TableOps};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

fn current_unix_sec() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Deposit Address Pool Engine with CAS-governed atomic route leasing and 4-hour cooldown management.
pub struct DepositAddressPoolEngine {
    storage: Arc<dyn ObererOrmStorageEngine>,
}

impl DepositAddressPoolEngine {
    pub fn new(storage: Arc<dyn ObererOrmStorageEngine>) -> Self {
        Self { storage }
    }

    fn mem_conn(&self) -> OrmConn {
        self.storage.memory_conn()
    }

    /// Atomically leases an available deposit address for an invoice with CAS version checking.
    pub async fn lease_route(
        &self,
        chain_id: ChainId,
        _asset: &PhysicalAssetId,
        lease_duration_secs: i64,
    ) -> Result<CheckoutDepositRoute, SovereignError> {
        let conn = self.mem_conn();
        let table = Table::<DepositAddressPoolRow>::new(conn.clone());
        let now = current_unix_sec();
        let target_chain_id = chain_id.as_i64();

        // Fetch candidate addresses matching target chain
        let candidates = table
            .find_all()
            .unenvelope()
            .map_err(|e| SovereignError::ObererOrmError {
                detail: format!("Failed to query deposit pool: {e}"),
            })?;

        for candidate in candidates {
            if candidate.chain_id != target_chain_id {
                continue;
            }

            // Must be Available or expired Cooldown
            let can_lease = match candidate.status {
                PoolLeaseStatus::Available => true,
                PoolLeaseStatus::Cooldown => {
                    candidate.cooldown_until_unix.is_some_and(|cd| cd <= now)
                }
                PoolLeaseStatus::Leased => {
                    // Check if existing lease has expired
                    candidate.leased_until_unix.is_some_and(|lu| lu <= now)
                }
            };

            if can_lease {
                let mut updated = candidate.clone();
                updated.status = PoolLeaseStatus::Leased;
                updated.leased_until_unix = Some(now.saturating_add(lease_duration_secs));
                updated.cooldown_until_unix = None;

                // Re-seal row MAC for the updated version
                let next_version = candidate.version.saturating_add(1);
                let mac = self.storage.row_mac_signer().compute_row_mac(
                    DepositAddressPoolRow::TABLE_LABEL,
                    &candidate.pk_string(),
                    next_version,
                    &updated.canonical_security_bytes(),
                );
                updated.assign_row_mac(mac);

                // Atomic CAS update
                let update_res = table.update_with_version(candidate.id, candidate.version, &updated);
                match update_res.unenvelope() {
                    Ok(()) => {
                        return Ok(CheckoutDepositRoute {
                            chain_id,
                            deposit_address: candidate.deposit_address,
                            routing_reference: None,
                            derivation_locator: candidate.derivation_locator,
                            counterfactual_commitment_hash: candidate.counterfactual_hash,
                        });
                    }
                    Err(OrmError::OptimisticLockConflict { .. }) => {
                        // CAS race lost to concurrent worker, continue to next candidate
                        continue;
                    }
                    Err(e) => {
                        return Err(SovereignError::ObererOrmError {
                            detail: format!("Deposit pool CAS lease error: {e}"),
                        });
                    }
                }
            }
        }

        Err(SovereignError::AddressPoolExhausted { chain: chain_id })
    }

    /// Releases a leased address and enters a 4-hour cooldown period.
    pub async fn release_route_to_cooldown(
        &self,
        chain_id: ChainId,
        deposit_address: &str,
        cooldown_secs: i64,
    ) -> Result<bool, SovereignError> {
        let conn = self.mem_conn();
        let table = Table::<DepositAddressPoolRow>::new(conn.clone());
        let now = current_unix_sec();
        let target_chain_id = chain_id.as_i64();

        let all_rows = table
            .find_all()
            .unenvelope()
            .map_err(|e| SovereignError::ObererOrmError {
                detail: format!("Failed to find address for release: {e}"),
            })?;

        for candidate in all_rows {
            if candidate.chain_id == target_chain_id && candidate.deposit_address == deposit_address {
                let mut updated = candidate.clone();
                updated.status = PoolLeaseStatus::Cooldown;
                updated.leased_until_unix = None;
                updated.cooldown_until_unix = Some(now.saturating_add(cooldown_secs));

                let next_version = candidate.version.saturating_add(1);
                let mac = self.storage.row_mac_signer().compute_row_mac(
                    DepositAddressPoolRow::TABLE_LABEL,
                    &candidate.pk_string(),
                    next_version,
                    &updated.canonical_security_bytes(),
                );
                updated.assign_row_mac(mac);

                match table
                    .update_with_version(candidate.id, candidate.version, &updated)
                    .unenvelope()
                {
                    Ok(()) => return Ok(true),
                    Err(OrmError::OptimisticLockConflict { .. }) => return Ok(false),
                    Err(e) => {
                        return Err(SovereignError::ObererOrmError {
                            detail: format!("Failed to release address to cooldown: {e}"),
                        });
                    }
                }
            }
        }

        Ok(false)
    }

    /// Pre-populates the address pool with a batch of pre-generated routes.
    pub async fn seed_pool_batch(
        &self,
        routes: &[CheckoutDepositRoute],
    ) -> Result<usize, SovereignError> {
        let conn = self.mem_conn();
        let table = Table::<DepositAddressPoolRow>::new(conn.clone());
        let mut inserted_count: usize = 0;

        for route in routes {
            let mut row = DepositAddressPoolRow {
                id: 0,
                chain_id: route.chain_id.as_i64(),
                deposit_address: route.deposit_address.clone(),
                derivation_locator: route.derivation_locator.clone(),
                counterfactual_hash: route.counterfactual_commitment_hash.clone(),
                status: PoolLeaseStatus::Available,
                leased_until_unix: None,
                cooldown_until_unix: None,
                version: 1,
                row_mac: Vec::new(),
            };

            let mac = self.storage.row_mac_signer().compute_row_mac(
                DepositAddressPoolRow::TABLE_LABEL,
                &row.pk_string(),
                1,
                &row.canonical_security_bytes(),
            );
            row.assign_row_mac(mac);

            table.insert(&row).unenvelope().map_err(|e| {
                SovereignError::ObererOrmError {
                    detail: format!("Failed to insert address pool row: {e}"),
                }
            })?;
            inserted_count = inserted_count.saturating_add(1);
        }

        Ok(inserted_count)
    }
}

