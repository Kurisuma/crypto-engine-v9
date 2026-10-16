#![forbid(unsafe_code)]
#![deny(
    clippy::float_arithmetic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::unwrap_used,
    clippy::indexing_slicing
)]

use crate::verification::gates::FiveGateEvaluator;
use async_trait::async_trait;
use chrono::Utc;
use gw_enterprise_traits::{
    CheckoutDepositRoute, EngineHealthSnapshot, EngineIntegrityReport,
    InvoiceVerificationEngine, NormalizedTransferEvent, ObererOrmStorageEngine,
    SecurityZone, SovereignEngine, StateDetected, StateExpired, StatePartial, StatePending,
    StateVerifiedComplete, TamperEvidentOrmModel, TypestateInvoice,
    VerifiedTransitionOutcome, WatchOnlyDerivationEngine,
};
use gw_primitives::{
    MicroCreditsMathBridge, ObererU256Codec, PhysicalAssetId, RawAssetAmount, SovereignError,
};
use gw_storage_oberer::{
    InvoiceLifecycleStatus, InvoiceRow, OrmLogicalAsset,
};
use oberer_orm::conn::OrmConn;
use oberer_orm::{MicroCredits, OrmResultExt, Table, TableOps};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

fn current_unix_sec() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Sovereign 5-Gate Typestate Invoice Verification Engine.
pub struct SovereignInvoiceVerificationEngine {
    storage: Arc<dyn ObererOrmStorageEngine>,
    derivation_engine: Arc<dyn WatchOnlyDerivationEngine>,
    _bridge: MicroCreditsMathBridge,
}

impl SovereignInvoiceVerificationEngine {
    pub fn new(
        storage: Arc<dyn ObererOrmStorageEngine>,
        derivation_engine: Arc<dyn WatchOnlyDerivationEngine>,
        bridge: MicroCreditsMathBridge,
    ) -> Self {
        Self {
            storage,
            derivation_engine,
            _bridge: bridge,
        }
    }

    fn mem_conn(&self) -> OrmConn {
        self.storage.memory_conn()
    }

    fn to_orm_logical_asset(logical: gw_primitives::LogicalAsset) -> OrmLogicalAsset {
        match logical {
            gw_primitives::LogicalAsset::Btc => OrmLogicalAsset::Btc,
            gw_primitives::LogicalAsset::Eth => OrmLogicalAsset::Eth,
            gw_primitives::LogicalAsset::Bnb => OrmLogicalAsset::Bnb,
            gw_primitives::LogicalAsset::Plume => OrmLogicalAsset::Plume,
            gw_primitives::LogicalAsset::Sol => OrmLogicalAsset::Sol,
            gw_primitives::LogicalAsset::Trx => OrmLogicalAsset::Trx,
            gw_primitives::LogicalAsset::Ton => OrmLogicalAsset::Ton,
            gw_primitives::LogicalAsset::Sui => OrmLogicalAsset::Sui,
            gw_primitives::LogicalAsset::Usdt => OrmLogicalAsset::Usdt,
            gw_primitives::LogicalAsset::Usdc => OrmLogicalAsset::Usdc,
        }
    }
}

