#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic, clippy::unwrap_used)]

use crate::oberer_storage_engine::SovereignEngine;
use async_trait::async_trait;
use gw_primitives::{ChainId, PhysicalAssetId, SovereignError};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Complete deterministic deposit routing descriptor returned to the checkout engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckoutDepositRoute {
    pub chain_id: ChainId,
    /// Primary receiving address (bc1q... on BTC, CREATE2 0x... on EVM, T... on Tron, Vault on TON/Solana).
    pub deposit_address: String,
    /// Optional secondary routing identifier (Solana Reference Pubkey or TON 8-Byte Crockford MemoTag).
    pub routing_reference: Option<String>,
    /// Exact deterministic derivation path (m/84'/0'/0'/0/i) or 32-byte CREATE2/PDA salt hex.
    pub derivation_locator: String,
    /// Cryptographic commitment attesting that the address is bound to Cold Storage.
    pub counterfactual_commitment_hash: String,
}

impl CheckoutDepositRoute {
    pub fn canonical_watchlist_key(&self) -> &str {
        self.routing_reference
            .as_deref()
            .unwrap_or(&self.deposit_address)
    }
}

/// Engine Trait II: Deterministic Watch-Only Address & Counterfactual Route Derivation Engine.
#[async_trait]
pub trait WatchOnlyDerivationEngine: SovereignEngine {
    fn chain_id(&self) -> ChainId;

    /// Derives a single deterministic checkout route in pure watch-only mode (Zero Private Keys).
    async fn derive_checkout_route(
        &self,
        invoice_id: i64,
        customer_id: Option<Uuid>,
        asset: &PhysicalAssetId,
        prefer_zero_sweep_reference: bool,
    ) -> Result<CheckoutDepositRoute, SovereignError>;

    /// Pre-generates a contiguous batch of deterministic routes and persists them into storage.
    async fn warm_address_pool_batch(
        &self,
        start_pool_index: u64,
        count: usize,
        asset: &PhysicalAssetId,
    ) -> Result<Vec<CheckoutDepositRoute>, SovereignError>;

    /// Cryptographically verifies that a candidate deposit_address and routing_reference match
    /// the deterministic derivation locator and immutable Cold Treasury bytecode commitment.
    fn verify_route_determinism(
        &self,
        route: &CheckoutDepositRoute,
        expected_cold_vault_address: &str,
    ) -> Result<bool, SovereignError>;

    /// Validates external user-supplied or exchange-whitelisted destination address syntax & checksum.
    fn validate_address_syntax_and_checksum(
        &self,
        raw_address: &str,
    ) -> Result<String, SovereignError>;
}
