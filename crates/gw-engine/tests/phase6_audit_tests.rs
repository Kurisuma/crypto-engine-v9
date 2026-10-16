#![forbid(unsafe_code)]
#![deny(
    clippy::float_arithmetic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::unwrap_used,
    clippy::indexing_slicing
)]

use alloy_primitives::U256;
use gw_chain_adapters::{DepositAddressPoolEngine, MultiChainWatchOnlyAdapter};
use gw_engine::{FiveGateEvaluator, SovereignInvoiceVerificationEngine};
use gw_enterprise_traits::{
    ConsensusBlockHeader, InvoiceVerificationEngine, NormalizedTransferEvent,
    ObererOrmStorageEngine, SovereignEngine, TamperEvidentOrmModel, VerifiedTransitionOutcome,
    WatchOnlyDerivationEngine,
};
use gw_primitives::{
    ChainArchitecture, ChainId, LogicalAsset, MicroCreditsMathBridge, PhysicalAssetId,
    RawAssetAmount, SovereignError,
};
use gw_storage_oberer::{InvoiceLifecycleStatus, InvoiceRow, ObererHybridStorageEngineInstance};
use oberer_orm::{MicroCredits, OrmResultExt, Table, TableOps};
use std::sync::Arc;

fn mock_header(chain_id: ChainId, height: i64, timestamp: i64) -> ConsensusBlockHeader {
    ConsensusBlockHeader {
        chain_id,
        block_number: height,
        block_hash: format!("0xblock_{height}"),
        parent_block_hash: format!("0xblock_{}", height.saturating_sub(1)),
        state_root_or_merkle_root: format!("0xroot_{height}"),
        timestamp_unix: timestamp,
        is_finalized: false,
    }
}

fn mock_asset(chain_id: ChainId, logical: LogicalAsset, decimals: u8) -> PhysicalAssetId {
    PhysicalAssetId {
        chain_id,
        logical_asset: logical,
        token_contract: "0xdac17f958d2ee523a2206206994597c13d831ec7".to_string(),
        decimals,
        keep_one_unit_dust: false,
        target_confirmations: 12,
    }
}

async fn setup_engine() -> (
    Arc<dyn ObererOrmStorageEngine>,
    SovereignInvoiceVerificationEngine,
) {
    let row_mac_key = [0x5A_u8; 32];
    let storage_inst = ObererHybridStorageEngineInstance::in_memory_testing(row_mac_key)
        .expect("Storage init failed");
    storage_inst
        .initialize_engine()
        .await
        .expect("Storage engine init failed");
    let storage: Arc<dyn ObererOrmStorageEngine> = Arc::new(storage_inst);

    let pool = Arc::new(DepositAddressPoolEngine::new(storage.clone()));
    let adapter = MultiChainWatchOnlyAdapter::new(
        ChainId::EthereumMainnet,
        "0x1111111111111111111111111111111111111111",
        pool,
        [0x42_u8; 32],
    )
    .with_create2([0x12; 20], [0x34; 32]);
    let derivation: Arc<dyn WatchOnlyDerivationEngine> = Arc::new(adapter);

    let engine = SovereignInvoiceVerificationEngine::new(
        storage.clone(),
        derivation,
        MicroCreditsMathBridge,
    );
    engine.initialize_engine().await.expect("Engine init failed");

    (storage, engine)
}

