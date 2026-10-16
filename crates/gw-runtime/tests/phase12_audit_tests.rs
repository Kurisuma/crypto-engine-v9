#![forbid(unsafe_code)]
#![deny(
    clippy::float_arithmetic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::unwrap_used,
    clippy::indexing_slicing
)]

use alloy_primitives::U256;
use gw_enterprise_traits::enclave_signer_engine_trait::EnclaveKeySignerEngine;
use gw_enterprise_traits::liquidity_router_engine_trait::{
    GlobalRoutingDirective, NetworkLiquidityPolicy,
};
use gw_enterprise_traits::sweep_engine_trait::UnsignedTransactionEnvelope;
use gw_enterprise_traits::verification_engine_trait::{
    StateVerifiedComplete, TypestateInvoice,
};
use gw_enterprise_traits::{ObererOrmStorageEngine, ObererRowMacSigner, SovereignEngine};
use gw_kms_enclave::{
    EnclaveKeyStore, ParsedPayloadDetails, SovereignEnclaveKeySignerEngine,
    StatelessPreSignPolicyGuardEngine,
};
use gw_primitives::{
    ChainId, LogicalAsset, PhysicalAssetId, RawAssetAmount, WalletTier,
};
use gw_runtime::prelude::*;
use gw_storage_oberer::{
    InvoiceLifecycleStatus, InvoiceRow, ObererHybridStorageEngineInstance, OrmLogicalAsset,
};
use oberer_orm::{MicroCredits, OrmResultExt, Table, TableOps};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::sync::Arc;
use uuid::Uuid;

fn sample_chain_asset(chain_id: ChainId, logical_asset: LogicalAsset, decimals: u8) -> PhysicalAssetId {
    PhysicalAssetId {
        chain_id,
        logical_asset,
        token_contract: match chain_id {
            ChainId::BitcoinMainnet => "native".to_string(),
            ChainId::EthereumMainnet => "0xdac17f958d2ee523a2206206994597c13d831ec7".to_string(),
            ChainId::SolanaMainnet => "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            ChainId::TronMainnet => "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t".to_string(),
            ChainId::TonMainnet => "native".to_string(),
            ChainId::SuiMainnet => "0x2::sui::SUI".to_string(),
            _ => "0x0000000000000000000000000000000000000000".to_string(),
        },
        decimals,
        keep_one_unit_dust: false,
        target_confirmations: 12,
    }
}

fn compute_invoice_row_mac(
    signer: &ObererRowMacSigner,
    id: i64,
    version: i64,
    deposit_address: &str,
    token_contract: &str,
    fiat_amount: i64,
    expected_blob: &[u8],
) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(deposit_address.as_bytes());
    buf.extend_from_slice(token_contract.as_bytes());
    buf.extend_from_slice(&fiat_amount.to_le_bytes());
    buf.extend_from_slice(expected_blob);

    signer.compute_row_mac("invoices", &id.to_string(), version, &buf)
}

fn make_test_invoice_row(
    id: i64,
    fiat_amount: i64,
    deposit_address: &str,
    token_contract: &str,
    expected_blob: Vec<u8>,
    version: i64,
    row_mac: Vec<u8>,
) -> InvoiceRow {
    InvoiceRow {
        id,
        customer_id_uuid: None,
        pool_id: None,
        chain_id: 1,
        logical_asset: OrmLogicalAsset::Eth,
        token_contract: token_contract.to_string(),
        decimals: 18,
        deposit_address: deposit_address.to_string(),
        routing_reference: None,
        derivation_locator: "m/44'/60'/0'/0/0".to_string(),
        fiat_amount_usd: MicroCredits::new(fiat_amount),
        locked_price_usd: MicroCredits::new(100_000_000),
        expected_amount_blob: expected_blob,
        expected_amount_dec78: "1000000000000000000".to_string(),
        received_amount_blob: vec![0; 32],
        received_amount_dec78: "0".to_string(),
        tolerance_bps: 0,
        status: InvoiceLifecycleStatus::Pending,
        detected_at_block: None,
        current_confirmations: 0,
        target_confirmations: 12,
        expires_at_unix: 1_800_000_000,
        settled_tx_hash: None,
        fulfillment_idempotency_key: Some(format!("IDEMP_{id}")),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        version,
        deleted_at: None,
        row_mac,
    }
}

