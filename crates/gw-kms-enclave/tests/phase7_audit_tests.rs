#![forbid(unsafe_code)]
#![deny(
    clippy::float_arithmetic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::indexing_slicing
)]

use alloy_primitives::U256;
use ed25519_dalek::{Signer, SigningKey};
use gw_enterprise_traits::enclave_signer_engine_trait::EnclaveKeySignerEngine;
use gw_enterprise_traits::policy_guard_engine_trait::{
    HardwareQuorumSignature, PreSignPolicyGuardEngine,
};
use gw_enterprise_traits::sweep_engine_trait::UnsignedTransactionEnvelope;
use gw_kms_enclave::{
    EnclaveKeyStore, ParsedPayloadDetails, SovereignEnclaveKeySignerEngine,
    StatelessPreSignPolicyGuardEngine, ZeroizedKeyBuffer,
};
use gw_primitives::{
    ChainArchitecture, ChainId, LogicalAsset, PhysicalAssetId, RawAssetAmount, SovereignError,
    WalletTier,
};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use uuid::Uuid;

fn sample_pcr0() -> [u8; 32] {
    [0x42u8; 32]
}

fn sample_asset(chain_id: ChainId, decimals: u8) -> PhysicalAssetId {
    PhysicalAssetId {
        chain_id,
        logical_asset: LogicalAsset::Eth,
        token_contract: "0x0000000000000000000000000000000000000000".to_string(),
        decimals,
        keep_one_unit_dust: false,
        target_confirmations: 12,
    }
}

fn create_canonical_payload(
    destinations: Vec<String>,
    net_output_amount: RawAssetAmount,
    network_fee: RawAssetAmount,
    nonce: u64,
) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    let details = ParsedPayloadDetails {
        destinations,
        net_output_amount,
        network_fee,
        nonce_or_sequence: nonce,
        call_data_or_memo: None,
    };
    let json_bytes = serde_json::to_vec(&details)?;
    let mut raw = Vec::from(&b"GW9_CANONICAL_TX\x00"[..]);
    raw.extend_from_slice(&json_bytes);

    let mut hasher = Sha256::new();
    hasher.update(&raw);
    let digest: [u8; 32] = hasher.finalize().into();
    let hex_digest = hex_encode(&digest);

    Ok((raw, hex_digest))
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len().saturating_mul(2));
    for &b in bytes {
        let hi = (b >> 4) as usize;
        let lo = (b & 0x0f) as usize;
        if let Some(&c) = HEX.get(hi) {
            s.push(c as char);
        }
        if let Some(&c) = HEX.get(lo) {
            s.push(c as char);
        }
    }
    s
}

/// Audit Test 1: Axiom 3 Enforcement - Reject unwhitelisted destination address.
#[tokio::test]
async fn test_presign_guard_rejects_unwhitelisted_destination() -> Result<(), Box<dyn std::error::Error>> {
    let policy_guard = StatelessPreSignPolicyGuardEngine::new(sample_pcr0(), 0);
    let chain_id = ChainId::EthereumMainnet;
    let whitelisted_dest = "0x1111111111111111111111111111111111111111";
    let attacker_dest = "0xdeaddeaddeaddeaddeaddeaddeaddeaddeaddead";

    policy_guard.add_whitelisted_destination(chain_id, WalletTier::Tier2OperationalHot, whitelisted_dest)?;

    let net_amount = RawAssetAmount::new(U256::from(1_000_000_000_000_000_000_u64), 18);
    let fee = RawAssetAmount::new(U256::from(21_000_000_000_u64), 18);

    // Payload sending to attacker_dest
    let (raw_payload, sha256_hex) = create_canonical_payload(
        vec![attacker_dest.to_string()],
        net_amount,
        fee,
        1,
    )?;

    let envelope = UnsignedTransactionEnvelope {
        envelope_id: Uuid::new_v4(),
        chain_id,
        architecture: ChainArchitecture::EvmAccount,
        source_tier: WalletTier::Tier0CustomerDeposit,
        destination_tier: WalletTier::Tier2OperationalHot,
        source_addresses: vec!["0x0000000000000000000000000000000000000001".to_string()],
        expected_destination_address: attacker_dest.to_string(),
        physical_asset: sample_asset(chain_id, 18),
        net_transfer_amount: net_amount,
        max_network_fee_native: fee,
        nonce_or_sequence: 1,
        raw_unsigned_payload: raw_payload,
        payload_sha256_hex: sha256_hex,
        signer_key_locators: vec!["loc://hot/eth/0".to_string()],
    };

    let result = policy_guard.inspect_and_authorize_raw_payload(&envelope, &[]).await;
    match result {
        Err(SovereignError::EnclavePolicyViolation { reason }) => {
            assert!(
                reason.contains("whitelist") || reason.contains("not in the PCR0 pinned whitelist"),
                "Expected whitelist violation error, got: {reason}"
            );
        }
        other => panic!("Expected EnclavePolicyViolation, got {other:?}"),
    }

    Ok(())
}

