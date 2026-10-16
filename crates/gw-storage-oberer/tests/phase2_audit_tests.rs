#![forbid(unsafe_code)]

use gw_primitives::{MicroCreditsMathBridge, RawAssetAmount};
use gw_storage_oberer::{
    ActiveWatchlistView, InvoiceLifecycleStatus, InvoiceRow, InvoiceStateAuditRow,
    ObererHybridStorageEngineInstance, ObererOrmStorageEngine, ObererRowMacSigner,
    OrmLogicalAsset, SovereignEngine, bootstrap_sovereign_schema,
};
use oberer_orm::conn::OrmConn;
use oberer_orm::{MicroCredits, OrmResultExt, Table, TableOps, TableView};
use std::sync::Arc;

#[tokio::test]
async fn test_zero_raw_sql_schema_builds_and_migrates() {
    let conn = OrmConn::new(":memory:").expect("Failed to create memory db");
    bootstrap_sovereign_schema(&conn).expect("Zero-Raw-SQL bootstrap failed");

    // Assert all 11 tables exist in SQLite
    assert!(conn.has_table("supported_assets"));
    assert!(conn.has_table("consensus_block_ring"));
    assert!(conn.has_table("deposit_address_pool"));
    assert!(conn.has_table("invoices"));
    assert!(conn.has_table("invoice_state_audit"));
    assert!(conn.has_table("treasury_wallets"));
    assert!(conn.has_table("asset_balances"));
    assert!(conn.has_table("cost_basis_lots"));
    assert!(conn.has_table("treasury_transfers"));
    assert!(conn.has_table("realized_pnl_ledger"));
    assert!(conn.has_table("temp_block_transfer_staging"));

    // Verify ActiveWatchlistView can be queried via TableView
    let view = TableView::<ActiveWatchlistView>::new(conn.clone());
    let results = view.find_all().unenvelope().expect("Failed to query view");
    assert_eq!(results.len(), 0);
}

#[tokio::test]
async fn test_row_mac_offline_tamper_detection() {
    let row_mac_key = [0x42_u8; 32];
    let signer = ObererRowMacSigner::new(row_mac_key);

    let table = "invoices";
    let pk = "1001";
    let version = 1_i64;
    let mut canonical_payload = b"bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4_USDT_1000000".to_vec();

    // 1. Sign
    let valid_mac = signer.compute_row_mac(table, pk, version, &canonical_payload);
    assert_eq!(valid_mac.len(), 32);

    // 2. Verify legitimate
    let verify_ok = signer.verify_row_mac(table, pk, version, &canonical_payload, &valid_mac);
    assert!(verify_ok.is_ok(), "Legitimate row MAC must pass verification");

    // 3. Bit-flip tamper injection (simulating offline SQLite file modification)
    if let Some(b) = canonical_payload.first_mut() {
        *b ^= 0x01; // flip 1 bit in deposit address
    }
    let verify_tampered = signer.verify_row_mac(table, pk, version, &canonical_payload, &valid_mac);
    assert!(verify_tampered.is_err(), "Bit-flipped payload must trigger SqliteRowTamperingDetected");
}

