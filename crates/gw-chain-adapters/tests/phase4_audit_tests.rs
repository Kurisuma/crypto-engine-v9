#![forbid(unsafe_code)]

use gw_chain_adapters::{
    calculate_tron_1sun_dust_keeper_sweep, derive_evm_create2_address, derive_evm_create2_raw,
    derive_solana_invoice_pda, derive_sui_deposit_address, derive_tron_create2_address,
    is_ed25519_curve_point, validate_bip84_address, validate_chain_address,
    validate_evm_address, validate_solana_address, validate_sui_address,
    validate_tron_address, Bip84WatchOnlyDeriver, DepositAddressPoolEngine,
    MultiChainWatchOnlyAdapter, TonFeistelMemoCipher,
};
use gw_enterprise_traits::{
    CheckoutDepositRoute, ObererOrmStorageEngine, SovereignEngine, WatchOnlyDerivationEngine,
};
use gw_primitives::{ChainId, LogicalAsset, PhysicalAssetId, RawAssetAmount, SovereignError};
use gw_storage_oberer::ObererHybridStorageEngineInstance;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::task::JoinSet;

fn mock_asset(chain_id: ChainId, logical_asset: LogicalAsset, decimals: u8) -> PhysicalAssetId {
    PhysicalAssetId {
        chain_id,
        logical_asset,
        token_contract: "0xcontract".to_string(),
        decimals,
        keep_one_unit_dust: false,
        target_confirmations: 12,
    }
}

/// 1. BIP-84 Watch-Only Native SegWit Derivation Vector & Homomorphism Test
#[test]
fn test_bip84_derivation_vectors() {
    // SEC1 compressed representation of secp256k1 base point G
    // G = 0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798
    let g_compressed: [u8; 33] = [
        0x02, 0x79, 0xbe, 0x66, 0x7e, 0xf9, 0xdc, 0xbb, 0xac, 0x55, 0xa0, 0x62, 0x95, 0xce, 0x87,
        0x0b, 0x07, 0x02, 0x9b, 0xfc, 0xdb, 0x2d, 0xce, 0x28, 0xd9, 0x59, 0xf2, 0x81, 0x5b, 0x16,
        0xf8, 0x17, 0x98,
    ];
    let chaincode: [u8; 32] = [0x55; 32];

    let deriver = Bip84WatchOnlyDeriver::new(g_compressed, chaincode, "bc")
        .expect("Deriver initialization must succeed for valid G");

    // Derive child indices 0, 1, 2, 100
    for idx in [0_u32, 1, 2, 100, 9999] {
        let (child_pubkey, child_chaincode) = deriver
            .derive_child_pubkey(idx)
            .expect("Non-hardened derivation must succeed");

        assert_eq!(child_pubkey.len(), 33);
        assert_eq!(child_chaincode.len(), 32);
        assert_ne!(child_pubkey, g_compressed, "Child pubkey must not equal parent");

        let address = deriver
            .derive_child_address(idx)
            .expect("Address derivation must succeed");

        assert!(address.starts_with("bc1q"), "BIP-84 must produce P2WPKH bc1q addresses");

        // Validate using bech32 segwit decoder
        let witness_prog = validate_bip84_address(&address, "bc")
            .expect("Address must be valid Bech32 SegWit v0");
        assert_eq!(witness_prog.len(), 20, "P2WPKH witness program must be 20 bytes (Hash160)");
    }

    // Mathematical fencing: Hardened derivation attempt in watch-only mode MUST fail
    let hardened_res = deriver.derive_child_pubkey(0x80000000);
    assert!(hardened_res.is_err(), "Hardened index >= 2^31 must be strictly rejected");
}

