#![forbid(unsafe_code)]
#![deny(
    clippy::float_arithmetic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::indexing_slicing
)]

use alloy_primitives::U256;
use gw_chain_adapters::MultiChainSweepAssemblerEngine;
use gw_enterprise_traits::oberer_storage_engine::SovereignEngine;
use gw_enterprise_traits::sweep_engine_trait::UnsignedSweepAssemblerEngine;
use gw_primitives::{
    ChainArchitecture, ChainId, LogicalAsset, PhysicalAssetId, RawAssetAmount, SovereignError,
    WalletTier,
};
use uuid::Uuid;

fn sample_asset(chain_id: ChainId, keep_dust: bool, decimals: u8) -> PhysicalAssetId {
    PhysicalAssetId {
        chain_id,
        logical_asset: match chain_id {
            ChainId::TronMainnet => LogicalAsset::Trx,
            ChainId::SolanaMainnet => LogicalAsset::Sol,
            _ => LogicalAsset::Eth,
        },
        token_contract: "0x0000000000000000000000000000000000000000".to_string(),
        decimals,
        keep_one_unit_dust: keep_dust,
        target_confirmations: 12,
    }
}

/// Audit Test 1: Sweep Assembly Matches Sweeper Factory Calldata Descriptor.
#[tokio::test]
async fn test_sweep_assembly_matches_factory_calldata() -> Result<(), Box<dyn std::error::Error>> {
    let chain_id = ChainId::EthereumMainnet;
    let engine = MultiChainSweepAssemblerEngine::new(chain_id);
    let asset = sample_asset(chain_id, false, 18);

    let locators = vec![
        "0x1111111111111111111111111111111111111111".to_string(),
        "0x2222222222222222222222222222222222222222".to_string(),
    ];
    let cold_vault = "0x3333333333333333333333333333333333333333";
    let relayer = "0x4444444444444444444444444444444444444444";

    let envelope = engine
        .assemble_tier0_deposit_sweep(
            &locators,
            cold_vault,
            WalletTier::Tier3ColdVault,
            &asset,
            relayer,
        )
        .await?;

    assert_eq!(envelope.chain_id, chain_id);
    assert_eq!(envelope.architecture, ChainArchitecture::EvmAccount);
    assert_eq!(envelope.destination_tier, WalletTier::Tier3ColdVault);
    assert_eq!(envelope.expected_destination_address, cold_vault);
    assert!(envelope.raw_unsigned_payload.starts_with(b"GW9_CANONICAL_TX\x00"));
    assert!(!envelope.payload_sha256_hex.is_empty());

    Ok(())
}

/// Audit Test 2: Tron 1-Sun Dust Keeper Retains Exact 1 Sun to Prevent 0.1 TRX Re-Activation Penalty.
#[tokio::test]
async fn test_tron_1sun_dust_keeper_retains_balance() -> Result<(), Box<dyn std::error::Error>> {
    let chain_id = ChainId::TronMainnet;
    let engine = MultiChainSweepAssemblerEngine::new(chain_id);
    let asset = sample_asset(chain_id, true, 6);

    let factory = [0u8; 20];
    let init_code_hash = [0xc5; 32];
    let locators = vec![gw_chain_adapters::derive_tron_create2_address(factory, [1u8; 32], init_code_hash)];
    let cold_vault = gw_chain_adapters::derive_tron_create2_address(factory, [2u8; 32], init_code_hash);
    let relayer = gw_chain_adapters::derive_tron_create2_address(factory, [3u8; 32], init_code_hash);

    let envelope = engine
        .assemble_tier0_deposit_sweep(
            &locators,
            &cold_vault,
            WalletTier::Tier3ColdVault,
            &asset,
            &relayer,
        )
        .await?;

    // Nominal balance was 1_000_000_000_000_000_000; with 1 sun kept: nominal - 1
    let nominal = U256::from(1_000_000_000_000_000_000_u64);
    let expected_swept = nominal - U256::from(1);
    assert_eq!(envelope.net_transfer_amount.raw(), expected_swept);

    // Simulate envelope to verify 1 sun dust retention report
    let sim_report = engine.simulate_unsigned_envelope(&envelope).await?;
    assert!(sim_report.simulation_succeeded);
    assert_eq!(sim_report.reclaimed_rent_or_dust_saved, 1, "Tron must report exactly 1 Sun dust saved");

    Ok(())
}

/// Audit Test 3: Pre-Broadcast Simulation Blocks Reverting / Zero Transactions.
#[tokio::test]
async fn test_pre_simulation_blocks_failing_tx() -> Result<(), Box<dyn std::error::Error>> {
    let chain_id = ChainId::EthereumMainnet;
    let engine = MultiChainSweepAssemblerEngine::new(chain_id);
    let asset = sample_asset(chain_id, false, 18);

    let zero_envelope = gw_enterprise_traits::sweep_engine_trait::UnsignedTransactionEnvelope {
        envelope_id: Uuid::new_v4(),
        chain_id,
        architecture: ChainArchitecture::EvmAccount,
        source_tier: WalletTier::Tier0CustomerDeposit,
        destination_tier: WalletTier::Tier2OperationalHot,
        source_addresses: vec!["0x1111111111111111111111111111111111111111".to_string()],
        expected_destination_address: "0x2222222222222222222222222222222222222222".to_string(),
        physical_asset: asset,
        net_transfer_amount: RawAssetAmount::zero(18),
        max_network_fee_native: RawAssetAmount::new(U256::from(21_000_000_000_u64), 18),
        nonce_or_sequence: 1,
        raw_unsigned_payload: vec![1, 2, 3],
        payload_sha256_hex: "010203".to_string(),
        signer_key_locators: vec!["loc://test".to_string()],
    };

    let report = engine.simulate_unsigned_envelope(&zero_envelope).await?;
    assert!(!report.simulation_succeeded, "Zero transfer amount must fail simulation");
    assert!(
        report.revert_reason.unwrap_or_default().contains("zero net transfer"),
        "Expected revert reason for zero amount"
    );

    Ok(())
}