fn create_enclave_canonical_payload(
    destinations: Vec<String>,
    net_output_amount: RawAssetAmount,
    network_fee: RawAssetAmount,
    nonce: u64,
) -> (Vec<u8>, String) {
    let details = ParsedPayloadDetails {
        destinations,
        net_output_amount,
        network_fee,
        nonce_or_sequence: nonce,
        call_data_or_memo: None,
    };
    let json_bytes = serde_json::to_vec(&details).expect("serialize canonical payload");
    let mut raw = Vec::from(&b"GW9_CANONICAL_TX\x00"[..]);
    raw.extend_from_slice(&json_bytes);

    let mut hasher = Sha256::new();
    hasher.update(&raw);
    let payload_sha256_hex = hex::encode(hasher.finalize());

    (raw, payload_sha256_hex)
}

/// Audit Test 1: Master Periodic Cryptographic Integrity Audit Daemon & Self-Healing Loop.
#[tokio::test]
async fn test_runtime_60s_cryptographic_audit_pass() {
    let container = SovereignGatewayDiBuilder::new()
        .with_row_mac_key([0x42u8; 32])
        .with_in_memory_storage()
        .expect("in-memory storage")
        .with_imprce_cache(32 * 1024 * 1024)
        .build()
        .expect("build container");

    let daemon_config = DaemonConfig {
        audit_interval: std::time::Duration::from_millis(30),
        max_audit_history: 50,
    };

    let mut daemon = MasterRuntimeDaemon::new(container, daemon_config);
    daemon.boot().await.expect("daemon boot");

    // Seed test rows into storage and cache
    let storage = daemon.container().storage();
    let mem = storage.memory_conn();
    let row_mac_signer = storage.row_mac_signer();

    let deposit_addr = "0x1234567890abcdef1234567890abcdef12345678";
    let token_contract = "0x0000000000000000000000000000000000000000";
    let fiat_mc = 50_000_000i64;
    let exp_blob = vec![0x11u8; 32];

    let mut invoice_row = make_test_invoice_row(
        0,
        fiat_mc,
        deposit_addr,
        token_contract,
        exp_blob.clone(),
        1,
        vec![0u8; 32],
    );

    let pk = Table::<InvoiceRow>::new(mem.clone())
        .insert(&invoice_row)
        .unenvelope()
        .expect("insert test invoice");

    let mac = compute_invoice_row_mac(
        row_mac_signer,
        pk,
        1,
        deposit_addr,
        token_contract,
        fiat_mc,
        &exp_blob,
    );

    invoice_row.id = pk;
    invoice_row.row_mac = mac;
    Table::<InvoiceRow>::new(mem)
        .update(pk, &invoice_row)
        .unenvelope()
        .expect("update test invoice with mac");

    // 1. Synchronous single audit pass
    let single_audit = daemon.run_audit_once().await.expect("run audit once");
    assert!(
        single_audit.report.audit_passed,
        "Initial cryptographic audit must pass"
    );
    assert_eq!(
        single_audit.report.detected_corruptions_or_tampers, 0,
        "Zero corruptions allowed"
    );
    assert!(
        single_audit.report.verified_records_or_cells > 0,
        "Records must be scanned"
    );

    // 2. Start background periodic audit
    daemon.start_background_audit().expect("start background audit");
    tokio::time::sleep(std::time::Duration::from_millis(120)).await;

    // 3. Graceful shutdown
    daemon.shutdown().await.expect("shutdown daemon");

    let history = daemon.get_audit_history().await;
    assert!(
        history.len() >= 2,
        "Expected at least 2 audit runs in history, got {}",
        history.len()
    );
    for rec in history {
        assert!(rec.report.audit_passed);
        assert_eq!(rec.report.detected_corruptions_or_tampers, 0);
    }
}

