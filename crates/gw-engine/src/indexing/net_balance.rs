#![forbid(unsafe_code)]
#![deny(
    clippy::float_arithmetic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::unwrap_used,
    clippy::indexing_slicing
)]

use gw_primitives::{RawAssetAmount, SovereignError};

/// Calculates the exact post-execution net balance differential:
///
/// $$\Delta B_{\text{net}} = B_{\text{post}} \ominus B_{\text{pre}}$$
///
/// Defends directly against Fee-on-Transfer / Tax tokens and fake calldata emission attacks.
/// The credit awarded to the invoice MUST NOT trust the declared calldata amount;
/// it is strictly bounded by the actual net delta received by the recipient vault.
pub fn calculate_net_credited_amount(
    pre_balance: RawAssetAmount,
    post_balance: RawAssetAmount,
    declared_calldata_amount: RawAssetAmount,
) -> Result<RawAssetAmount, SovereignError> {
    if pre_balance.decimals() != post_balance.decimals()
        || pre_balance.decimals() != declared_calldata_amount.decimals()
    {
        return Err(SovereignError::DecimalMismatch {
            expected: pre_balance.decimals(),
            actual: post_balance.decimals(),
        });
    }

    // Vault balance must not have decreased
    if post_balance.raw() < pre_balance.raw() {
        return Ok(RawAssetAmount::new(alloy_primitives::U256::ZERO, pre_balance.decimals()));
    }

    // Actual physical delta received in custody account
    let actual_net_delta = post_balance.raw().saturating_sub(pre_balance.raw());

    // If actual delta is zero, zero credit
    if actual_net_delta == alloy_primitives::U256::ZERO {
        return Ok(RawAssetAmount::new(alloy_primitives::U256::ZERO, pre_balance.decimals()));
    }

    // In fee-on-transfer tokens, actual delta is strictly less than declared amount.
    // In fake event log attacks, declared amount is vastly greater than actual delta.
    // In both cases, the credited amount is the MINIMUM of actual delta and declared amount.
    let safe_credit = actual_net_delta.min(declared_calldata_amount.raw());

    Ok(RawAssetAmount::new(safe_credit, pre_balance.decimals()))
}