#[async_trait]
impl SovereignEngine for SovereignInvoiceVerificationEngine {
    fn engine_id(&self) -> &'static str {
        "SOVEREIGN_INVOICE_VERIFICATION_ENGINE"
    }

    fn security_zone(&self) -> SecurityZone {
        SecurityZone::ZoneAPublicWatchOnly
    }

    async fn initialize_engine(&self) -> Result<(), SovereignError> {
        Ok(())
    }

    async fn audit_cryptographic_integrity(&self) -> Result<EngineIntegrityReport, SovereignError> {
        let conn = self.mem_conn();
        let table = Table::<InvoiceRow>::new(conn);
        let rows = table.find_all().unenvelope().map_err(|e| {
            SovereignError::ObererOrmError {
                detail: format!("Failed to read invoices for audit: {e}"),
            }
        })?;

        let mut corrupted = 0_u64;
        let signer = self.storage.row_mac_signer();

        for row in &rows {
            if signer
                .verify_row_mac(
                    InvoiceRow::TABLE_LABEL,
                    &row.pk_string(),
                    row.version,
                    &row.canonical_security_bytes(),
                    row.row_mac_slice(),
                )
                .is_err()
            {
                corrupted = corrupted.saturating_add(1);
            }
        }

        Ok(EngineIntegrityReport {
            engine_id: self.engine_id().to_string(),
            security_zone: self.security_zone(),
            verified_records_or_cells: rows.len() as u64,
            detected_corruptions_or_tampers: corrupted,
            autonomous_self_heals: 0,
            hash_chain_head_hex: String::new(),
            audit_passed: corrupted == 0,
        })
    }

    async fn health_snapshot(&self) -> Result<EngineHealthSnapshot, SovereignError> {
        let conn = self.mem_conn();
        let table = Table::<InvoiceRow>::new(conn);
        let count = table.find_all().unenvelope().map(|r| r.len()).unwrap_or(0);

        Ok(EngineHealthSnapshot {
            engine_id: self.engine_id().to_string(),
            security_zone: self.security_zone(),
            is_healthy: true,
            active_workers_or_shards: 1,
            total_operations_processed: count as u64,
            p99_operation_latency_us: 15,
            memory_allocated_bytes: count.saturating_mul(256),
        })
    }

    async fn shutdown_gracefully(&self) -> Result<(), SovereignError> {
        Ok(())
    }
}

#[async_trait]
impl InvoiceVerificationEngine for SovereignInvoiceVerificationEngine {
    async fn create_and_persist_locked_invoice(
        &self,
        customer_id: Option<Uuid>,
        asset: PhysicalAssetId,
        fiat_amount_mc: MicroCredits,
        cex_mid_price_mc: MicroCredits,
        onchain_oracle_price_mc: MicroCredits,
        tolerance_bps: i64,
        quote_ttl_seconds: i64,
    ) -> Result<TypestateInvoice<StatePending>, SovereignError> {
        // Dual-oracle circuit breaker verification
        let locked_price = MicroCreditsMathBridge::verify_dual_oracle_microcredits(
            cex_mid_price_mc,
            onchain_oracle_price_mc,
            50,
        )?;

        // Exact clean locked token calculation: (fiat * 10^decimals) / price with dust truncation
        let expected_amount_raw = MicroCreditsMathBridge::to_clean_locked_quote(
            fiat_amount_mc,
            locked_price,
            asset.decimals,
        )?;

        let now = current_unix_sec();
        let expires_at_unix = now.saturating_add(quote_ttl_seconds);

        // Derive watch-only checkout route
        let route = self.derivation_engine
            .derive_checkout_route(now, customer_id, &asset, false)
            .await?;

        let conn = self.mem_conn();
        let table = Table::<InvoiceRow>::new(conn);

        let mut row = InvoiceRow {
            id: 0,
            customer_id_uuid: customer_id.map(|u| u.to_string()),
            pool_id: None,
            chain_id: asset.chain_id.as_i64(),
            logical_asset: Self::to_orm_logical_asset(asset.logical_asset),
            token_contract: asset.token_contract.clone(),
            decimals: asset.decimals as i64,
            deposit_address: route.deposit_address.clone(),
            routing_reference: route.routing_reference.clone(),
            derivation_locator: route.derivation_locator.clone(),
            fiat_amount_usd: fiat_amount_mc,
            locked_price_usd: locked_price,
            expected_amount_blob: expected_amount_raw.to_oberer_blob_vec(),
            expected_amount_dec78: expected_amount_raw.to_oberer_dec78(),
            received_amount_blob: vec![0u8; 32],
            received_amount_dec78: format!("{:0>78}", "0"),
            tolerance_bps,
            status: InvoiceLifecycleStatus::Pending,
            detected_at_block: None,
            current_confirmations: 0,
            target_confirmations: asset.target_confirmations as i64,
            expires_at_unix,
            settled_tx_hash: None,
            fulfillment_idempotency_key: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            version: 1,
            deleted_at: None,
            row_mac: Vec::new(),
        };

        // Seal Row MAC
        let mac = self.storage.row_mac_signer().compute_row_mac(
            InvoiceRow::TABLE_LABEL,
            &row.pk_string(),
            1,
            &row.canonical_security_bytes(),
        );
        row.assign_row_mac(mac);

        let invoice_id = table.insert(&row).unenvelope().map_err(|e| {
            SovereignError::ObererOrmError {
                detail: format!("Failed to persist pending invoice: {e}"),
            }
        })?;

        Ok(TypestateInvoice {
            invoice_id,
            customer_id,
            asset,
            route,
            fiat_amount_mc,
            locked_price_mc: locked_price,
            expected_amount_raw,
            cumulative_received_raw: RawAssetAmount::new(alloy_primitives::U256::ZERO, expected_amount_raw.decimals()),
            tolerance_bps,
            expires_at_unix,
            version: 1,
            row_mac: row.row_mac,
            state: StatePending,
        })
    }

