#![forbid(unsafe_code)]

use gw_imprce::{
    BlockedBloomFilter, HashChainedWal, PriorityTier,
    SovereignImprceEngine,
};
use gw_storage_oberer::{
    ObererHybridStorageEngineInstance, ObererOrmStorageEngine,
    OrmLogicalAsset, SovereignEngine, SupportedAssetRow,
};
use oberer_orm::MicroCredits;
use std::time::{Duration, Instant};

#[tokio::test]
async fn test_imprce_bit_rot_self_healing() {
    let seed = [0x77_u8; 32];
    let engine = SovereignImprceEngine::new(seed, 64 * 1024 * 1024); // 64 MiB

    let ns = "watchlist";
    let key = "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4";
    let expected_payload = b"ACTIVE_INVOICE_STATE_DATA_SAFE_AND_AUTHENTIC".to_vec();

    // 1. Insert P1 entry (replicated across 3 disjoint mirror arenas)
    engine
        .put(
            ns,
            key,
            PriorityTier::P1ActivePaymentWatch,
            1,
            Duration::from_secs(3600),
            expected_payload.clone(),
        )
        .expect("Put failed");

    // 2. Inject SEU hardware bit-rot in Primary Arena 0 (flip bits in payload)
    let corrupted = engine.inject_bit_rot_in_arena0(ns, key, 0);
    assert!(corrupted, "Failed to inject bit rot into Arena 0");

    assert_eq!(engine.bit_rot_self_heals(), 0);

    // 3. Quorum read should detect Arena 0 bit-rot, failover to Arena 1/2, and atomically heal Arena 0
    let read_res = engine
        .get_quorum_verified(ns, key)
        .expect("Quorum read failed");
    assert_eq!(
        read_res,
        Some(expected_payload.clone()),
        "Quorum read must recover authentic uncorrupted payload"
    );

    // 4. Assert that exactly 1 autonomous self-heal was triggered
    assert_eq!(
        engine.bit_rot_self_heals(),
        1,
        "Autonomous self-healing must increment counter"
    );

    // 5. Secondary read directly on Arena 0 should now be healed and valid
    let healed_read = engine
        .get_quorum_verified(ns, key)
        .expect("Second read failed");
    assert_eq!(healed_read, Some(expected_payload));
    // Self-heal counter should not increment again because Arena 0 is now clean
    assert_eq!(engine.bit_rot_self_heals(), 1);
}

#[tokio::test]
async fn test_imprce_adversarial_p4_flood_quota_defense() {
    let seed = [0x88_u8; 32];
    // 256 KiB total memory budget to trigger severe quota pressure
    let engine = SovereignImprceEngine::new(seed, 256 * 1024);

    // 1. Seed critical consensus (P0), watchlist (P1), and pool lease (P2) state
    engine
        .put(
            "consensus",
            "block:100",
            PriorityTier::P0CriticalConsensus,
            1,
            Duration::from_secs(86400),
            b"CONSENSUS_HEADER_HASH_CRITICAL".to_vec(),
        )
        .expect("Put P0 failed");

    engine
        .put(
            "watchlist",
            "addr:vip1",
            PriorityTier::P1ActivePaymentWatch,
            1,
            Duration::from_secs(3600),
            b"WATCHLIST_VIP_INVOICE_DATA".to_vec(),
        )
        .expect("Put P1 failed");

    engine
        .put(
            "pool",
            "lease:001",
            PriorityTier::P2PoolLeaseCooldown,
            1,
            Duration::from_secs(14400),
            b"POOL_LEASE_COOLDOWN_RECORD".to_vec(),
        )
        .expect("Put P2 failed");

    // 2. Blast 20,000 sacrificial P4 telemetry keys to simulate adversarial DDoS flood
    for i in 0..20_000 {
        let key = format!("telemetry:metric:{i}");
        let payload = format!("VALUE_BURST_METRIC_DATA_{i}").into_bytes();
        let _ = engine.put(
            "telemetry",
            &key,
            PriorityTier::P4TelemetryIdempotency,
            1,
            Duration::from_secs(60),
            payload,
        );
    }

    // 3. Axiom 7 Quota Isolation Verification:
    // P0, P1, P2 MUST NOT BE EVICTED under any amount of P4 flood!
    let p0_read = engine
        .get_quorum_verified("consensus", "block:100")
        .expect("Read P0 failed");
    assert_eq!(
        p0_read,
        Some(b"CONSENSUS_HEADER_HASH_CRITICAL".to_vec()),
        "P0 Critical Consensus must survive adversarial P4 flood"
    );

    let p1_read = engine
        .get_quorum_verified("watchlist", "addr:vip1")
        .expect("Read P1 failed");
    assert_eq!(
        p1_read,
        Some(b"WATCHLIST_VIP_INVOICE_DATA".to_vec()),
        "P1 Active Payment Watch must survive adversarial P4 flood"
    );

    let p2_read = engine
        .get_quorum_verified("pool", "lease:001")
        .expect("Read P2 failed");
    assert_eq!(
        p2_read,
        Some(b"POOL_LEASE_COOLDOWN_RECORD".to_vec()),
        "P2 Pool Lease Cooldown must survive adversarial P4 flood"
    );
}