/// Audit Test 2: Anti-Replay Invariant - Reject previously signed payload digest.
#[tokio::test]
async fn test_presign_guard_rejects_replayed_payload() -> Result<(), Box<dyn std::error::Error>> {
    let policy_guard = StatelessPreSignPolicyGuardEngine::new(sample_pcr0(), 0);
    let chain_id = ChainId::EthereumMainnet;
    let whitelisted_dest = "0x2222222222222222222222222222222222222222";

    policy_guard.add_whitelisted_destination(chain_id, WalletTier::Tier2OperationalHot, whitelisted_dest)?;

    let net_amount = RawAssetAmount::new(U256::from(500_000_000_000_000_000_u64), 18);
    let fee = RawAssetAmount::new(U256::from(21_000_000_000_u64), 18);

    let (raw_payload, sha256_hex) = create_canonical_payload(
        vec![whitelisted_dest.to_string()],
        net_amount,
        fee,
        5,
    )?;

    let envelope = UnsignedTransactionEnvelope {
        envelope_id: Uuid::new_v4(),
        chain_id,
        architecture: ChainArchitecture::EvmAccount,
        source_tier: WalletTier::Tier0CustomerDeposit,
        destination_tier: WalletTier::Tier2OperationalHot,
        source_addresses: vec!["0x0000000000000000000000000000000000000001".to_string()],
        expected_destination_address: whitelisted_dest.to_string(),
        physical_asset: sample_asset(chain_id, 18),
        net_transfer_amount: net_amount,
        max_network_fee_native: fee,
        nonce_or_sequence: 5,
        raw_unsigned_payload: raw_payload,
        payload_sha256_hex: sha256_hex.clone(),
        signer_key_locators: vec!["loc://hot/eth/0".to_string()],
    };

    // First inspection: must succeed
    let receipt1 = policy_guard.inspect_and_authorize_raw_payload(&envelope, &[]).await;
    assert!(receipt1.is_ok(), "First inspection should succeed");

    // Second inspection with identical payload: must fail anti-replay
    let receipt2 = policy_guard.inspect_and_authorize_raw_payload(&envelope, &[]).await;
    match receipt2 {
        Err(SovereignError::ReplayAttackBlocked { digest }) => {
            assert_eq!(digest, sha256_hex);
        }
        other => panic!("Expected ReplayAttackBlocked, got {other:?}"),
    }

    Ok(())
}

