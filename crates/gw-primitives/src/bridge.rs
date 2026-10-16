#![forbid(unsafe_code)]

use crate::amount::RawAssetAmount;
use crate::error::SovereignError;
use alloy_primitives::U256;
use oberer_orm::MicroCredits;

/// Bridge helper connecting `oberer_orm::MicroCredits` ($1.00 = 1_000_000 microcredits)
/// with 256-bit RawAssetAmount calculations.
pub struct MicroCreditsMathBridge;

impl MicroCreditsMathBridge {
    /// Evaluates \delta_{bps}(P_cex, P_chain) using `oberer_orm::MicroCredits` and enforces the circuit breaker.
    pub fn verify_dual_oracle_microcredits(
        cex_mid_mc: MicroCredits,
        onchain_oracle_mc: MicroCredits,
        max_divergence_bps: u32,
    ) -> Result<MicroCredits, SovereignError> {
        let cex_val = cex_mid_mc.get();
        let chain_val = onchain_oracle_mc.get();
        if cex_val <= 0 || chain_val <= 0 {
            return Err(SovereignError::DivisionByZero);
        }
        let cex_u128 = u128::try_from(cex_val).map_err(|_| SovereignError::Overflow {
            context: "verify_dual_oracle_microcredits::cex_u128",
        })?;
        let chain_u128 = u128::try_from(chain_val).map_err(|_| SovereignError::Overflow {
            context: "verify_dual_oracle_microcredits::chain_u128",
        })?;
        let min_p = cex_u128.min(chain_u128);
        let abs_diff = cex_u128.abs_diff(chain_u128);
        let divergence_u128 = abs_diff
            .checked_mul(10_000)
            .ok_or(SovereignError::Overflow {
                context: "verify_dual_oracle_microcredits::mul",
            })?
            .checked_div(min_p)
            .ok_or(SovereignError::DivisionByZero)?;

        let divergence_bps = u32::try_from(divergence_u128).unwrap_or(u32::MAX);
        if divergence_bps > max_divergence_bps {
            return Err(SovereignError::OracleDivergenceCircuitBreaker {
                divergence_bps,
                max_bps: max_divergence_bps,
            });
        }
        Ok(MicroCredits::new(cex_val.min(chain_val)))
    }

    /// Converts an invoice fiat `MicroCredits` amount into clean locked raw token units \mathcal{T}_{clean}.
    pub fn to_clean_locked_quote(
        fiat_mc: MicroCredits,
        locked_price_mc: MicroCredits,
        decimals: u8,
    ) -> Result<RawAssetAmount, SovereignError> {
        let fiat_val = fiat_mc.get();
        let price_val = locked_price_mc.get();
        if fiat_val < 0 || price_val <= 0 {
            return Err(SovereignError::DivisionByZero);
        }
        let fiat_u64 = u64::try_from(fiat_val).map_err(|_| SovereignError::Underflow {
            context: "to_clean_locked_quote::fiat_negative",
        })?;
        let price_u64 = u64::try_from(price_val).map_err(|_| SovereignError::DivisionByZero)?;

        let scale = U256::from(10_u64)
            .checked_pow(U256::from(decimals))
            .ok_or(SovereignError::Overflow {
                context: "to_clean_locked_quote::pow",
            })?;
        let mut raw = U256::from(fiat_u64)
            .checked_mul(scale)
            .ok_or(SovereignError::Overflow {
                context: "to_clean_locked_quote::mul",
            })?
            .checked_div(U256::from(price_u64))
            .ok_or(SovereignError::DivisionByZero)?;

        if decimals == 18 {
            let mask = U256::from(1_000_000_000_000_u64);
            raw = raw
                .checked_div(mask)
                .ok_or(SovereignError::DivisionByZero)?
                .checked_mul(mask)
                .ok_or(SovereignError::Overflow {
                    context: "to_clean_locked_quote::mask",
                })?;
        }
        Ok(RawAssetAmount::new(raw, decimals))
    }

    /// Evaluates the USD `MicroCredits` valuation of a `RawAssetAmount` at `price_mc`.
    pub fn to_microcredits_valuation(
        amount: RawAssetAmount,
        price_mc: MicroCredits,
    ) -> Result<MicroCredits, SovereignError> {
        let price_val = price_mc.get();
        if price_val < 0 {
            return Err(SovereignError::Underflow {
                context: "to_microcredits_valuation::negative_price",
            });
        }
        let price_u64 = u64::try_from(price_val).map_err(|_| SovereignError::Underflow {
            context: "to_microcredits_valuation::price_negative",
        })?;

        let scale = U256::from(10_u64)
            .checked_pow(U256::from(amount.decimals()))
            .ok_or(SovereignError::Overflow {
                context: "to_microcredits_valuation::pow",
            })?;
        let val_u256 = amount
            .raw()
            .checked_mul(U256::from(price_u64))
            .ok_or(SovereignError::Overflow {
                context: "to_microcredits_valuation::mul",
            })?
            .checked_div(scale)
            .ok_or(SovereignError::DivisionByZero)?;

        let val_i64 = i64::try_from(val_u256).map_err(|_| SovereignError::Overflow {
            context: "to_microcredits_valuation::i64_cast",
        })?;
        Ok(MicroCredits::new(val_i64))
    }
}
