#![forbid(unsafe_code)]

use crate::codec::ObererU256Codec;
use crate::error::SovereignError;
use alloy_primitives::U256;
use serde::{Deserialize, Serialize};

/// Checked fixed-scale raw on-chain integer amount.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RawAssetAmount {
    raw: U256,
    decimals: u8,
}

impl RawAssetAmount {
    pub const fn new(raw: U256, decimals: u8) -> Self {
        Self { raw, decimals }
    }

    pub const fn zero(decimals: u8) -> Self {
        Self {
            raw: U256::ZERO,
            decimals,
        }
    }

    pub const fn raw(&self) -> U256 {
        self.raw
    }

    pub const fn decimals(&self) -> u8 {
        self.decimals
    }

    pub fn is_zero(&self) -> bool {
        self.raw.is_zero()
    }

    pub fn to_oberer_blob_vec(self) -> Vec<u8> {
        ObererU256Codec::to_be_blob_vec(self.raw)
    }

    pub fn to_oberer_dec78(self) -> String {
        ObererU256Codec::to_padded_dec78(self.raw)
    }

    pub fn checked_add(self, rhs: Self) -> Result<Self, SovereignError> {
        if self.decimals != rhs.decimals {
            return Err(SovereignError::DecimalMismatch {
                expected: self.decimals,
                actual: rhs.decimals,
            });
        }
        let sum = self
            .raw
            .checked_add(rhs.raw)
            .ok_or(SovereignError::Overflow {
                context: "RawAssetAmount::checked_add",
            })?;
        Ok(Self::new(sum, self.decimals))
    }

    pub fn checked_sub(self, rhs: Self) -> Result<Self, SovereignError> {
        if self.decimals != rhs.decimals {
            return Err(SovereignError::DecimalMismatch {
                expected: self.decimals,
                actual: rhs.decimals,
            });
        }
        let diff = self
            .raw
            .checked_sub(rhs.raw)
            .ok_or(SovereignError::Underflow {
                context: "RawAssetAmount::checked_sub",
            })?;
        Ok(Self::new(diff, self.decimals))
    }

    /// Maps any physical decimal scale d \in [0, 36] into canonical 18-decimal units \mathcal{N}_{18}(r, d).
    pub fn to_canonical_18(self) -> Result<U256, SovereignError> {
        if self.decimals == 18 {
            return Ok(self.raw);
        }
        if self.decimals < 18 {
            let exp = 18_u8
                .checked_sub(self.decimals)
                .ok_or(SovereignError::Underflow {
                    context: "to_canonical_18::exp_lt",
                })?;
            let factor = U256::from(10_u64)
                .checked_pow(U256::from(exp))
                .ok_or(SovereignError::Overflow {
                    context: "to_canonical_18::pow_lt",
                })?;
            self.raw
                .checked_mul(factor)
                .ok_or(SovereignError::Overflow {
                    context: "to_canonical_18::mul",
                })
        } else {
            let exp = self
                .decimals
                .checked_sub(18_u8)
                .ok_or(SovereignError::Underflow {
                    context: "to_canonical_18::exp_gt",
                })?;
            let divisor = U256::from(10_u64)
                .checked_pow(U256::from(exp))
                .ok_or(SovereignError::Overflow {
                    context: "to_canonical_18::pow_gt",
                })?;
            self.raw
                .checked_div(divisor)
                .ok_or(SovereignError::DivisionByZero)
        }
    }
}