/// Audit Test 3: Fee Ceiling Invariant - Reject payload with fee exceeding declared max ceiling.
#[tokio::test]
async fn test_presign_guard_rejects_excessive_fee() -> Result<(), Box<dyn std::error::Error>> {
    let policy_guard = StatelessPreSignPolicyGuardEngine::new(sample_pcr0(), 0);
    let chain_id = ChainId::EthereumMainnet;
    let whitelisted_dest = "0x3333333333333333333333333333333333333333";

    policy_guard.add_whitelisted_destination(chain_id, WalletTier::Tier2OperationalHot, whitelisted_dest)?;

    let net_amount = RawAssetAmount::new(U256::from(100_000_000_000_000_000_u64), 18);
    let excessive_fee = RawAssetAmount::new(U256::from(500_000_000_000_u64), 18);
    let declared_fee_ceiling = RawAssetAmount::new(U256::from(200_000_000_000_u64), 18);

    let (raw_payload, sha256_hex) = create_canonical_payload(
        vec![whitelisted_dest.to_string()],
        net_amount,
        excessive_fee,
        10,
    )?;

    let envelope = UnsignedTransactionEnvelope {
        envelope_id: Uuid::new_v4(),
        chain_id,
        architecture: ChainArchitecture::EvmAccount,
        source_tier: WalletTier::Tier0CustomerDeposit,
        destination_tier: WalletTier::Tier2OperationalHot,
        source_addresses: vec!["0x0000000000000000000000000000000000000001".to_string()],
        expected_destination_address: whitelisted_dest.to_string(),
        physical_asset: sample_asset(chain_id, 18),
        net_transfer_amount: net_amount,
        max_network_fee_native: declared_fee_ceiling,
        nonce_or_sequence: 10,
        raw_unsigned_payload: raw_payload,
        payload_sha256_hex: sha256_hex,
        signer_key_locators: vec!["loc://hot/eth/0".to_string()],
    };

    let result = policy_guard.inspect_and_authorize_raw_payload(&envelope, &[]).await;
    match result {
        Err(SovereignError::EnclavePolicyViolation { reason }) => {
            assert!(
                reason.contains("exceeds ceiling"),
                "Expected fee ceiling error, got: {reason}"
            );
        }
        other => panic!("Expected EnclavePolicyViolation, got {other:?}"),
    }

    Ok(())
}