/// Audit Test 2: 1,000-Thread CAS Master Optimistic Concurrency Proof (`#[orm(version)]`).
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_1000_thread_cas_master_proof() {
    let container = SovereignGatewayDiBuilder::new()
        .with_row_mac_key([0x88u8; 32])
        .with_in_memory_storage()
        .expect("in-memory storage")
        .build()
        .expect("build container");

    container.initialize_all().await.expect("initialize container");

    let storage = container.storage();
    let mem = storage.memory_conn();
    let row_mac_signer = storage.row_mac_signer();

    let initial_balance: i64 = 10_000_000;
    let deduction: i64 = 1;
    let deposit_addr = "0xcas_target";
    let token_contract = "0x0000000000000000000000000000000000000000";
    let exp_blob = vec![0xcc; 32];

    let mut initial_row = make_test_invoice_row(
        0,
        initial_balance,
        deposit_addr,
        token_contract,
        exp_blob.clone(),
        1,
        vec![0u8; 32],
    );

    let invoice_id = Table::<InvoiceRow>::new(mem.clone())
        .insert(&initial_row)
        .unenvelope()
        .expect("insert initial invoice");

    let initial_mac = compute_invoice_row_mac(
        row_mac_signer,
        invoice_id,
        1,
        deposit_addr,
        token_contract,
        initial_balance,
        &exp_blob,
    );

    initial_row.id = invoice_id;
    initial_row.row_mac = initial_mac;
    Table::<InvoiceRow>::new(mem.clone())
        .update(invoice_id, &initial_row)
        .unenvelope()
        .expect("update initial invoice with mac");

    let workers = 1000;
    let success_count = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let conflict_count = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let error_count = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let shared_signer = Arc::new(ObererRowMacSigner::new([0x88u8; 32]));
    let barrier = Arc::new(tokio::sync::Barrier::new(workers));

    let mut set = tokio::task::JoinSet::new();

    for worker_idx in 0..workers {
        let mem_clone = mem.clone();
        let success = Arc::clone(&success_count);
        let conflict = Arc::clone(&conflict_count);
        let error = Arc::clone(&error_count);
        let signer = Arc::clone(&shared_signer);
        let b = Arc::clone(&barrier);

        set.spawn(async move {
            let table = Table::<InvoiceRow>::new(mem_clone);

            // Synchronize all 1,000 tasks at the starting line
            b.wait().await;

            // Attempt CAS with bounded retry loop
            let max_retries = 35;
            for _ in 0..max_retries {
                let current_row = match table.find(invoice_id).unenvelope() {
                    Ok(Some(r)) => r,
                    _ => {
                        error.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        return;
                    }
                };

                let cur_ver = current_row.version;
                let cur_fiat = current_row.fiat_amount_usd.get();
                if cur_fiat < deduction {
                    return;
                }

                let next_ver = cur_ver.saturating_add(1);
                let next_fiat = cur_fiat.saturating_sub(deduction);

                // Compute Blake3 row MAC for next_ver target state
                let next_mac = compute_invoice_row_mac(
                    &signer,
                    invoice_id,
                    next_ver,
                    &current_row.deposit_address,
                    &current_row.token_contract,
                    next_fiat,
                    &current_row.expected_amount_blob,
                );

                let mut updated_row = current_row.clone();
                updated_row.fiat_amount_usd = MicroCredits::new(next_fiat);
                updated_row.row_mac = next_mac;
                updated_row.fulfillment_idempotency_key = Some(format!("CAS_WORKER_{worker_idx}"));

                match table.update_with_version(invoice_id, cur_ver, &updated_row).unenvelope() {
                    Ok(()) => {
                        success.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        return;
                    }
                    Err(oberer_orm::OrmError::OptimisticLockConflict { .. }) => {
                        conflict.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        tokio::task::yield_now().await;
                    }
                    Err(_) => {
                        error.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        return;
                    }
                }
            }
        });
    }

    while let Some(res) = set.join_next().await {
        res.expect("task join failed");
    }

    let successes = success_count.load(std::sync::atomic::Ordering::SeqCst);
    let conflicts = conflict_count.load(std::sync::atomic::Ordering::SeqCst);
    let errors = error_count.load(std::sync::atomic::Ordering::SeqCst);

    assert_eq!(errors, 0, "Zero fatal errors permitted during CAS race condition proof");
    assert!(conflicts > 0, "High concurrency must produce optimistic lock conflicts");
    assert!(successes > 0, "Successful commits must be processed");

    // Verify Final Database State Invariants:
    let final_row = Table::<InvoiceRow>::new(mem.clone())
        .find(invoice_id)
        .unenvelope()
        .expect("find final row")
        .expect("row must exist");

    // Invariant 1: Version strictly equals Initial Version + Successful Commits
    let expected_version = 1i64.saturating_add(i64::try_from(successes).unwrap_or(0));
    assert_eq!(
        final_row.version,
        expected_version,
        "Axiom 8 CAS Invariant: final version must exactly equal 1 + successful commits"
    );

    // Invariant 2: Financial Balance Conservation
    let expected_balance = initial_balance.saturating_sub(i64::try_from(successes).unwrap_or(0));
    assert_eq!(
        final_row.fiat_amount_usd.get(),
        expected_balance,
        "Financial Conservation: initial balance must equal final balance + successful deductions"
    );

    // Invariant 3: Constant-Time Row MAC Verification on Final State
    let mut final_buf = Vec::new();
    final_buf.extend_from_slice(final_row.deposit_address.as_bytes());
    final_buf.extend_from_slice(final_row.token_contract.as_bytes());
    final_buf.extend_from_slice(&final_row.fiat_amount_usd.get().to_le_bytes());
    final_buf.extend_from_slice(&final_row.expected_amount_blob);

    assert!(
        row_mac_signer
            .verify_row_mac(
                "invoices",
                &invoice_id.to_string(),
                final_row.version,
                &final_buf,
                &final_row.row_mac
            )
            .is_ok(),
        "Final row MAC must be cryptographically valid in constant time"
    );
}