    async fn evaluate_pending_invoice(
        &self,
        invoice: TypestateInvoice<StatePending>,
        event: &NormalizedTransferEvent,
        parent_hash_continuity_ok: bool,
    ) -> Result<VerifiedTransitionOutcome, SovereignError> {
        // Gate 1: Execution success
        FiveGateEvaluator::evaluate_gate1(event)?;

        // Gate 2: Destination & token contract match
        FiveGateEvaluator::evaluate_gate2(&invoice, event)?;

        // Gate 4: Expiration check
        if !FiveGateEvaluator::evaluate_gate4(invoice.expires_at_unix, event.block_header.timestamp_unix) {
            let expired_invoice = TypestateInvoice {
                invoice_id: invoice.invoice_id,
                customer_id: invoice.customer_id,
                asset: invoice.asset.clone(),
                route: invoice.route.clone(),
                fiat_amount_mc: invoice.fiat_amount_mc,
                locked_price_mc: invoice.locked_price_mc,
                expected_amount_raw: invoice.expected_amount_raw,
                cumulative_received_raw: event.net_credited_amount,
                tolerance_bps: invoice.tolerance_bps,
                expires_at_unix: invoice.expires_at_unix,
                version: invoice.version.saturating_add(1),
                row_mac: Vec::new(),
                state: StateExpired {
                    expired_at_unix: event.block_header.timestamp_unix,
                    late_received_raw: event.net_credited_amount,
                },
            };
            return Ok(VerifiedTransitionOutcome::MovedToExpired(expired_invoice));
        }

        // Gate 3: Amount calculation
        let (new_cumulative, is_complete, is_overpaid, surplus_or_shortfall) =
            FiveGateEvaluator::evaluate_gate3(
                &invoice.expected_amount_raw,
                &invoice.cumulative_received_raw,
                &event.net_credited_amount,
                invoice.tolerance_bps,
            )?;

        if !is_complete {
            let partial_invoice = TypestateInvoice {
                invoice_id: invoice.invoice_id,
                customer_id: invoice.customer_id,
                asset: invoice.asset.clone(),
                route: invoice.route.clone(),
                fiat_amount_mc: invoice.fiat_amount_mc,
                locked_price_mc: invoice.locked_price_mc,
                expected_amount_raw: invoice.expected_amount_raw,
                cumulative_received_raw: new_cumulative,
                tolerance_bps: invoice.tolerance_bps,
                expires_at_unix: invoice.expires_at_unix,
                version: invoice.version.saturating_add(1),
                row_mac: Vec::new(),
                state: StatePartial {
                    shortfall_raw: surplus_or_shortfall,
                    received_tx_hashes: vec![event.tx_hash.clone()],
                    extended_grace_expires_at_unix: invoice.expires_at_unix.saturating_add(3600),
                },
            };
            return Ok(VerifiedTransitionOutcome::MovedToPartial(partial_invoice));
        }

        // Gate 5: Parent hash continuity and confirmations check
        if !parent_hash_continuity_ok || event.confirmations < invoice.asset.target_confirmations {
            let detected_invoice = TypestateInvoice {
                invoice_id: invoice.invoice_id,
                customer_id: invoice.customer_id,
                asset: invoice.asset.clone(),
                route: invoice.route.clone(),
                fiat_amount_mc: invoice.fiat_amount_mc,
                locked_price_mc: invoice.locked_price_mc,
                expected_amount_raw: invoice.expected_amount_raw,
                cumulative_received_raw: new_cumulative,
                tolerance_bps: invoice.tolerance_bps,
                expires_at_unix: invoice.expires_at_unix,
                version: invoice.version.saturating_add(1),
                row_mac: Vec::new(),
                state: StateDetected {
                    current_confirmations: event.confirmations,
                    target_confirmations: invoice.asset.target_confirmations,
                    candidate_tx_hash: event.tx_hash.clone(),
                    detected_at_block: event.block_header.block_number,
                },
            };
            return Ok(VerifiedTransitionOutcome::RemainsDetected(detected_invoice));
        }

        // All 5 Gates passed: verified complete!
        let token = format!("FULFILL_{}_{}", invoice.invoice_id, event.tx_hash);
        let complete_invoice = TypestateInvoice {
            invoice_id: invoice.invoice_id,
            customer_id: invoice.customer_id,
            asset: invoice.asset.clone(),
            route: invoice.route.clone(),
            fiat_amount_mc: invoice.fiat_amount_mc,
            locked_price_mc: invoice.locked_price_mc,
            expected_amount_raw: invoice.expected_amount_raw,
            cumulative_received_raw: new_cumulative,
            tolerance_bps: invoice.tolerance_bps,
            expires_at_unix: invoice.expires_at_unix,
            version: invoice.version.saturating_add(1),
            row_mac: Vec::new(),
            state: StateVerifiedComplete {
                settlement_tx_hashes: vec![event.tx_hash.clone()],
                settled_at_unix: event.block_header.timestamp_unix,
                is_overpaid,
                surplus_raw: surplus_or_shortfall,
                fulfillment_idempotency_token: token,
            },
        };

        Ok(VerifiedTransitionOutcome::MovedToVerifiedComplete(complete_invoice))
    }