/// 1. 5-Gate Conjunction Theorem & Affine Linear Typestate Transition to Complete
#[tokio::test]
async fn test_5gate_all_pass_transitions_to_complete() {
    let (_storage, engine) = setup_engine().await;
    let asset = mock_asset(ChainId::EthereumMainnet, LogicalAsset::Usdt, 6);

    // Fiat: $100.00 USD = 100_000_000 MicroCredits
    // Price: $1.00 USD = 1_000_000 MicroCredits
    let fiat = MicroCredits::new(100_000_000);
    let price = MicroCredits::new(1_000_000);
    let tolerance_bps = 50; // 0.50%
    let ttl_seconds = 3600;

    let invoice = engine
        .create_and_persist_locked_invoice(
            None,
            asset.clone(),
            fiat,
            price,
            price,
            tolerance_bps,
            ttl_seconds,
        )
        .await
        .expect("Invoice creation failed");

    assert_eq!(invoice.expected_amount_raw.raw(), U256::from(100_000_000_u64));

    // Construct valid transfer event satisfying all 5 gates
    let now = 1700000000_i64;
    let header = mock_header(ChainId::EthereumMainnet, 20_000_000, now.saturating_add(60));
    let event = NormalizedTransferEvent {
        chain_id: ChainId::EthereumMainnet,
        architecture: ChainArchitecture::EvmAccount,
        tx_hash: "0xdeadbeefcafe0001".to_string(),
        event_index: 0,
        block_header: header,
        token_contract: asset.token_contract.clone(),
        sender_address: "0xsender1111111111111111111111111111111111".to_string(),
        recipient_address: invoice.route.deposit_address.clone(),
        watchlist_lookup_key: invoice.route.deposit_address.clone(),
        net_credited_amount: invoice.expected_amount_raw,
        confirmations: 12,
        execution_succeeded: true,
    };

    let outcome = engine
        .evaluate_pending_invoice(invoice, &event, true)
        .await
        .expect("Evaluation failed");

    match outcome {
        VerifiedTransitionOutcome::MovedToVerifiedComplete(complete) => {
            assert_eq!(complete.state.settlement_tx_hashes.len(), 1);
            assert_eq!(
                complete.state.settlement_tx_hashes.first().map(|s| s.as_str()),
                Some("0xdeadbeefcafe0001")
            );
            assert!(!complete.state.is_overpaid);
            assert_eq!(complete.state.surplus_raw.raw(), U256::ZERO);
            assert!(complete
                .state
                .fulfillment_idempotency_token
                .starts_with("FULFILL_"));
        }
        _ => panic!("Expected MovedToVerifiedComplete"),
    }
}

/// 2. Gate 1 Rejection: Reverted On-Chain Transaction
#[tokio::test]
async fn test_gate1_reverted_tx_fails() {
    let (_storage, engine) = setup_engine().await;
    let asset = mock_asset(ChainId::EthereumMainnet, LogicalAsset::Usdt, 6);

    let invoice = engine
        .create_and_persist_locked_invoice(
            None,
            asset.clone(),
            MicroCredits::new(50_000_000),
            MicroCredits::new(1_000_000),
            MicroCredits::new(1_000_000),
            50,
            1800,
        )
        .await
        .expect("Invoice creation failed");

    let header = mock_header(ChainId::EthereumMainnet, 20_000_001, 1700000010);
    let event = NormalizedTransferEvent {
        chain_id: ChainId::EthereumMainnet,
        architecture: ChainArchitecture::EvmAccount,
        tx_hash: "0xreverted_tx".to_string(),
        event_index: 0,
        block_header: header,
        token_contract: asset.token_contract.clone(),
        sender_address: "0xsender".to_string(),
        recipient_address: invoice.route.deposit_address.clone(),
        watchlist_lookup_key: invoice.route.deposit_address.clone(),
        net_credited_amount: invoice.expected_amount_raw,
        confirmations: 12,
        execution_succeeded: false, // Fails Gate 1
    };

    let err = engine
        .evaluate_pending_invoice(invoice, &event, true)
        .await
        .expect_err("Must fail Gate 1");

    match err {
        SovereignError::GateVerificationFailed { gate, .. } => assert_eq!(gate, 1),
        _ => panic!("Expected GateVerificationFailed for Gate 1"),
    }
}