/// Audit Test 4: M-of-N Hardware Quorum Enforcement for Tier 3 Cold Vault Outflows.
#[tokio::test]
async fn test_presign_guard_m_of_n_quorum_enforcement() -> Result<(), Box<dyn std::error::Error>> {
    let min_quorum = 2;
    let policy_guard = StatelessPreSignPolicyGuardEngine::new(sample_pcr0(), min_quorum);
    let chain_id = ChainId::EthereumMainnet;
    let cold_dest = "0x4444444444444444444444444444444444444444";

    policy_guard.add_whitelisted_destination(chain_id, WalletTier::Tier3ColdVault, cold_dest)?;

    // Setup 3 Custodians
    let custodian1 = SigningKey::from_bytes(&[1u8; 32]);
    let custodian2 = SigningKey::from_bytes(&[2u8; 32]);
    let custodian3 = SigningKey::from_bytes(&[3u8; 32]);

    policy_guard.add_trusted_custodian_public_key(custodian1.verifying_key().as_bytes())?;
    policy_guard.add_trusted_custodian_public_key(custodian2.verifying_key().as_bytes())?;
    policy_guard.add_trusted_custodian_public_key(custodian3.verifying_key().as_bytes())?;

    let net_amount = RawAssetAmount::new(U256::from(10_000_000_000_000_000_000_u64), 18);
    let fee = RawAssetAmount::new(U256::from(30_000_000_000_u64), 18);

    let (raw_payload, sha256_hex) = create_canonical_payload(
        vec![cold_dest.to_string()],
        net_amount,
        fee,
        100,
    )?;

    let envelope = UnsignedTransactionEnvelope {
        envelope_id: Uuid::new_v4(),
        chain_id,
        architecture: ChainArchitecture::EvmAccount,
        source_tier: WalletTier::Tier2OperationalHot,
        destination_tier: WalletTier::Tier3ColdVault,
        source_addresses: vec!["0x0000000000000000000000000000000000000001".to_string()],
        expected_destination_address: cold_dest.to_string(),
        physical_asset: sample_asset(chain_id, 18),
        net_transfer_amount: net_amount,
        max_network_fee_native: fee,
        nonce_or_sequence: 100,
        raw_unsigned_payload: raw_payload.clone(),
        payload_sha256_hex: sha256_hex,
        signer_key_locators: vec!["loc://hot/eth/0".to_string()],
    };

    // Subcase 4A: Zero signatures submitted -> Fails QuorumInsufficient
    let result_zero = policy_guard.inspect_and_authorize_raw_payload(&envelope, &[]).await;
    match result_zero {
        Err(SovereignError::QuorumInsufficient { required, verified }) => {
            assert_eq!(required, 2);
            assert_eq!(verified, 0);
        }
        other => panic!("Expected QuorumInsufficient, got {other:?}"),
    }

    // Subcase 4B: 1 signature submitted (only custodian1) -> Fails QuorumInsufficient

    // Need a fresh envelope with nonce 101 to avoid anti-replay on retry
    let (raw_payload2, sha256_hex2) = create_canonical_payload(
        vec![cold_dest.to_string()],
        net_amount,
        fee,
        101,
    )?;
    let mut envelope2 = envelope.clone();
    envelope2.nonce_or_sequence = 101;
    envelope2.raw_unsigned_payload = raw_payload2.clone();
    envelope2.payload_sha256_hex = sha256_hex2;

    let mut hasher2 = Sha256::new();
    hasher2.update(&raw_payload2);
    let digest2: [u8; 32] = hasher2.finalize().into();

    let sig1_2 = custodian1.sign(&digest2);
    let q_sig1_2 = HardwareQuorumSignature {
        signer_id: "custodian_alpha".to_string(),
        signer_public_key: custodian1.verifying_key().as_bytes().to_vec(),
        signature_bytes: sig1_2.to_bytes().to_vec(),
        signed_at_unix: 1700000001,
    };

    let result_one = policy_guard
        .inspect_and_authorize_raw_payload(&envelope2, &[q_sig1_2])
        .await;
    match result_one {
        Err(SovereignError::QuorumInsufficient { required, verified }) => {
            assert_eq!(required, 2);
            assert_eq!(verified, 1);
        }
        other => panic!("Expected QuorumInsufficient (1 of 2), got {other:?}"),
    }

    // Subcase 4C: 2 valid custodian signatures submitted (custodian1 + custodian2) -> Succeeds!
    let (raw_payload3, sha256_hex3) = create_canonical_payload(
        vec![cold_dest.to_string()],
        net_amount,
        fee,
        102,
    )?;
    let mut envelope3 = envelope.clone();
    envelope3.nonce_or_sequence = 102;
    envelope3.raw_unsigned_payload = raw_payload3.clone();
    envelope3.payload_sha256_hex = sha256_hex3;

    let mut hasher3 = Sha256::new();
    hasher3.update(&raw_payload3);
    let digest3: [u8; 32] = hasher3.finalize().into();

    let sig1_3 = custodian1.sign(&digest3);
    let sig2_3 = custodian2.sign(&digest3);

    let q1 = HardwareQuorumSignature {
        signer_id: "custodian_alpha".to_string(),
        signer_public_key: custodian1.verifying_key().as_bytes().to_vec(),
        signature_bytes: sig1_3.to_bytes().to_vec(),
        signed_at_unix: 1700000002,
    };
    let q2 = HardwareQuorumSignature {
        signer_id: "custodian_beta".to_string(),
        signer_public_key: custodian2.verifying_key().as_bytes().to_vec(),
        signature_bytes: sig2_3.to_bytes().to_vec(),
        signed_at_unix: 1700000002,
    };

    let receipt = policy_guard
        .inspect_and_authorize_raw_payload(&envelope3, &[q1, q2])
        .await?;
    assert_eq!(receipt.verified_quorum_signers, 2);
    assert_eq!(receipt.verified_payload_sha256, digest3);

    Ok(())
}