    async fn advance_detected_invoice(
        &self,
        invoice: TypestateInvoice<StateDetected>,
        updated_confirmations: u32,
        parent_hash_continuity_ok: bool,
    ) -> Result<VerifiedTransitionOutcome, SovereignError> {
        if !parent_hash_continuity_ok {
            let height_u64 = u64::try_from(invoice.state.detected_at_block.max(0)).unwrap_or(0);
            let lca_u64 = u64::try_from(invoice.state.detected_at_block.saturating_sub(1).max(0)).unwrap_or(0);
            return Err(SovereignError::ConsensusReorgDetected {
                height: height_u64,
                lca_height: lca_u64,
            });
        }

        if updated_confirmations >= invoice.state.target_confirmations {
            let token = format!("FULFILL_{}_{}", invoice.invoice_id, invoice.state.candidate_tx_hash);
            let complete_invoice = TypestateInvoice {
                invoice_id: invoice.invoice_id,
                customer_id: invoice.customer_id,
                asset: invoice.asset.clone(),
                route: invoice.route.clone(),
                fiat_amount_mc: invoice.fiat_amount_mc,
                locked_price_mc: invoice.locked_price_mc,
                expected_amount_raw: invoice.expected_amount_raw,
                cumulative_received_raw: invoice.cumulative_received_raw,
                tolerance_bps: invoice.tolerance_bps,
                expires_at_unix: invoice.expires_at_unix,
                version: invoice.version.saturating_add(1),
                row_mac: Vec::new(),
                state: StateVerifiedComplete {
                    settlement_tx_hashes: vec![invoice.state.candidate_tx_hash.clone()],
                    settled_at_unix: current_unix_sec(),
                    is_overpaid: invoice.cumulative_received_raw.raw() > invoice.expected_amount_raw.raw(),
                    surplus_raw: RawAssetAmount::new(
                        invoice.cumulative_received_raw.raw().saturating_sub(invoice.expected_amount_raw.raw()),
                        invoice.asset.decimals,
                    ),
                    fulfillment_idempotency_token: token,
                },
            };
            Ok(VerifiedTransitionOutcome::MovedToVerifiedComplete(complete_invoice))
        } else {
            let updated_detected = TypestateInvoice {
                invoice_id: invoice.invoice_id,
                customer_id: invoice.customer_id,
                asset: invoice.asset.clone(),
                route: invoice.route.clone(),
                fiat_amount_mc: invoice.fiat_amount_mc,
                locked_price_mc: invoice.locked_price_mc,
                expected_amount_raw: invoice.expected_amount_raw,
                cumulative_received_raw: invoice.cumulative_received_raw,
                tolerance_bps: invoice.tolerance_bps,
                expires_at_unix: invoice.expires_at_unix,
                version: invoice.version,
                row_mac: invoice.row_mac.clone(),
                state: StateDetected {
                    current_confirmations: updated_confirmations,
                    target_confirmations: invoice.state.target_confirmations,
                    candidate_tx_hash: invoice.state.candidate_tx_hash.clone(),
                    detected_at_block: invoice.state.detected_at_block,
                },
            };
            Ok(VerifiedTransitionOutcome::RemainsDetected(updated_detected))
        }
    }