/// 3. Gate 2 Rejection: Counterfeit Contract and Address Mismatch
#[tokio::test]
async fn test_gate2_counterfeit_contract_and_address_mismatch_fails() {
    let (_storage, engine) = setup_engine().await;
    let asset = mock_asset(ChainId::EthereumMainnet, LogicalAsset::Usdt, 6);

    let invoice = engine
        .create_and_persist_locked_invoice(
            None,
            asset.clone(),
            MicroCredits::new(50_000_000),
            MicroCredits::new(1_000_000),
            MicroCredits::new(1_000_000),
            50,
            1800,
        )
        .await
        .expect("Invoice creation failed");

    let header = mock_header(ChainId::EthereumMainnet, 20_000_002, 1700000020);

    // Case 2A: Fake USDT token contract
    let fake_token_event = NormalizedTransferEvent {
        chain_id: ChainId::EthereumMainnet,
        architecture: ChainArchitecture::EvmAccount,
        tx_hash: "0xfake_token".to_string(),
        event_index: 0,
        block_header: header.clone(),
        token_contract: "0xdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef".to_string(),
        sender_address: "0xsender".to_string(),
        recipient_address: invoice.route.deposit_address.clone(),
        watchlist_lookup_key: invoice.route.deposit_address.clone(),
        net_credited_amount: invoice.expected_amount_raw,
        confirmations: 12,
        execution_succeeded: true,
    };

    let err_token = engine
        .evaluate_pending_invoice(invoice.clone(), &fake_token_event, true)
        .await
        .expect_err("Must fail Gate 2 token match");

    match err_token {
        SovereignError::GateVerificationFailed { gate, .. } => assert_eq!(gate, 2),
        _ => panic!("Expected Gate 2 error"),
    }

    // Case 2B: Wrong recipient address
    let wrong_dest_event = NormalizedTransferEvent {
        chain_id: ChainId::EthereumMainnet,
        architecture: ChainArchitecture::EvmAccount,
        tx_hash: "0xwrong_dest".to_string(),
        event_index: 0,
        block_header: header,
        token_contract: asset.token_contract.clone(),
        sender_address: "0xsender".to_string(),
        recipient_address: "0x9999999999999999999999999999999999999999".to_string(),
        watchlist_lookup_key: "0x9999999999999999999999999999999999999999".to_string(),
        net_credited_amount: invoice.expected_amount_raw,
        confirmations: 12,
        execution_succeeded: true,
    };

    let err_dest = engine
        .evaluate_pending_invoice(invoice, &wrong_dest_event, true)
        .await
        .expect_err("Must fail Gate 2 destination match");

    match err_dest {
        SovereignError::GateVerificationFailed { gate, .. } => assert_eq!(gate, 2),
        _ => panic!("Expected Gate 2 error"),
    }
}

/// 4. Gate 3: Tolerance-Bounded Amount Arithmetic & Shortfall Monotonicity
#[tokio::test]
async fn test_gate3_tolerance_boundary_exact_bps() {
    // Expected: 1,000 USDT = 1_000_000_000 raw
    let expected = RawAssetAmount::new(U256::from(1_000_000_000_u64), 6);
    let cumulative_prior = RawAssetAmount::new(U256::ZERO, 6);
    let tolerance_bps = 50_i64; // 50 bps = 0.50%
    // Minimum threshold: 1_000_000_000 * (10000 - 50) / 10000 = 995_000_000

    // Test 4A: Underpaid by 49 bps (Received 995_100_000) -> >= 995_000_000 -> Accepted Complete
    let delta_49bps = RawAssetAmount::new(U256::from(995_100_000_u64), 6);
    let (cum1, is_comp1, is_over1, surplus1) = FiveGateEvaluator::evaluate_gate3(
        &expected,
        &cumulative_prior,
        &delta_49bps,
        tolerance_bps,
    )
    .expect("Gate 3 eval failed");

    assert_eq!(cum1.raw(), U256::from(995_100_000_u64));
    assert!(is_comp1, "49 bps underpaid must be accepted within 50 bps tolerance");
    assert!(!is_over1);
    assert_eq!(surplus1.raw(), U256::ZERO);

    // Test 4B: Underpaid by 51 bps (Received 994_900_000) -> < 995_000_000 -> Shortfall Partial
    let delta_51bps = RawAssetAmount::new(U256::from(994_900_000_u64), 6);
    let (cum2, is_comp2, is_over2, shortfall2) = FiveGateEvaluator::evaluate_gate3(
        &expected,
        &cumulative_prior,
        &delta_51bps,
        tolerance_bps,
    )
    .expect("Gate 3 eval failed");

    assert_eq!(cum2.raw(), U256::from(994_900_000_u64));
    assert!(!is_comp2, "51 bps underpaid must NOT complete invoice");
    assert!(!is_over2);
    // Shortfall: 1_000_000_000 - 994_900_000 = 5_100_000
    assert_eq!(shortfall2.raw(), U256::from(5_100_000_u64));

    // Test 4C: Overpaid (Received 1_000_500_000) -> Complete with Surplus
    let delta_over = RawAssetAmount::new(U256::from(1_000_500_000_u64), 6);
    let (cum3, is_comp3, is_over3, surplus3) = FiveGateEvaluator::evaluate_gate3(
        &expected,
        &cumulative_prior,
        &delta_over,
        tolerance_bps,
    )
    .expect("Gate 3 eval failed");

    assert_eq!(cum3.raw(), U256::from(1_000_500_000_u64));
    assert!(is_comp3);
    assert!(is_over3);
    assert_eq!(surplus3.raw(), U256::from(500_000_u64));
}

