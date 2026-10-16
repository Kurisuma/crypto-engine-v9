#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic, clippy::unwrap_used)]

use crate::oberer_storage_engine::SovereignEngine;
use crate::policy_guard_engine_trait::{
    HardwareQuorumSignature, PolicyVerificationReceipt, PreSignPolicyGuardEngine,
};
use crate::sweep_engine_trait::UnsignedTransactionEnvelope;
use async_trait::async_trait;
use gw_primitives::{ChainId, SovereignError};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Signed transaction artifact emitted by the Zone C Enclave after zeroizing temporary RAM buffers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedTransactionArtifact {
    pub envelope_id: Uuid,
    pub chain_id: ChainId,
    pub verified_payload_sha256: [u8; 32],
    pub broadcastable_signed_bytes: Vec<u8>,
    pub expected_tx_hash: String,
    pub ephemeral_ram_zeroized: bool,
}

/// Engine Trait IX: Unified Zero-Trust Enclave Key Signer Engine.
#[async_trait]
pub trait EnclaveKeySignerEngine: SovereignEngine {
    /// Executes the two-step Enclave Signing Ceremony with ephemeral RAM zeroization.
    async fn authorize_and_sign_envelope(
        &self,
        policy_guard: &dyn PreSignPolicyGuardEngine,
        envelope: &UnsignedTransactionEnvelope,
        quorum_signatures: &[HardwareQuorumSignature],
    ) -> Result<(PolicyVerificationReceipt, SignedTransactionArtifact), SovereignError>;

    /// Exports only the extended public key (`zpub`) or view/relayer public key for Zone A bootstrap.
    async fn export_watch_only_public_descriptor(
        &self,
        chain_id: ChainId,
        key_locator: &str,
    ) -> Result<String, SovereignError>;

    /// Generates a cryptographic Nitro Enclave / TPM attestation document binding PCR0.
    async fn generate_enclave_attestation_report(
        &self,
        nonce: [u8; 32],
    ) -> Result<Vec<u8>, SovereignError>;
}