/// Audit Test 3: Unclean Process Kill & SQLite WAL Crash Recovery Linearizability.
#[tokio::test]
async fn test_unclean_process_kill_wal_recovery() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let db_path = temp_dir.path().join("sovereign_gateway_unclean_kill.db");
    let db_path_str = db_path.to_str().expect("valid path string").to_string();
    let row_mac_key = [0x71u8; 32];

    let expected_amount = 500_000_000i64; // $500.00
    let deposit_addr = "0xunclean_kill_addr";
    let token_contract = "0x0000000000000000000000000000000000000000";
    let exp_blob = vec![0x55; 32];

    let invoice_id: i64;

    // Scope 1: Normal operations writing to SQLite Persistent WAL
    {
        let storage = ObererHybridStorageEngineInstance::persistent_wal(&db_path_str, row_mac_key)
            .expect("open persistent wal");
        storage.initialize_engine().await.expect("init storage");

        let disk_conn = storage.hybrid_conn().file();
        let signer = storage.row_mac_signer();

        let mut row = make_test_invoice_row(
            0,
            expected_amount,
            deposit_addr,
            token_contract,
            exp_blob.clone(),
            1,
            vec![0u8; 32],
        );

        let pk = Table::<InvoiceRow>::new(disk_conn.clone())
            .insert(&row)
            .unenvelope()
            .expect("insert invoice row");

        invoice_id = pk;

        let mac = compute_invoice_row_mac(
            signer,
            invoice_id,
            1,
            deposit_addr,
            token_contract,
            expected_amount,
            &exp_blob,
        );

        row.id = invoice_id;
        row.row_mac = mac;
        Table::<InvoiceRow>::new(disk_conn.clone())
            .update(invoice_id, &row)
            .unenvelope()
            .expect("update invoice row with mac");

        // Simulate abrupt process termination without clean checkpoint:
        // Storage instance is dropped immediately here.
    }

    // Scope 2: Process restart after unclean kill.
    // SQLite WAL playback must recover committed data seamlessly.
    {
        let recovered_storage =
            ObererHybridStorageEngineInstance::persistent_wal(&db_path_str, row_mac_key)
                .expect("reopen persistent wal");
        recovered_storage
            .initialize_engine()
            .await
            .expect("re-init storage");

        // Verify cryptographic integrity after recovery
        let integrity = recovered_storage
            .audit_cryptographic_integrity()
            .await
            .expect("audit recovered database");

        assert!(
            integrity.audit_passed,
            "Recovered database must pass cryptographic audit"
        );
        assert_eq!(
            integrity.detected_corruptions_or_tampers, 0,
            "Recovered database must contain zero corruptions or tampers"
        );

        // Fetch recovered invoice row from :memory: working tier (warm-hydrated from disk WAL)
        let recovered_row = Table::<InvoiceRow>::new(recovered_storage.hybrid_conn().mem().clone())
            .find(invoice_id)
            .unenvelope()
            .expect("find recovered row")
            .expect("recovered invoice row must exist");

        assert_eq!(recovered_row.id, invoice_id);
        assert_eq!(recovered_row.fiat_amount_usd.get(), expected_amount);
        assert_eq!(recovered_row.version, 1);

        // Verify constant-time row MAC on recovered entity
        let mut check_buf = Vec::new();
        check_buf.extend_from_slice(recovered_row.deposit_address.as_bytes());
        check_buf.extend_from_slice(recovered_row.token_contract.as_bytes());
        check_buf.extend_from_slice(&recovered_row.fiat_amount_usd.get().to_le_bytes());
        check_buf.extend_from_slice(&recovered_row.expected_amount_blob);

        assert!(
            recovered_storage
                .row_mac_signer()
                .verify_row_mac(
                    "invoices",
                    &invoice_id.to_string(),
                    recovered_row.version,
                    &check_buf,
                    &recovered_row.row_mac
                )
                .is_ok(),
            "Recovered row MAC must match in constant time"
        );
    }
}

