#![forbid(unsafe_code)]
#![deny(
    clippy::float_arithmetic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::unwrap_used,
    clippy::indexing_slicing
)]

use gw_enterprise_traits::{
    NormalizedTransferEvent, StatePending, TypestateInvoice,
};
use gw_primitives::{RawAssetAmount, SovereignError};

/// 5-Gate Conjunction Evaluator:
///
/// $$G_{\text{total}}(e, I) = \bigwedge_{i=1}^5 G_i(e, I)$$
pub struct FiveGateEvaluator;

impl FiveGateEvaluator {
    /// Evaluates Gate 1: Transaction Execution Success
    pub fn evaluate_gate1(event: &NormalizedTransferEvent) -> Result<(), SovereignError> {
        if event.execution_succeeded {
            Ok(())
        } else {
            Err(SovereignError::GateVerificationFailed {
                gate: 1,
                reason: format!("Transaction {} reverted on-chain", event.tx_hash),
            })
        }
    }

    /// Evaluates Gate 2: Signed Asset & Destination Verification
    pub fn evaluate_gate2(
        invoice: &TypestateInvoice<StatePending>,
        event: &NormalizedTransferEvent,
    ) -> Result<(), SovereignError> {
        if event.chain_id != invoice.asset.chain_id {
            return Err(SovereignError::GateVerificationFailed {
                gate: 2,
                reason: format!(
                    "Chain mismatch: expected {:?}, got {:?}",
                    invoice.asset.chain_id, event.chain_id
                ),
            });
        }

        // Compare contract / native symbol (case-insensitive)
        let expected_token = invoice.asset.token_contract.to_lowercase();
        let actual_token = event.token_contract.to_lowercase();
        if expected_token != actual_token {
            return Err(SovereignError::GateVerificationFailed {
                gate: 2,
                reason: format!(
                    "Token contract mismatch: expected '{expected_token}', got '{actual_token}'"
                ),
            });
        }

        // Destination match: must match either deposit address or routing reference
        let expected_addr = invoice.route.deposit_address.to_lowercase();
        let actual_recip = event.recipient_address.to_lowercase();
        let actual_lookup = event.watchlist_lookup_key.to_lowercase();

        let addr_matched = actual_recip == expected_addr || actual_lookup == expected_addr;
        let memo_matched = invoice
            .route
            .routing_reference
            .as_ref()
            .map(|r| r.to_lowercase() == actual_lookup)
            .unwrap_or(false);

        if !addr_matched && !memo_matched {
            return Err(SovereignError::GateVerificationFailed {
                gate: 2,
                reason: format!("Recipient address '{}' does not match invoice route", event.recipient_address),
            });
        }

        Ok(())
    }

    /// Evaluates Gate 4: Expiration Window Monotonicity
    pub fn evaluate_gate4(
        invoice_expires_at_unix: i64,
        block_timestamp_unix: i64,
    ) -> bool {
        block_timestamp_unix <= invoice_expires_at_unix
    }

    /// Evaluates Gate 3: Tolerance-Bounded Amount Arithmetic
    /// Returns: (cumulative_new, is_complete, is_overpaid, surplus_or_shortfall)
    pub fn evaluate_gate3(
        expected: &RawAssetAmount,
        cumulative_prior: &RawAssetAmount,
        incoming_delta: &RawAssetAmount,
        tolerance_bps: i64,
    ) -> Result<(RawAssetAmount, bool, bool, RawAssetAmount), SovereignError> {
        if expected.decimals() != incoming_delta.decimals() {
            return Err(SovereignError::DecimalMismatch {
                expected: expected.decimals(),
                actual: incoming_delta.decimals(),
            });
        }

        let new_total_raw = cumulative_prior.raw().saturating_add(incoming_delta.raw());
        let new_cumulative = RawAssetAmount::new(new_total_raw, expected.decimals());

        let bps_u64 = u64::try_from(tolerance_bps.max(0)).unwrap_or(0);
        let bps_u256 = alloy_primitives::U256::from(bps_u64);
        let ten_thousand = alloy_primitives::U256::from(10_000);
        let factor = ten_thousand.saturating_sub(bps_u256);
        let min_threshold = (expected.raw().saturating_mul(factor)) / ten_thousand;

        if new_total_raw >= min_threshold {
            // Payment complete! Check if overpaid
            let is_overpaid = new_total_raw > expected.raw();
            let surplus = if is_overpaid {
                RawAssetAmount::new(new_total_raw.saturating_sub(expected.raw()), expected.decimals())
            } else {
                RawAssetAmount::new(alloy_primitives::U256::ZERO, expected.decimals())
            };
            Ok((new_cumulative, true, is_overpaid, surplus))
        } else {
            // Shortfall: moves to partial
            let shortfall = RawAssetAmount::new(
                expected.raw().saturating_sub(new_total_raw),
                expected.decimals(),
            );
            Ok((new_cumulative, false, false, shortfall))
        }
    }
}
