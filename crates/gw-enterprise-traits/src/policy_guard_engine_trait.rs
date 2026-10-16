#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic, clippy::unwrap_used)]

use crate::oberer_storage_engine::SovereignEngine;
use crate::sweep_engine_trait::UnsignedTransactionEnvelope;
use async_trait::async_trait;
use gw_primitives::{ChainId, RawAssetAmount, SovereignError, WalletTier};
use serde::{Deserialize, Serialize};

/// Cryptographic M-of-N hardware approval signature submitted by an institutional custodian.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HardwareQuorumSignature {
    pub signer_id: String,
    pub signer_public_key: Vec<u8>,
    pub signature_bytes: Vec<u8>,
    pub signed_at_unix: i64,
}

/// Cryptographic proof emitted by the Stateless Pre-Sign Policy Guard Engine after parsing raw bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyVerificationReceipt {
    pub verified_payload_sha256: [u8; 32],
    pub parsed_destination_addresses: Vec<String>,
    pub parsed_net_output_amount: RawAssetAmount,
    pub parsed_network_fee: RawAssetAmount,
    pub verified_quorum_signers: usize,
}

/// Engine Trait VIII: Zone C Stateless Pre-Sign Bytecode, PSBT & AST Policy Guard Engine.
#[async_trait]
pub trait PreSignPolicyGuardEngine: SovereignEngine {
    /// Deserializes `envelope.raw_unsigned_payload` and validates security policies and signatures.
    async fn inspect_and_authorize_raw_payload(
        &self,
        envelope: &UnsignedTransactionEnvelope,
        quorum_signatures: &[HardwareQuorumSignature],
    ) -> Result<PolicyVerificationReceipt, SovereignError>;

    /// Checks whether `(chain_id, destination_tier, address)` is present in the PCR0 pinned whitelist.
    fn is_pinned_whitelisted_destination(
        &self,
        chain_id: ChainId,
        destination_tier: WalletTier,
        address: &str,
    ) -> bool;
}