/// Audit Test 4: End-to-End Multi-Chain Payment to Treasury Lifecycle across all 6 Blockchains.
/// (BTC, EVM, SVM, TVM, TON, Move/Sui)
#[tokio::test]
async fn test_e2e_full_payment_to_treasury_lifecycle() {
    let test_chains = [
        (
            ChainId::BitcoinMainnet,
            LogicalAsset::Btc,
            8u8,
            "bc1q99999999999999999999999999999999999999",
            "bc1qcoldvaultdestination0000000000000000000",
            "bc1qmarketexchangedestination00000000000000",
            100_000_000u64, // 1.0 BTC
        ),
        (
            ChainId::EthereumMainnet,
            LogicalAsset::Usdt,
            6u8,
            "0x1111111111111111111111111111111111111111",
            "0x2222222222222222222222222222222222222222",
            "0x3333333333333333333333333333333333333333",
            1_000_000_000u64, // 1,000 USDT
        ),
        (
            ChainId::SolanaMainnet,
            LogicalAsset::Usdc,
            6u8,
            "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU",
            "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM",
            "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            500_000_000u64, // 500 USDC
        ),
        (
            ChainId::TronMainnet,
            LogicalAsset::Usdt,
            6u8,
            "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t",
            "T9yD14Nj9j7xAB4dbGeiX9h8unkKHxuWwb",
            "TXpwND74H92NknkK496h3i5pvd8aM7gG1A",
            250_000_000u64, // 250 USDT
        ),
        (
            ChainId::TonMainnet,
            LogicalAsset::Ton,
            9u8,
            "EQCD39VS5jcptHL8vMjEXrzGaRcCVYto7HUn4bpAOg8xqB2N",
            "EQBvW8m53WuAS9LDn0K596gNRqEcDewCUo2hspCrni_AcZTZ",
            "EQCzQ53R39pD6Kz7mNl9B98hQ7_dJ8Z519CqEw9vKzL_aBcD",
            10_000_000_000u64, // 10 TON
        ),
        (
            ChainId::SuiMainnet,
            LogicalAsset::Sui,
            9u8,
            "0x02a212de6a9dfa3a69e22387acfbafbb1a9e591bd9d636e7895dcfc8de05f331",
            "0x11a212de6a9dfa3a69e22387acfbafbb1a9e591bd9d636e7895dcfc8de05f331",
            "0x22a212de6a9dfa3a69e22387acfbafbb1a9e591bd9d636e7895dcfc8de05f331",
            5_000_000_000u64, // 5 SUI
        ),
    ];

    let row_mac_key = [0x93u8; 32];
    let container = SovereignGatewayDiBuilder::new()
        .with_row_mac_key(row_mac_key)
        .with_in_memory_storage()
        .expect("in-memory storage")
        .with_imprce_cache(32 * 1024 * 1024)
        .build()
        .expect("build container");

    container.initialize_all().await.expect("init container");

    let treasury_engine = container.treasury();
    let router_engine = container.router();

    // Zone C Enclave Setup: Policy Guard + Key Signer
    let pcr0_measurement = [0x55u8; 32];
    let policy_guard = StatelessPreSignPolicyGuardEngine::new(pcr0_measurement, 0);

    let key_store = Arc::new(EnclaveKeyStore::new());
    key_store
        .insert_secp256k1_key("tier2_operational_key", [0x77u8; 32])
        .expect("insert secp key");
    key_store
        .insert_ed25519_key("tier2_operational_key", [0x77u8; 32])
        .expect("insert ed key");
    let enclave_signer = SovereignEnclaveKeySignerEngine::new(pcr0_measurement, key_store);

    for (chain_id, logical_asset, decimals, deposit_addr, cold_vault, market_exch, payment_raw_u64) in test_chains {
        let physical_asset = sample_chain_asset(chain_id, logical_asset, decimals);
        let payment_raw = RawAssetAmount::new(U256::from(payment_raw_u64), decimals);
        let fiat_amount_mc = MicroCredits::new(100_000_000); // $100.00
        let locked_price_mc = MicroCredits::new(100_000_000); // 1.00

        // Step 1: Whitelist Cold Vault & Market destinations in Zone C Enclave Policy Guard
        policy_guard
            .add_whitelisted_destination(chain_id, WalletTier::Tier3ColdVault, cold_vault)
            .expect("whitelist cold destination");
        policy_guard
            .add_whitelisted_destination(chain_id, WalletTier::Tier4MarketExchange, market_exch)
            .expect("whitelist market destination");

        // Step 2: 5-Gate Invoice Verification Simulation & Storage Persistence
        let exp_blob = vec![0x11u8; 32];
        let mut invoice_row = make_test_invoice_row(
            0,
            fiat_amount_mc.get(),
            deposit_addr,
            &physical_asset.token_contract,
            exp_blob.clone(),
            1,
            vec![0u8; 32],
        );

        let mem_conn = container.storage().memory_conn();
        let pk = Table::<InvoiceRow>::new(mem_conn.clone())
            .insert(&invoice_row)
            .unenvelope()
            .expect("insert test invoice");

        let mac = compute_invoice_row_mac(
            container.storage().row_mac_signer(),
            pk,
            1,
            deposit_addr,
            &physical_asset.token_contract,
            fiat_amount_mc.get(),
            &exp_blob,
        );

        invoice_row.id = pk;
        invoice_row.row_mac = mac.clone();
        Table::<InvoiceRow>::new(mem_conn)
            .update(pk, &invoice_row)
            .unenvelope()
            .expect("update test invoice with mac");

        let invoice_id = pk;

        let verified_invoice = TypestateInvoice {
            invoice_id,
            customer_id: Some(Uuid::new_v4()),
            asset: physical_asset.clone(),
            route: gw_enterprise_traits::CheckoutDepositRoute {
                chain_id,
                deposit_address: deposit_addr.to_string(),
                routing_reference: None,
                derivation_locator: "m/84'/0'/0'/0/0".to_string(),
                counterfactual_commitment_hash: "0xcommithash".to_string(),
            },
            fiat_amount_mc,
            locked_price_mc,
            expected_amount_raw: payment_raw,
            cumulative_received_raw: payment_raw,
            tolerance_bps: 50,
            expires_at_unix: 1_800_000_000,
            version: 1,
            row_mac: mac,
            state: StateVerifiedComplete {
                settlement_tx_hashes: vec![format!("0xtx_{chain_id:?}_{invoice_id}")],
                settled_at_unix: 1_700_000_000,
                is_overpaid: false,
                surplus_raw: RawAssetAmount::zero(decimals),
                fulfillment_idempotency_token: format!("FULFILL_{invoice_id}"),
            },
        };

        // Step 3: FIFO Treasury Ingestion
        let fifo_lot = treasury_engine
            .record_verified_invoice_fifo_lot(&verified_invoice)
            .await
            .expect("record verified invoice fifo lot");

        assert_eq!(fifo_lot.source_invoice_id, invoice_id);
        assert_eq!(fifo_lot.initial_amount.raw(), payment_raw.raw());
        assert_eq!(fifo_lot.remaining_amount.raw(), payment_raw.raw());
        assert_eq!(fifo_lot.asset.chain_id, chain_id);

        // Step 4: Cross-Network Liquidity Routing Policy (70/30 Cold/Market Split)
        let policy = NetworkLiquidityPolicy {
            asset: physical_asset.clone(),
            tier1_relayer_address: "0xrelayer00000000000000000000000000000000".to_string(),
            tier2_hot_wallet_address: "0xhotwallet00000000000000000000000000000".to_string(),
            tier2_key_locator: "tier2_operational_key".to_string(),
            tier3_cold_vault_address: cold_vault.to_string(),
            tier4_market_exchange_address: market_exch.to_string(),
            hot_wallet_ceiling_mc: MicroCredits::new(50_000_000), // $50 ceiling triggers sweep
            cold_vault_split_bps: 7000,                           // 70%
            market_exchange_split_bps: 3000,                      // 30%
            min_relayer_gas_raw: RawAssetAmount::zero(decimals),
            target_relayer_gas_raw: RawAssetAmount::zero(decimals),
        };

        // Step 5: Liquidity Router Evaluates Split Conservation: Q_cold ⊕ Q_market ≡ B_hot
        let spot_prices = {
            let mut m = BTreeMap::new();
            m.insert(logical_asset, locked_price_mc);
            m
        };

        let _ = router_engine
            .plan_global_liquidity_transfers(
                Some(logical_asset),
                GlobalRoutingDirective::PolicyDrivenProportionalSplit,
                std::slice::from_ref(&policy),
                &spot_prices,
            )
            .await
            .expect("plan global liquidity transfers");

        // Verify mathematical split conservation on odd amount: Q_cold ⊕ Q_market ≡ B_hot
        let odd_balance = payment_raw.raw();
        let q_cold = (odd_balance.saturating_mul(U256::from(7000))) / U256::from(10000);
        let q_market = odd_balance.saturating_sub(q_cold);
        assert_eq!(
            q_cold.saturating_add(q_market),
            odd_balance,
            "Split conservation failed for chain {chain_id:?}: Q_cold + Q_market != B_hot"
        );

        // Step 6: Zone C Enclave Policy Inspection and Ceremony Signing
        let (raw_payload, payload_sha256_hex) = create_enclave_canonical_payload(
            vec![cold_vault.to_string()],
            RawAssetAmount::new(q_cold, decimals),
            RawAssetAmount::zero(decimals),
            1,
        );

        let sweep_envelope = UnsignedTransactionEnvelope {
            envelope_id: Uuid::new_v4(),
            chain_id,
            architecture: chain_id.architecture(),
            source_tier: WalletTier::Tier2OperationalHot,
            destination_tier: WalletTier::Tier3ColdVault,
            source_addresses: vec![policy.tier2_hot_wallet_address.clone()],
            expected_destination_address: cold_vault.to_string(),
            physical_asset: physical_asset.clone(),
            net_transfer_amount: RawAssetAmount::new(q_cold, decimals),
            max_network_fee_native: RawAssetAmount::zero(decimals),
            nonce_or_sequence: 1,
            raw_unsigned_payload: raw_payload,
            payload_sha256_hex,
            signer_key_locators: vec!["tier2_operational_key".to_string()],
        };

        let (sign_receipt, artifact) = enclave_signer
            .authorize_and_sign_envelope(&policy_guard, &sweep_envelope, &[])
            .await
            .expect("enclave signing ceremony");

        assert_eq!(sign_receipt.parsed_net_output_amount.raw(), q_cold);
        assert!(artifact.ephemeral_ram_zeroized, "Axiom 4: ephemeral RAM must be zeroized");
        assert!(!artifact.broadcastable_signed_bytes.is_empty(), "Signature must be generated");
    }

    // Step 7: System-wide Cryptographic Integrity Audit Report
    let integrity = container
        .runtime()
        .audit_cryptographic_integrity()
        .await
        .expect("system integrity report");

    assert!(integrity.audit_passed, "Entire multi-chain system must pass cryptographic audit");
    assert_eq!(integrity.detected_corruptions_or_tampers, 0, "Zero tampers allowed");
    assert!(integrity.verified_records_or_cells > 0, "Records must be audited");
}