/// 2. Deterministic CREATE2 Counterfactual Address Verification (EVM, Tron, Dust Keeper)
#[test]
fn test_create2_factory_bytecode_matches() {
    // Standard EVM CREATE2 test vector
    let factory: [u8; 20] = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    let salt: [u8; 32] = [0x00; 32];
    // Keccak256(b"") = c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470
    let init_code_hash: [u8; 32] = [
        0xc5, 0xd2, 0x46, 0x01, 0x86, 0xf7, 0x23, 0x3c, 0x92, 0x7e, 0x7d, 0xb2, 0xdc, 0xc7, 0x03,
        0xc0, 0xe5, 0x00, 0xb6, 0x53, 0xca, 0x82, 0x27, 0x3b, 0x7b, 0xfa, 0xd8, 0x04, 0x5d, 0x85,
        0xa4, 0x70,
    ];

    let raw_evm = derive_evm_create2_raw(factory, salt, init_code_hash);
    let eip55_addr = derive_evm_create2_address(factory, salt, init_code_hash);

    assert_eq!(eip55_addr.len(), 42);
    assert!(eip55_addr.starts_with("0x"));

    let validated_bytes = validate_evm_address(&eip55_addr).expect("EIP-55 validation must pass");
    assert_eq!(validated_bytes, raw_evm);

    // Tron CREATE2 counterfactual address verification
    let tron_addr = derive_tron_create2_address(factory, salt, init_code_hash);
    assert!(tron_addr.starts_with('T'), "Tron address must start with 'T'");
    let validated_tron = validate_tron_address(&tron_addr).expect("Tron validation must pass");
    assert_eq!(validated_tron.len(), 21);
    assert_eq!(validated_tron[0], 0x41);

    // Tron 1-Sun Dust Keeper: Axiom of residual 1 Sun
    // Current balance: 10,000,000 Sun (10 USDT on Tron)
    let initial_balance = RawAssetAmount::new(alloy_primitives::U256::from(10_000_000_u64), 6);
    let sweep_amount = calculate_tron_1sun_dust_keeper_sweep(initial_balance)
        .expect("1-Sun sweep calculation must succeed");

    // $10,000,000 - 1 = 9,999,999
    assert_eq!(
        sweep_amount.raw(),
        alloy_primitives::U256::from(9_999_999_u64),
        "Dust keeper must leave exactly 1 Sun"
    );
    let residual = initial_balance.raw().saturating_sub(sweep_amount.raw());
    assert_eq!(residual, alloy_primitives::U256::from(1), "Residual must be exactly 1 Sun");
}

/// 3. TON 40-bit Key-Scheduled Feistel Permutation Bijection & Crockford Base32 Fuzzing
#[test]
fn test_ton_feistel_memo_bijection_fuzzing() {
    let master_memo_key: [u8; 32] = [0xA5; 32];
    let cipher = TonFeistelMemoCipher::new(&master_memo_key);

    // Test extreme boundary values: 0, 1, 2^20 - 1, 2^20, 2^40 - 1
    let boundaries: [u64; 6] = [
        0,
        1,
        0x000F_FFFF,
        0x0010_0000,
        0x0000_00FF_FFFF_FFFF,
        1_099_511_627_775, // 2^40 - 1
    ];

    for &val in &boundaries {
        let encrypted = cipher.encrypt_40bit(val);
        let decrypted = cipher.decrypt_40bit(encrypted);
        assert_eq!(
            val, decrypted,
            "Feistel permutation bijection failed for boundary {val}"
        );

        let tag = cipher.encode_memo_tag(val);
        assert_eq!(tag.len(), 8, "Crockford Base32 tag must be exactly 8 chars");
        let decoded = cipher.decode_memo_tag(&tag).expect("Tag decoding must succeed");
        assert_eq!(val, decoded, "Round-trip encode/decode must match for {val}");
    }

    // Pseudo-random fuzzing across 25,000 integers
    let mut state: u64 = 0x123456789ABCDEF0;
    for _ in 0..25_000 {
        // Linear feedback shift step for deterministic fast PRNG
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let invoice_id = state & 0x0000_00FF_FFFF_FFFF;

        let enc = cipher.encrypt_40bit(invoice_id);
        let dec = cipher.decrypt_40bit(enc);
        assert_eq!(
            invoice_id, dec,
            "Bijection F^-1(F(x)) == x violated for invoice {invoice_id}"
        );

        let tag = cipher.encode_memo_tag(invoice_id);
        let decoded = cipher.decode_memo_tag(&tag).expect("Decode memo failed");
        assert_eq!(invoice_id, decoded);
    }

    // Verify Crockford Base32 normalization tolerance ('O' -> '0', 'I'/'L' -> '1')
    let canonical_tag = "01234567";
    let noisy_tag = "Ol234567"; // 'O' instead of '0', 'l' instead of '1'
    let id_canonical = cipher.decode_memo_tag(canonical_tag).expect("Canonical decode");
    let id_noisy = cipher.decode_memo_tag(noisy_tag).expect("Noisy decode");
    assert_eq!(
        id_canonical, id_noisy,
        "Crockford parser must normalize visual confusions"
    );
}

