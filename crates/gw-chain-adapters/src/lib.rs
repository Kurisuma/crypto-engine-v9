#![forbid(unsafe_code)]
#![deny(
    clippy::float_arithmetic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::unwrap_used,
    clippy::indexing_slicing
)]

pub mod adapter;
pub mod bip84;
pub mod create2;
pub mod pool;
pub mod solana;
pub mod sui;
pub mod ton;
pub mod sweep;
pub mod validator;

pub use adapter::MultiChainWatchOnlyAdapter;
pub use bip84::{validate_bip84_address, Bip84WatchOnlyDeriver};
pub use create2::{
    calculate_tron_1sun_dust_keeper_sweep, derive_evm_create2_address, derive_evm_create2_raw,
    derive_tron_create2_address, derive_tron_create2_raw, to_eip55_checksum, validate_evm_address,
    validate_tron_address,
};
pub use pool::DepositAddressPoolEngine;
pub use solana::{
    derive_ephemeral_reference, derive_solana_invoice_pda, find_program_address,
    is_ed25519_curve_point, validate_solana_address,
    SOLANA_RENT_EXEMPT_TOKEN_ACCOUNT_LAMPORTS,
};
pub use sui::{derive_sui_deposit_address, validate_sui_address};
pub use sweep::MultiChainSweepAssemblerEngine;
pub use ton::{
    ton_crc16, validate_ton_address, TonFeistelMemoCipher, CROCKFORD_BASE32_ALPHABET,
};
pub use validator::validate_chain_address;