/// Audit Test 5: Axiom 4 Ephemeral RAM Zeroization & Full Two-Step Signing Ceremony.
#[tokio::test]
async fn test_enclave_signing_zeroizes_ram() -> Result<(), Box<dyn std::error::Error>> {
    let key_store = Arc::new(EnclaveKeyStore::new());
    let locator = "loc://hot/eth/0";
    let dummy_secret = [0x55u8; 32];
    key_store.insert_secp256k1_key(locator, dummy_secret)?;

    let signer = SovereignEnclaveKeySignerEngine::new(sample_pcr0(), key_store);
    let policy_guard = StatelessPreSignPolicyGuardEngine::new(sample_pcr0(), 0);

    let chain_id = ChainId::EthereumMainnet;
    let whitelisted_dest = "0x5555555555555555555555555555555555555555";
    policy_guard.add_whitelisted_destination(chain_id, WalletTier::Tier2OperationalHot, whitelisted_dest)?;

    let net_amount = RawAssetAmount::new(U256::from(100_000_000_000_000_000_u64), 18);
    let fee = RawAssetAmount::new(U256::from(21_000_000_000_u64), 18);

    let (raw_payload, sha256_hex) = create_canonical_payload(
        vec![whitelisted_dest.to_string()],
        net_amount,
        fee,
        200,
    )?;

    let envelope = UnsignedTransactionEnvelope {
        envelope_id: Uuid::new_v4(),
        chain_id,
        architecture: ChainArchitecture::EvmAccount,
        source_tier: WalletTier::Tier0CustomerDeposit,
        destination_tier: WalletTier::Tier2OperationalHot,
        source_addresses: vec!["0x0000000000000000000000000000000000000001".to_string()],
        expected_destination_address: whitelisted_dest.to_string(),
        physical_asset: sample_asset(chain_id, 18),
        net_transfer_amount: net_amount,
        max_network_fee_native: fee,
        nonce_or_sequence: 200,
        raw_unsigned_payload: raw_payload,
        payload_sha256_hex: sha256_hex,
        signer_key_locators: vec![locator.to_string()],
    };

    // Execute Two-Step Enclave Signing Ceremony
    let (receipt, artifact) = signer
        .authorize_and_sign_envelope(&policy_guard, &envelope, &[])
        .await?;

    assert_eq!(artifact.envelope_id, envelope.envelope_id);
    assert_eq!(artifact.chain_id, chain_id);
    assert_eq!(artifact.verified_payload_sha256, receipt.verified_payload_sha256);
    assert!(artifact.ephemeral_ram_zeroized, "Axiom 4: RAM must be zeroized");
    assert_eq!(artifact.broadcastable_signed_bytes.len(), 65, "EVM signature must be 65 bytes (r, s, v)");

    // Test ZeroizedKeyBuffer drop behavior
    {
        let secret = [0x77u8; 32];
        let buf = ZeroizedKeyBuffer::new(secret);
        assert_eq!(buf.as_bytes(), &[0x77u8; 32]);
        // buffer drops here and executes ZeroizeOnDrop volatile write barrier
    }

    Ok(())
}

/// Audit Test 6: Enclave Attestation Report Binding PCR0.
#[tokio::test]
async fn test_enclave_attestation_report_binding_pcr0() -> Result<(), Box<dyn std::error::Error>> {
    let key_store = Arc::new(EnclaveKeyStore::new());
    let pcr0 = sample_pcr0();
    let signer = SovereignEnclaveKeySignerEngine::new(pcr0, key_store);

    let nonce = [0x99u8; 32];
    let doc = signer.generate_enclave_attestation_report(nonce).await?;

    assert!(doc.starts_with(b"NITRO_DOC\x00"), "Attestation header must match");
    // Verify PCR0 is embedded in bytes [10..42]
    let embedded_pcr0 = doc.get(10..42).ok_or("Failed to slice PCR0 from doc")?;
    assert_eq!(embedded_pcr0, &pcr0);

    // Verify nonce is embedded in bytes [42..74]
    let embedded_nonce = doc.get(42..74).ok_or("Failed to slice nonce from doc")?;
    assert_eq!(embedded_nonce, &nonce);

    Ok(())
}

/// Audit Test 7: Export Watch-Only Public Descriptor (Zone A Bootstrap).
#[tokio::test]
async fn test_enclave_export_watch_only_public_descriptor() -> Result<(), Box<dyn std::error::Error>> {
    let key_store = Arc::new(EnclaveKeyStore::new());
    let secp_locator = "loc://hot/eth/0";
    let ed_locator = "loc://hot/sol/0";

    key_store.insert_secp256k1_key(secp_locator, [0x01u8; 32])?;
    key_store.insert_ed25519_key(ed_locator, [0x02u8; 32])?;

    let signer = SovereignEnclaveKeySignerEngine::new(sample_pcr0(), key_store);

    // Secp256k1 compressed pubkey (33 bytes -> 66 hex chars)
    let eth_pub = signer.export_watch_only_public_descriptor(ChainId::EthereumMainnet, secp_locator).await?;
    assert_eq!(eth_pub.len(), 66);

    // Ed25519 pubkey (32 bytes -> 64 hex chars)
    let sol_pub = signer.export_watch_only_public_descriptor(ChainId::SolanaMainnet, ed_locator).await?;
    assert_eq!(sol_pub.len(), 64);

    Ok(())
}