    async fn evaluate_partial_topup(
        &self,
        invoice: TypestateInvoice<StatePartial>,
        topup_event: &NormalizedTransferEvent,
        parent_hash_continuity_ok: bool,
    ) -> Result<VerifiedTransitionOutcome, SovereignError> {
        FiveGateEvaluator::evaluate_gate1(topup_event)?;

        let (new_cumulative, is_complete, is_overpaid, surplus_or_shortfall) =
            FiveGateEvaluator::evaluate_gate3(
                &invoice.expected_amount_raw,
                &invoice.cumulative_received_raw,
                &topup_event.net_credited_amount,
                invoice.tolerance_bps,
            )?;

        if is_complete && parent_hash_continuity_ok && topup_event.confirmations >= invoice.asset.target_confirmations {
            let mut all_txs = invoice.state.received_tx_hashes.clone();
            all_txs.push(topup_event.tx_hash.clone());
            let token = format!("FULFILL_{}_{}", invoice.invoice_id, topup_event.tx_hash);
            let complete = TypestateInvoice {
                invoice_id: invoice.invoice_id,
                customer_id: invoice.customer_id,
                asset: invoice.asset.clone(),
                route: invoice.route.clone(),
                fiat_amount_mc: invoice.fiat_amount_mc,
                locked_price_mc: invoice.locked_price_mc,
                expected_amount_raw: invoice.expected_amount_raw,
                cumulative_received_raw: new_cumulative,
                tolerance_bps: invoice.tolerance_bps,
                expires_at_unix: invoice.expires_at_unix,
                version: invoice.version.saturating_add(1),
                row_mac: Vec::new(),
                state: StateVerifiedComplete {
                    settlement_tx_hashes: all_txs,
                    settled_at_unix: topup_event.block_header.timestamp_unix,
                    is_overpaid,
                    surplus_raw: surplus_or_shortfall,
                    fulfillment_idempotency_token: token,
                },
            };
            Ok(VerifiedTransitionOutcome::MovedToVerifiedComplete(complete))
        } else {
            let mut all_txs = invoice.state.received_tx_hashes.clone();
            all_txs.push(topup_event.tx_hash.clone());
            let partial = TypestateInvoice {
                invoice_id: invoice.invoice_id,
                customer_id: invoice.customer_id,
                asset: invoice.asset.clone(),
                route: invoice.route.clone(),
                fiat_amount_mc: invoice.fiat_amount_mc,
                locked_price_mc: invoice.locked_price_mc,
                expected_amount_raw: invoice.expected_amount_raw,
                cumulative_received_raw: new_cumulative,
                tolerance_bps: invoice.tolerance_bps,
                expires_at_unix: invoice.expires_at_unix,
                version: invoice.version.saturating_add(1),
                row_mac: Vec::new(),
                state: StatePartial {
                    shortfall_raw: surplus_or_shortfall,
                    received_tx_hashes: all_txs,
                    extended_grace_expires_at_unix: invoice.state.extended_grace_expires_at_unix,
                },
            };
            Ok(VerifiedTransitionOutcome::MovedToPartial(partial))
        }
    }