/// 5. Gate 4: Monotonic Expiration Window Check
#[tokio::test]
async fn test_gate4_monotonic_expiration() {
    let (_storage, engine) = setup_engine().await;
    let asset = mock_asset(ChainId::EthereumMainnet, LogicalAsset::Usdt, 6);

    let invoice = engine
        .create_and_persist_locked_invoice(
            None,
            asset.clone(),
            MicroCredits::new(100_000_000),
            MicroCredits::new(1_000_000),
            MicroCredits::new(1_000_000),
            50,
            600, // TTL 600s
        )
        .await
        .expect("Invoice creation failed");

    // Event arrives after expiration: expires_at_unix + 10
    let late_timestamp = invoice.expires_at_unix.saturating_add(10);
    let header = mock_header(ChainId::EthereumMainnet, 20_000_005, late_timestamp);
    let late_event = NormalizedTransferEvent {
        chain_id: ChainId::EthereumMainnet,
        architecture: ChainArchitecture::EvmAccount,
        tx_hash: "0xlate_payment".to_string(),
        event_index: 0,
        block_header: header,
        token_contract: asset.token_contract.clone(),
        sender_address: "0xsender".to_string(),
        recipient_address: invoice.route.deposit_address.clone(),
        watchlist_lookup_key: invoice.route.deposit_address.clone(),
        net_credited_amount: invoice.expected_amount_raw,
        confirmations: 12,
        execution_succeeded: true,
    };

    let outcome = engine
        .evaluate_pending_invoice(invoice, &late_event, true)
        .await
        .expect("Evaluation failed");

    match outcome {
        VerifiedTransitionOutcome::MovedToExpired(expired) => {
            assert_eq!(expired.state.expired_at_unix, late_timestamp);
            assert_eq!(
                expired.state.late_received_raw.raw(),
                late_event.net_credited_amount.raw()
            );
        }
        _ => panic!("Expected MovedToExpired"),
    }
}

/// 6. Gate 5: Confirmation Depth Gating & Detection Progression
#[tokio::test]
async fn test_gate5_confirmation_depth_and_advance() {
    let (_storage, engine) = setup_engine().await;
    let asset = mock_asset(ChainId::EthereumMainnet, LogicalAsset::Usdt, 6);

    let invoice = engine
        .create_and_persist_locked_invoice(
            None,
            asset.clone(),
            MicroCredits::new(100_000_000),
            MicroCredits::new(1_000_000),
            MicroCredits::new(1_000_000),
            50,
            3600,
        )
        .await
        .expect("Invoice creation failed");

    // Event has only 4 confirmations (< 12 target confirmations)
    let header = mock_header(ChainId::EthereumMainnet, 20_000_010, 1700000100);
    let low_conf_event = NormalizedTransferEvent {
        chain_id: ChainId::EthereumMainnet,
        architecture: ChainArchitecture::EvmAccount,
        tx_hash: "0xlow_conf_tx".to_string(),
        event_index: 0,
        block_header: header,
        token_contract: asset.token_contract.clone(),
        sender_address: "0xsender".to_string(),
        recipient_address: invoice.route.deposit_address.clone(),
        watchlist_lookup_key: invoice.route.deposit_address.clone(),
        net_credited_amount: invoice.expected_amount_raw,
        confirmations: 4, // 4 < 12
        execution_succeeded: true,
    };

    let outcome1 = engine
        .evaluate_pending_invoice(invoice, &low_conf_event, true)
        .await
        .expect("Evaluation failed");

    let detected_invoice = match outcome1 {
        VerifiedTransitionOutcome::RemainsDetected(det) => {
            assert_eq!(det.state.current_confirmations, 4);
            assert_eq!(det.state.target_confirmations, 12);
            det
        }
        _ => panic!("Expected RemainsDetected"),
    };

    // Advance confirmations: first to 8 (< 12) -> still RemainsDetected
    let outcome2 = engine
        .advance_detected_invoice(detected_invoice, 8, true)
        .await
        .expect("Advance failed");

    let detected_8 = match outcome2 {
        VerifiedTransitionOutcome::RemainsDetected(det) => {
            assert_eq!(det.state.current_confirmations, 8);
            det
        }
        _ => panic!("Expected RemainsDetected at 8 confirmations"),
    };

    // Advance confirmations to 12 (>= 12 target) -> transitions to Complete!
    let outcome3 = engine
        .advance_detected_invoice(detected_8, 12, true)
        .await
        .expect("Advance failed");

    match outcome3 {
        VerifiedTransitionOutcome::MovedToVerifiedComplete(complete) => {
            assert_eq!(complete.state.settlement_tx_hashes.len(), 1);
            assert_eq!(
                complete.state.settlement_tx_hashes.first().map(|s| s.as_str()),
                Some("0xlow_conf_tx")
            );
        }
        _ => panic!("Expected MovedToVerifiedComplete at 12 confirmations"),
    }
}

