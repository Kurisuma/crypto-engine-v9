#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic, clippy::unwrap_used)]

use crate::derivation_engine_trait::CheckoutDepositRoute;
use crate::indexing_engine_trait::NormalizedTransferEvent;
use crate::oberer_storage_engine::SovereignEngine;
use async_trait::async_trait;
use gw_primitives::{PhysicalAssetId, RawAssetAmount, SovereignError};
use oberer_orm::MicroCredits;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Compile-Time Typestate Lifecycle Markers
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatePending;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateDetected {
    pub current_confirmations: u32,
    pub target_confirmations: u32,
    pub candidate_tx_hash: String,
    pub detected_at_block: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatePartial {
    pub shortfall_raw: RawAssetAmount,
    pub received_tx_hashes: Vec<String>,
    pub extended_grace_expires_at_unix: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateVerifiedComplete {
    pub settlement_tx_hashes: Vec<String>,
    pub settled_at_unix: i64,
    pub is_overpaid: bool,
    pub surplus_raw: RawAssetAmount,
    pub fulfillment_idempotency_token: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateExpired {
    pub expired_at_unix: i64,
    pub late_received_raw: RawAssetAmount,
}

/// Generic Typestate Invoice Container enforcing Axiom 6 at compile time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypestateInvoice<S> {
    pub invoice_id: i64,
    pub customer_id: Option<Uuid>,
    pub asset: PhysicalAssetId,
    pub route: CheckoutDepositRoute,
    pub fiat_amount_mc: MicroCredits,
    pub locked_price_mc: MicroCredits,
    pub expected_amount_raw: RawAssetAmount,
    pub cumulative_received_raw: RawAssetAmount,
    pub tolerance_bps: i64,
    pub expires_at_unix: i64,
    pub version: i64,
    pub row_mac: Vec<u8>,
    pub state: S,
}

/// Algebraic union of valid typestate transitions produced by the 5-Gate Engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerifiedTransitionOutcome {
    RemainsDetected(TypestateInvoice<StateDetected>),
    MovedToPartial(TypestateInvoice<StatePartial>),
    MovedToVerifiedComplete(TypestateInvoice<StateVerifiedComplete>),
    MovedToExpired(TypestateInvoice<StateExpired>),
}

/// Engine Trait VII: 5-Gate Typestate Invoice Verification Engine.
#[async_trait]
pub trait InvoiceVerificationEngine: SovereignEngine {
    /// Creates a new locked quote `TypestateInvoice<StatePending>`.
    #[allow(clippy::too_many_arguments)]
    async fn create_and_persist_locked_invoice(
        &self,
        customer_id: Option<Uuid>,
        asset: PhysicalAssetId,
        fiat_amount_mc: MicroCredits,
        cex_mid_price_mc: MicroCredits,
        onchain_oracle_price_mc: MicroCredits,
        tolerance_bps: i64,
        quote_ttl_seconds: i64,
    ) -> Result<TypestateInvoice<StatePending>, SovereignError>;

    /// Evaluates a `StatePending` invoice against an incoming `NormalizedTransferEvent` across all 5 Gates.
    async fn evaluate_pending_invoice(
        &self,
        invoice: TypestateInvoice<StatePending>,
        event: &NormalizedTransferEvent,
        parent_hash_continuity_ok: bool,
    ) -> Result<VerifiedTransitionOutcome, SovereignError>;

    /// Advances a `StateDetected` invoice as new blocks confirm the transaction toward finality.
    async fn advance_detected_invoice(
        &self,
        invoice: TypestateInvoice<StateDetected>,
        updated_confirmations: u32,
        parent_hash_continuity_ok: bool,
    ) -> Result<VerifiedTransitionOutcome, SovereignError>;

    /// Evaluates a top-up payment arriving for a `StatePartial` invoice within the extended grace window.
    async fn evaluate_partial_topup(
        &self,
        invoice: TypestateInvoice<StatePartial>,
        topup_event: &NormalizedTransferEvent,
        parent_hash_continuity_ok: bool,
    ) -> Result<VerifiedTransitionOutcome, SovereignError>;

    /// Atomically rolls back all `StateDetected` invoices with `detected_at_block > lca_height` back to `StatePending`.
    async fn rollback_reorged_invoices_above_lca(
        &self,
        lca_height: i64,
    ) -> Result<Vec<TypestateInvoice<StatePending>>, SovereignError>;
}