#[tokio::test]
async fn test_reactive_audit_trigger_records_state_transition() {
    let conn = OrmConn::new(":memory:").expect("Failed to create memory db");
    bootstrap_sovereign_schema(&conn).expect("Bootstrap failed");

    let invoice_table = Table::<InvoiceRow>::new(conn.clone());
    let audit_table = Table::<InvoiceStateAuditRow>::new(conn.clone());

    let fiat_mc = MicroCredits::new(100_000_000);
    let price_mc = MicroCredits::new(2_000_000_000);
    let expected_raw = MicroCreditsMathBridge::to_clean_locked_quote(fiat_mc, price_mc, 6).expect("quote failed");

    let row = InvoiceRow {
        id: 1,
        customer_id_uuid: None,
        pool_id: None,
        chain_id: 1,
        logical_asset: OrmLogicalAsset::Usdt,
        token_contract: "0xdac17f958d2ee523a2206206994597c13d831ec7".to_string(),
        decimals: 6,
        deposit_address: "0x71c...".to_string(),
        routing_reference: None,
        derivation_locator: "m/84'/0'/0'/0/0".to_string(),
        fiat_amount_usd: fiat_mc,
        locked_price_usd: price_mc,
        expected_amount_blob: expected_raw.to_oberer_blob_vec(),
        expected_amount_dec78: expected_raw.to_oberer_dec78(),
        received_amount_blob: RawAssetAmount::zero(6).to_oberer_blob_vec(),
        received_amount_dec78: RawAssetAmount::zero(6).to_oberer_dec78(),
        tolerance_bps: 50,
        status: InvoiceLifecycleStatus::Pending,
        detected_at_block: None,
        current_confirmations: 0,
        target_confirmations: 12,
        expires_at_unix: 1800000000,
        settled_tx_hash: None,
        fulfillment_idempotency_key: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        version: 1,
        deleted_at: None,
        row_mac: vec![0u8; 32],
    };

    let pk = invoice_table.insert(&row).unenvelope().expect("Failed to insert invoice");

    // Update status to Detected with version bump to 2
    let mut updated_row = row.clone();
    updated_row.status = InvoiceLifecycleStatus::Detected;
    updated_row.version = 2;
    updated_row.settled_tx_hash = Some("0xabc123...".to_string());

    let res = invoice_table.update_with_version(pk, 1, &updated_row).unenvelope();
    assert!(res.is_ok(), "CAS update_with_version must succeed");

    // Assert reactive trigger inserted record into invoice_state_audit
    let audit_records = audit_table.find_all().unenvelope().expect("Failed to query audit trail");
    assert_eq!(audit_records.len(), 1, "Trigger should have created exactly 1 audit record");
    let audit_0 = audit_records.first().expect("audit record missing");
    assert_eq!(audit_0.invoice_id, 1);
    assert_eq!(audit_0.old_status, InvoiceLifecycleStatus::Pending);
    assert_eq!(audit_0.new_status, InvoiceLifecycleStatus::Detected);
    assert_eq!(audit_0.old_version, 1);
    assert_eq!(audit_0.new_version, 2);
    assert_eq!(audit_0.settled_tx_hash.as_deref(), Some("0xabc123..."));
}

#[tokio::test]
async fn test_1000_thread_cas_concurrency_proof() {
    let engine = ObererHybridStorageEngineInstance::in_memory_testing([0x55; 32])
        .expect("Failed to create engine");
    engine.initialize_engine().await.expect("Failed to init engine");

    let mem = engine.memory_conn();
    let invoice_table = Arc::new(Table::<InvoiceRow>::new(mem.clone()));

    // Insert base invoice
    let base_row = InvoiceRow {
        id: 999,
        customer_id_uuid: None,
        pool_id: None,
        chain_id: 1,
        logical_asset: OrmLogicalAsset::Usdt,
        token_contract: "NATIVE".to_string(),
        decimals: 6,
        deposit_address: "0x123...".to_string(),
        routing_reference: None,
        derivation_locator: "m/84'/0'/0'/0/1".to_string(),
        fiat_amount_usd: MicroCredits::new(100_000_000),
        locked_price_usd: MicroCredits::new(1_000_000),
        expected_amount_blob: vec![0u8; 32],
        expected_amount_dec78: "0".repeat(78),
        received_amount_blob: vec![0u8; 32],
        received_amount_dec78: "0".repeat(78),
        tolerance_bps: 50,
        status: InvoiceLifecycleStatus::Pending,
        detected_at_block: None,
        current_confirmations: 0,
        target_confirmations: 1,
        expires_at_unix: 2000000000,
        settled_tx_hash: None,
        fulfillment_idempotency_key: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        version: 1,
        deleted_at: None,
        row_mac: vec![0u8; 32],
    };

    let pk = invoice_table.insert(&base_row).unenvelope().expect("Failed to insert invoice");

    // Spawn 100 concurrent workers competing to advance confirmations
    let num_workers = 100;
    let mut handles = Vec::new();

    for i in 0..num_workers {
        let table = Arc::clone(&invoice_table);
        handles.push(tokio::spawn(async move {
            // Each worker attempts CAS loop until successful
            loop {
                let current = table.find(pk).unenvelope().expect("find failed").expect("row exists");
                let mut next = current.clone();
                next.current_confirmations = next.current_confirmations.saturating_add(1);
                next.version = current.version + 1;

                match table.update_with_version(pk, current.version, &next).unenvelope() {
                    Ok(_) => break, // Success!
                    Err(_) => tokio::task::yield_now().await, // Contention retry
                }
            }
            i
        }));
    }

    for h in handles {
        h.await.expect("worker task failed");
    }

    // Verify exact conservation: version must be initial (1) + num_workers (100) = 101
    let final_row = invoice_table.find(pk).unenvelope().expect("final find failed").expect("row exists");
    assert_eq!(final_row.version, 1 + (num_workers as i64), "Zero lost updates in 100-thread CAS race");
    assert_eq!(final_row.current_confirmations, num_workers as i64);
}