/// Audit Test 4: RBF / EIP-1559 Fee-Bump Enforces >= 120% Multiplier and Preserves Nonce.
#[tokio::test]
async fn test_rbf_fee_bump_calculation() -> Result<(), Box<dyn std::error::Error>> {
    let chain_id = ChainId::EthereumMainnet;
    let engine = MultiChainSweepAssemblerEngine::new(chain_id);
    let asset = sample_asset(chain_id, false, 18);

    let locators = vec!["0x1111111111111111111111111111111111111111".to_string()];
    let cold_vault = "0x2222222222222222222222222222222222222222";
    let relayer = "0x3333333333333333333333333333333333333333";

    let initial_envelope = engine
        .assemble_tier0_deposit_sweep(
            &locators,
            cold_vault,
            WalletTier::Tier3ColdVault,
            &asset,
            relayer,
        )
        .await?;

    // Subcase 4A: Fee bump < 12,000 bps (110% = 11,000 bps) -> Fails with EnclavePolicyViolation
    let sub_120_result = engine.assemble_fee_bump_replacement(&initial_envelope, 11_000).await;
    match sub_120_result {
        Err(SovereignError::EnclavePolicyViolation { reason }) => {
            assert!(
                reason.contains("at least 12,000 bps"),
                "Expected 120% minimum bump error, got: {reason}"
            );
        }
        other => panic!("Expected EnclavePolicyViolation, got {other:?}"),
    }

    // Subcase 4B: Fee bump >= 12,000 bps (125% = 12,500 bps) -> Succeeds!
    let bumped_envelope = engine.assemble_fee_bump_replacement(&initial_envelope, 12_500).await?;

    // Nonce must be preserved for RBF replacement
    assert_eq!(bumped_envelope.nonce_or_sequence, initial_envelope.nonce_or_sequence);

    // Fee must be exactly 1.25x the old fee
    let expected_fee = initial_envelope.max_network_fee_native.raw() * U256::from(12_500) / U256::from(10_000);
    assert_eq!(bumped_envelope.max_network_fee_native.raw(), expected_fee);

    // Payload SHA-256 must be newly computed
    assert_ne!(bumped_envelope.payload_sha256_hex, initial_envelope.payload_sha256_hex);

    Ok(())
}

/// Audit Test 5: Solana PDA Rent Reclamation Invariant.
#[tokio::test]
async fn test_solana_pda_rent_reclamation_in_simulation() -> Result<(), Box<dyn std::error::Error>> {
    let chain_id = ChainId::SolanaMainnet;
    let engine = MultiChainSweepAssemblerEngine::new(chain_id);
    let asset = sample_asset(chain_id, false, 9);

    let pda_address = "11111111111111111111111111111111";
    let treasury_vault = "4fYNw3dojWmQ4dXtSGE9epjRGy9pFSx62YypT7avPYvA";
    let relayer = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

    let envelope = engine
        .assemble_tier0_deposit_sweep(
            &[pda_address.to_string()],
            treasury_vault,
            WalletTier::Tier3ColdVault,
            &asset,
            relayer,
        )
        .await?;

    let report = engine.simulate_unsigned_envelope(&envelope).await?;
    assert!(report.simulation_succeeded);
    assert_eq!(
        report.reclaimed_rent_or_dust_saved, 2_039_280,
        "Solana sweep must report exactly 2,039,280 lamports rent reclamation constant"
    );

    Ok(())
}

/// Audit Test 6: MultiChain Sweep Assembler SovereignEngine Lifecycle & Broadcast.
#[tokio::test]
async fn test_multichain_sweep_assembler_oop_lifecycle() -> Result<(), Box<dyn std::error::Error>> {
    let chain_id = ChainId::EthereumMainnet;
    let engine = MultiChainSweepAssemblerEngine::new(chain_id);

    // Initialize
    engine.initialize_engine().await?;

    // Audit Cryptographic Integrity
    let integrity = engine.audit_cryptographic_integrity().await?;
    assert!(integrity.audit_passed);

    // Health Snapshot
    let health = engine.health_snapshot().await?;
    assert!(health.is_healthy);

    // Test Broadcast of Signed Bytes
    let dummy_signed_bytes = vec![0xde, 0xad, 0xbe, 0xef];
    let tx_hash = engine.broadcast_signed_transaction(Uuid::new_v4(), &dummy_signed_bytes).await?;
    assert!(tx_hash.starts_with("0x"));

    // Empty broadcast fails
    let empty_res = engine.broadcast_signed_transaction(Uuid::new_v4(), &[]).await;
    match empty_res {
        Err(SovereignError::NodeTransportFailure { chain, detail }) => {
            assert_eq!(chain, chain_id);
            assert!(detail.contains("empty"));
        }
        other => panic!("Expected NodeTransportFailure, got {other:?}"),
    }

    Ok(())
}
