#![forbid(unsafe_code)]
#![deny(
    clippy::float_arithmetic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::unwrap_used,
    clippy::indexing_slicing
)]

use crate::bip84::validate_bip84_address;
use crate::create2::{validate_evm_address, validate_tron_address};
use crate::solana::validate_solana_address;
use crate::sui::validate_sui_address;
use crate::ton::validate_ton_address;
use gw_primitives::{ChainArchitecture, ChainId, SovereignError};

/// Unified multi-chain address syntax and checksum validator.
pub fn validate_chain_address(
    chain_id: ChainId,
    address: &str,
) -> Result<String, SovereignError> {
    let trimmed = address.trim();

    match chain_id.architecture() {
        ChainArchitecture::BitcoinUtxo => {
            let hrp = match chain_id {
                ChainId::BitcoinMainnet => "bc",
                _ => "bc",
            };
            validate_bip84_address(trimmed, hrp)?;
            Ok(trimmed.to_string())
        }
        ChainArchitecture::EvmAccount => {
            let _raw = validate_evm_address(trimmed)?;
            // Returns canonical EIP-55 representation
            Ok(crate::create2::to_eip55_checksum(&_raw))
        }
        ChainArchitecture::TvmAccount => {
            let _raw = validate_tron_address(trimmed)?;
            Ok(trimmed.to_string())
        }
        ChainArchitecture::SvmAccount => {
            let _raw = validate_solana_address(trimmed)?;
            Ok(trimmed.to_string())
        }
        ChainArchitecture::TonActor => {
            validate_ton_address(trimmed)?;
            Ok(trimmed.to_string())
        }
        ChainArchitecture::MoveObject => {
            let _raw = validate_sui_address(trimmed)?;
            Ok(trimmed.to_lowercase())
        }
    }
}