    async fn rollback_reorged_invoices_above_lca(
        &self,
        lca_height: i64,
    ) -> Result<Vec<TypestateInvoice<StatePending>>, SovereignError> {
        let conn = self.mem_conn();
        let table = Table::<InvoiceRow>::new(conn);
        let all_invoices = table.find_all().unenvelope().map_err(|e| {
            SovereignError::ObererOrmError {
                detail: format!("Failed to query invoices for reorg rollback: {e}"),
            }
        })?;

        let mut rolled_back = Vec::new();

        for inv in all_invoices {
            if inv.status == InvoiceLifecycleStatus::Detected {
                if let Some(detected_block) = inv.detected_at_block {
                    if detected_block > lca_height {
                        let mut updated = inv.clone();
                        updated.status = InvoiceLifecycleStatus::Pending;
                        updated.detected_at_block = None;
                        updated.current_confirmations = 0;
                        updated.received_amount_blob = vec![0u8; 32];
                        updated.received_amount_dec78 = format!("{:0>78}", "0");

                        let next_version = inv.version.saturating_add(1);
                        let mac = self.storage.row_mac_signer().compute_row_mac(
                            InvoiceRow::TABLE_LABEL,
                            &inv.pk_string(),
                            next_version,
                            &updated.canonical_security_bytes(),
                        );
                        updated.assign_row_mac(mac);

                        table.update_with_version(inv.id, inv.version, &updated).unenvelope().map_err(|e| {
                            SovereignError::ObererOrmError {
                                detail: format!("Failed to rollback invoice {}: {e}", inv.id),
                            }
                        })?;

                        let decimals = u8::try_from(inv.decimals.max(0)).unwrap_or(18);
                        let target_conf = u32::try_from(inv.target_confirmations.max(0)).unwrap_or(1);
                        let asset = PhysicalAssetId {
                            chain_id: gw_primitives::ChainId::EthereumMainnet,
                            logical_asset: gw_primitives::LogicalAsset::Usdt,
                            token_contract: inv.token_contract.clone(),
                            decimals,
                            keep_one_unit_dust: false,
                            target_confirmations: target_conf,
                        };
                        let route = CheckoutDepositRoute {
                            chain_id: asset.chain_id,
                            deposit_address: inv.deposit_address.clone(),
                            routing_reference: inv.routing_reference.clone(),
                            derivation_locator: inv.derivation_locator.clone(),
                            counterfactual_commitment_hash: String::new(),
                        };

                        let exp_u256 = ObererU256Codec::from_be_blob_slice(&inv.expected_amount_blob)?;
                        let exp_raw = RawAssetAmount::new(exp_u256, decimals);
                        rolled_back.push(TypestateInvoice {
                            invoice_id: inv.id,
                            customer_id: None,
                            asset,
                            route,
                            fiat_amount_mc: inv.fiat_amount_usd,
                            locked_price_mc: inv.locked_price_usd,
                            expected_amount_raw: exp_raw,
                            cumulative_received_raw: RawAssetAmount::new(alloy_primitives::U256::ZERO, decimals),
                            tolerance_bps: inv.tolerance_bps,
                            expires_at_unix: inv.expires_at_unix,
                            version: next_version,
                            row_mac: updated.row_mac,
                            state: StatePending,
                        });
                    }
                }
            }
        }

        Ok(rolled_back)
    }
}