/// 4. Solana Off-Curve PDA Derivation & Rent Reclamation Proof
#[test]
fn test_solana_pda_off_curve_proof() {
    let program_id: [u8; 32] = [0x77; 32];

    // Verify known curve point: Ed25519 Base Point B
    // In compressed little-endian format:
    // y = 4/5 mod (2^255 - 19)
    let mut base_point = [0x66_u8; 32];
    base_point[0] = 0x58;
    assert!(
        is_ed25519_curve_point(&base_point),
        "Ed25519 base point B must be recognized as on-curve"
    );

    // Verify off-curve PDA derivation across invoices
    for invoice_id in [1_i64, 2, 42, 1000, 99999] {
        let (pda_b58, pda_bytes, _bump) = derive_solana_invoice_pda(&program_id, invoice_id)
            .expect("PDA derivation must succeed");

        assert!(!pda_b58.is_empty());

        // Mathematical proof: Candidate PDA MUST NOT lie on Ed25519
        let on_curve = is_ed25519_curve_point(&pda_bytes);
        assert!(
            !on_curve,
            "PDA for invoice {invoice_id} must be strictly off-curve (Axiom 3)"
        );

        let validated = validate_solana_address(&pda_b58).expect("PDA must be valid Base58");
        assert_eq!(validated, pda_bytes);
    }

    // Verify rent reclamation constant: exactly 2,039,280 lamports (~0.00203928 SOL)
    assert_eq!(
        gw_chain_adapters::SOLANA_RENT_EXEMPT_TOKEN_ACCOUNT_LAMPORTS,
        2_039_280
    );
}

/// 5. 100-Worker Address Pool CAS Race & Route Leasing Cooldown Audit
#[tokio::test]
async fn test_pool_leasing_concurrency_race() {
    let row_mac_key = [0x33_u8; 32];
    let storage_inst = ObererHybridStorageEngineInstance::in_memory_testing(row_mac_key)
        .expect("In-memory storage failed");
    storage_inst.initialize_engine().await.expect("Init failed");
    let storage: Arc<dyn ObererOrmStorageEngine> = Arc::new(storage_inst);

    let pool_engine = Arc::new(DepositAddressPoolEngine::new(storage.clone()));
    let chain = ChainId::EthereumMainnet;
    let asset = mock_asset(chain, LogicalAsset::Usdc, 6);

    // Seed pool with exactly 10 addresses
    let mut initial_routes = Vec::new();
    for i in 0..10 {
        initial_routes.push(CheckoutDepositRoute {
            chain_id: chain,
            deposit_address: format!("0x111111111111111111111111111111111111100{i}"),
            routing_reference: None,
            derivation_locator: format!("pool_idx:{i}"),
            counterfactual_commitment_hash: "0xcommithash".to_string(),
        });
    }
    let seeded = pool_engine
        .seed_pool_batch(&initial_routes)
        .await
        .expect("Seed pool failed");
    assert_eq!(seeded, 10);

    // Spawn 100 concurrent workers racing to lease the 10 available addresses
    let mut join_set = JoinSet::new();
    for _ in 0..100 {
        let engine = pool_engine.clone();
        let a = asset.clone();
        join_set.spawn(async move {
            engine.lease_route(chain, &a, 3600).await
        });
    }

    let mut successful_leases = Vec::new();
    let mut exhausted_errors = 0_usize;

    while let Some(res) = join_set.join_next().await {
        match res.expect("Task panicked") {
            Ok(route) => successful_leases.push(route),
            Err(SovereignError::AddressPoolExhausted { .. }) => {
                exhausted_errors = exhausted_errors.saturating_add(1);
            }
            Err(other) => panic!("Unexpected error: {other:?}"),
        }
    }

    // Mathematical conservation of addresses:
    // Exactly 10 successful leases, exactly 90 pool-exhausted errors
    assert_eq!(
        successful_leases.len(),
        10,
        "Exactly 10 addresses must be leased"
    );
    assert_eq!(
        exhausted_errors, 90,
        "Exactly 90 workers must receive AddressPoolExhausted"
    );

    // Mutual exclusion: All 10 leased addresses must be strictly unique!
    let mut seen_addresses = HashSet::new();
    for lease in &successful_leases {
        assert!(
            seen_addresses.insert(lease.deposit_address.clone()),
            "Duplicate lease detected: {}",
            lease.deposit_address
        );
    }

    // Now test Cooldown behavior:
    // Pool is currently 100% leased. Attempting to lease again must fail
    let try_lease_full = pool_engine.lease_route(chain, &asset, 3600).await;
    assert!(matches!(
        try_lease_full,
        Err(SovereignError::AddressPoolExhausted { .. })
    ));

    // Release address 0 to cooldown with a 4-hour window (14,400 seconds)
    let first_lease = successful_leases.first().expect("No leased route");
    let released_addr = &first_lease.deposit_address;
    let released_ok = pool_engine
        .release_route_to_cooldown(chain, released_addr, 14400)
        .await
        .expect("Release to cooldown failed");
    assert!(released_ok);

    // Still in cooldown -> pool is still exhausted
    let try_lease_cooldown = pool_engine.lease_route(chain, &asset, 3600).await;
    assert!(matches!(
        try_lease_cooldown,
        Err(SovereignError::AddressPoolExhausted { .. })
    ));

    // Release address 0 with expired cooldown (cooldown_secs = -10)
    let expired_release = pool_engine
        .release_route_to_cooldown(chain, released_addr, -10)
        .await
        .expect("Expired release failed");
    assert!(expired_release);

    // Now leasing MUST succeed and yield the recycled address
    let recycled_lease = pool_engine
        .lease_route(chain, &asset, 3600)
        .await
        .expect("Recycled lease must succeed");
    assert_eq!(&recycled_lease.deposit_address, released_addr);
}