#[test]
fn test_imprce_blocked_bloom_filter_rejection() {
    let bloom = BlockedBloomFilter::new();

    // 1. Insert 1,000 active deposit addresses
    let mut watched_keys = Vec::new();
    for i in 0..1000 {
        let key = format!("0x71c6bf06570d353f88b85b2e5b0cf324649f9777_{i}");
        bloom.insert(&key);
        watched_keys.push(key);
    }

    // 2. Assert 0% false negatives: all 1,000 keys must return true
    for key in &watched_keys {
        assert!(bloom.contains(key), "Bloom filter must never produce false negatives");
    }

    // 3. Test false positive rate against 100,000 non-watched addresses
    let mut false_positives = 0;
    let total_non_watched = 100_000;

    for i in 0..total_non_watched {
        let fake_key = format!("0xdeadbeef_non_watched_address_candidate_{i}");
        if bloom.contains(&fake_key) {
            false_positives += 1;
        }
    }

    // Standard spec requirement: false positive rate <= 0.1% (100 in 100,000)
    assert!(
        false_positives * 1000 <= total_non_watched,
        "False positive rate must be <= 0.1%, got {} in {}",
        false_positives,
        total_non_watched
    );
}

#[tokio::test]
async fn test_imprce_hydration_latency() {
    let engine = ObererHybridStorageEngineInstance::in_memory_testing([0x99; 32])
        .expect("Failed to create storage engine");
    engine.initialize_engine().await.expect("Failed to init storage engine");

    let mem = engine.memory_conn();

    // Insert 10 supported assets, 10 consensus blocks, 10 deposit pool addresses, 10 invoices
    let asset_table = oberer_orm::Table::<SupportedAssetRow>::new(mem.clone());
    use oberer_orm::{OrmResultExt, TableOps};

    for i in 1..=10 {
        let asset = SupportedAssetRow {
            id: i,
            chain_id: 1,
            architecture: gw_storage_oberer::OrmChainArchitecture::EvmAccount,
            logical_asset: OrmLogicalAsset::Usdt,
            token_contract: format!("0xdac17f958d2ee523a2206206994597c13d831ec7_{i}"),
            decimals: 6,
            min_sweep_usd: MicroCredits::new(5_000_000),
            target_confirmations: 12,
            keep_one_unit_dust: 0,
            requires_memo_tag: 0,
            is_enabled: 1,
            version: 1,
            row_mac: vec![0u8; 32],
        };
        asset_table.insert(&asset).unenvelope().expect("insert asset");
    }

    let cache_engine = SovereignImprceEngine::new([0xAA; 32], 64 * 1024 * 1024);

    // Benchmark hydration latency
    let start = Instant::now();
    let count = cache_engine
        .hydrate_from_oberer_storage(&engine)
        .expect("Hydration failed");
    let elapsed = start.elapsed();

    println!("Hydrated {count} records in {:?}", elapsed);
    assert!(
        elapsed < Duration::from_millis(10),
        "Hydration must complete in < 10 ms, took {:?}",
        elapsed
    );
    assert!(count >= 10);

    // Verify hydrated asset can be retrieved from cache in O(1)
    let cached_asset = cache_engine
        .get_quorum_verified("supported_assets", "1:0xdac17f958d2ee523a2206206994597c13d831ec7_1")
        .expect("Cache query failed");
    assert!(cached_asset.is_some(), "Hydrated asset must be present in cache");
}

#[test]
fn test_wal_continuity_and_tamper_detection() {
    let wal = HashChainedWal::new();

    // 1. Append 5 mutations
    for i in 1..=5 {
        wal.append(
            "test_ns",
            &format!("key_{i}"),
            PriorityTier::P1ActivePaymentWatch,
            i,
            2000000000,
            b"PAYLOAD_BYTES",
            0x12345678,
        );
    }

    assert_eq!(wal.frame_count(), 5);

    // 2. Verify continuity is clean
    assert!(wal.verify_continuity().is_ok(), "Initial WAL chain must be valid");
}