/// 7. Consensus Reorg Rollback above LCA: Reversion to Pending & Row MAC Re-sealing
#[tokio::test]
async fn test_reorg_reversion_to_pending_above_lca() {
    let (storage, engine) = setup_engine().await;
    let asset = mock_asset(ChainId::EthereumMainnet, LogicalAsset::Usdt, 6);

    let invoice = engine
        .create_and_persist_locked_invoice(
            None,
            asset.clone(),
            MicroCredits::new(250_000_000),
            MicroCredits::new(1_000_000),
            MicroCredits::new(1_000_000),
            50,
            3600,
        )
        .await
        .expect("Invoice creation failed");

    // Manually mark invoice as Detected at block 115 in storage
    let conn = storage.memory_conn();
    let table = Table::<InvoiceRow>::new(conn);
    let mut row = table
        .find(invoice.invoice_id)
        .unenvelope()
        .expect("Query failed")
        .expect("Invoice not found");

    row.status = InvoiceLifecycleStatus::Detected;
    row.detected_at_block = Some(115);
    row.current_confirmations = 2;
    row.received_amount_blob = invoice.expected_amount_raw.to_oberer_blob_vec();
    row.received_amount_dec78 = invoice.expected_amount_raw.to_oberer_dec78();
    let next_version = row.version.saturating_add(1);
    let mac = storage.row_mac_signer().compute_row_mac(
        InvoiceRow::TABLE_LABEL,
        &row.pk_string(),
        next_version,
        &row.canonical_security_bytes(),
    );
    row.assign_row_mac(mac);
    table
        .update_with_version(row.id, row.version, &row)
        .unenvelope()
        .expect("Update to Detected failed");

    // Now consensus reorg happens: LCA = 110 (block 115 is orphaned!)
    let rolled_back = engine
        .rollback_reorged_invoices_above_lca(110)
        .await
        .expect("Reorg rollback failed");

    assert_eq!(rolled_back.len(), 1);
    let reverted_invoice = rolled_back.first().expect("Invoice missing in rollback list");
    assert_eq!(reverted_invoice.invoice_id, invoice.invoice_id);

    // Verify database state: status is Pending, detected_at_block is None, received is 0
    let db_row = table
        .find(invoice.invoice_id)
        .unenvelope()
        .expect("Query failed")
        .expect("Invoice missing");

    assert_eq!(db_row.status, InvoiceLifecycleStatus::Pending);
    assert_eq!(db_row.detected_at_block, None);
    assert_eq!(db_row.current_confirmations, 0);

    // Verify cryptographic integrity audit passes on storage engine
    let audit_report = engine
        .audit_cryptographic_integrity()
        .await
        .expect("Audit failed");
    assert!(audit_report.audit_passed, "Row MACs must be strictly valid after reorg rollback");
    assert_eq!(audit_report.detected_corruptions_or_tampers, 0);
}