/// 6. MultiChainWatchOnlyAdapter Full Enterprise Trait Integration
#[tokio::test]
async fn test_multichain_watch_only_adapter_integration() {
    let row_mac_key = [0x55_u8; 32];
    let storage_inst = ObererHybridStorageEngineInstance::in_memory_testing(row_mac_key)
        .expect("In-memory storage failed");
    storage_inst.initialize_engine().await.expect("Init failed");
    let storage: Arc<dyn ObererOrmStorageEngine> = Arc::new(storage_inst);
    let pool_engine = Arc::new(DepositAddressPoolEngine::new(storage.clone()));

    // 1. Ethereum Adapter
    let eth_adapter = MultiChainWatchOnlyAdapter::new(
        ChainId::EthereumMainnet,
        "0xVaultEth0000000000000000000000000000000000",
        pool_engine.clone(),
        [0x11; 32],
    ).with_create2([0x12; 20], [0x34; 32]);

    let asset_eth = mock_asset(ChainId::EthereumMainnet, LogicalAsset::Usdt, 6);
    let route_eth = eth_adapter
        .derive_checkout_route(101, None, &asset_eth, false)
        .await
        .expect("ETH route derivation failed");
    assert_eq!(route_eth.chain_id, ChainId::EthereumMainnet);
    assert!(route_eth.deposit_address.starts_with("0x"));

    // 2. Solana Adapter - Direct Mode vs CEX Mode
    let sol_adapter = MultiChainWatchOnlyAdapter::new(
        ChainId::SolanaMainnet,
        "VaultSol11111111111111111111111111111111111",
        pool_engine.clone(),
        [0x22; 32],
    ).with_solana_program([0x77; 32]);

    let asset_sol = mock_asset(ChainId::SolanaMainnet, LogicalAsset::Sol, 9);
    // Direct Mode: vault address + ephemeral reference
    let route_sol_direct = sol_adapter
        .derive_checkout_route(102, None, &asset_sol, true)
        .await
        .expect("Solana direct route failed");
    assert_eq!(route_sol_direct.deposit_address, "VaultSol11111111111111111111111111111111111");
    assert!(route_sol_direct.routing_reference.is_some());

    // CEX Mode: off-curve PDA
    let route_sol_pda = sol_adapter
        .derive_checkout_route(103, None, &asset_sol, false)
        .await
        .expect("Solana PDA route failed");
    assert_ne!(route_sol_pda.deposit_address, "VaultSol11111111111111111111111111111111111");

    // 3. TON Adapter - Memo routing
    let ton_cipher = TonFeistelMemoCipher::new(&[0xA5; 32]);
    let ton_adapter = MultiChainWatchOnlyAdapter::new(
        ChainId::TonMainnet,
        "EQD__________________________________________0",
        pool_engine.clone(),
        [0x33; 32],
    ).with_ton_memo_cipher(ton_cipher);

    let asset_ton = mock_asset(ChainId::TonMainnet, LogicalAsset::Ton, 9);
    let route_ton = ton_adapter
        .derive_checkout_route(104, None, &asset_ton, false)
        .await
        .expect("TON route failed");
    assert_eq!(route_ton.deposit_address, "EQD__________________________________________0");
    let memo = route_ton.routing_reference.expect("TON route must have memo tag");
    assert_eq!(memo.len(), 8);

    // 4. Sui Move Object Derivation
    let sui_seed: [u8; 32] = [0x99; 32];
    let (sui_addr, sui_bytes) = derive_sui_deposit_address(&sui_seed, 105);
    assert!(sui_addr.starts_with("0x"));
    let validated_sui = validate_sui_address(&sui_addr).expect("Sui address must be valid");
    assert_eq!(validated_sui, sui_bytes);

    // 5. Unified multi-chain validator
    validate_chain_address(ChainId::EthereumMainnet, &route_eth.deposit_address)
        .expect("Unified validator for ETH");
    validate_chain_address(ChainId::SolanaMainnet, &route_sol_pda.deposit_address)
        .expect("Unified validator for SOL");
    validate_chain_address(ChainId::SuiMainnet, &sui_addr)
        .expect("Unified validator for SUI");
}
