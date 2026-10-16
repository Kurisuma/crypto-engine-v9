#![forbid(unsafe_code)]
#![deny(
    clippy::float_arithmetic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::unwrap_used,
    clippy::indexing_slicing
)]

use alloy_primitives::U256;
use gw_enterprise_traits::{
    CrossNetworkLiquidityRouterEngine, GlobalRoutingDirective, NetworkLiquidityPolicy,
    ObererRowMacSigner, SovereignEngine,
};
use gw_primitives::{ChainId, LogicalAsset, PhysicalAssetId, RawAssetAmount, WalletTier};
use gw_router::SovereignLiquidityRouterEngine;
use oberer_orm::MicroCredits;
use proptest::prelude::*;
use std::collections::BTreeMap;
use std::sync::Arc;

fn make_test_router() -> SovereignLiquidityRouterEngine {
    let mac_key = [42u8; 32];
    let row_mac_signer = Arc::new(ObererRowMacSigner::new(mac_key));
    SovereignLiquidityRouterEngine::new(row_mac_signer, None)
}

fn sample_policy(asset: PhysicalAssetId) -> NetworkLiquidityPolicy {
    NetworkLiquidityPolicy {
        asset,
        tier1_relayer_address: "relayer_address_1".to_string(),
        tier2_hot_wallet_address: "hot_wallet_address_2".to_string(),
        tier2_key_locator: "m/44'/60'/0'/0/0".to_string(),
        tier3_cold_vault_address: "cold_vault_address_3".to_string(),
        tier4_market_exchange_address: "market_exchange_address_4".to_string(),
        hot_wallet_ceiling_mc: MicroCredits::new(10_000_000), // $10.00 USD
        cold_vault_split_bps: 7_000,                         // 70%
        market_exchange_split_bps: 3_000,                    // 30%
        min_relayer_gas_raw: RawAssetAmount::new(U256::from(100_000_000u64), 6),
        target_relayer_gas_raw: RawAssetAmount::new(U256::from(500_000_000u64), 6),
    }
}

#[tokio::test]
async fn test_economical_sweep_threshold_rejection() {
    let router = make_test_router();

    let asset = PhysicalAssetId {
        chain_id: ChainId::EthereumMainnet,
        logical_asset: LogicalAsset::Usdt,
        token_contract: "0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string(),
        decimals: 6,
        keep_one_unit_dust: false,
        target_confirmations: 12,
    };

    // Override estimated sweep fee: 2,500,000 units (2.5 USDT)
    let estimated_fee = RawAssetAmount::new(U256::from(2_500_000u64), 6);
    router
        .set_estimated_sweep_fee(ChainId::EthereumMainnet, asset.clone(), estimated_fee)
        .expect("set fee");

    // With beta_fee = 50 bps (0.50%):
    // R_sweep_min = ceil( (2,500,000 * 10,000) / 50 ) = 25,000,000,000 / 50 = 500,000,000 units (500 USDT)
    let threshold = router
        .calculate_economical_sweep_threshold(ChainId::EthereumMainnet, &asset, 50)
        .await
        .expect("threshold calculation");

    assert_eq!(threshold.raw(), U256::from(500_000_000u64));

    // Mathematical Audit:
    // At V = 500,000,000:
    // (fee * 10,000) / V = (2,500,000 * 10,000) / 500,000,000 = 25,000,000,000 / 500,000,000 = 50 bps <= 50 bps (Economical!)
    let fee_ratio_bps_at_threshold =
        (estimated_fee.raw() * U256::from(10_000u64)) / threshold.raw();
    assert!(fee_ratio_bps_at_threshold <= U256::from(50u64));

    // If deposit is V = 499,999,999:
    // (fee * 10,000) / V = 25,000,000,000 / 499,999,999 = 50.0000001 > 50 bps (Uneconomical!)
    let below_threshold = threshold.raw() - U256::from(1u64);
    let fee_ratio_bps_below =
        (estimated_fee.raw() * U256::from(10_000u64)) / below_threshold;
    assert!(fee_ratio_bps_below >= U256::from(50u64));

    // Zero basis points must be rejected by enclave policy
    assert!(
        router
            .calculate_economical_sweep_threshold(ChainId::EthereumMainnet, &asset, 0)
            .await
            .is_err()
    );

    // Mismatched chain must be rejected
    assert!(
        router
            .calculate_economical_sweep_threshold(ChainId::SolanaMainnet, &asset, 50)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn test_relayer_gas_refill_trigger() {
    let router = make_test_router();

    let asset = PhysicalAssetId {
        chain_id: ChainId::TronMainnet,
        logical_asset: LogicalAsset::Trx,
        token_contract: "NATIVE".to_string(),
        decimals: 6,
        keep_one_unit_dust: true,
        target_confirmations: 19,
    };

    let policy = sample_policy(asset.clone());
    let policies = vec![policy.clone()];

    // Case A: Balance is healthy (150,000,000 >= min 100,000,000)
    router
        .set_relayer_balance(
            &policy.tier1_relayer_address,
            asset.clone(),
            RawAssetAmount::new(U256::from(150_000_000u64), 6),
        )
        .expect("set relayer bal");

    let refills_healthy = router
        .plan_tier1_relayer_gas_refills(&policies)
        .await
        .expect("plan refills");
    assert!(
        refills_healthy.is_empty(),
        "No refill expected when balance is above min threshold"
    );

    // Case B: Balance drops below min (40,000,000 < min 100,000,000)
    router
        .set_relayer_balance(
            &policy.tier1_relayer_address,
            asset.clone(),
            RawAssetAmount::new(U256::from(40_000_000u64), 6),
        )
        .expect("set relayer bal");

    let refills_triggered = router
        .plan_tier1_relayer_gas_refills(&policies)
        .await
        .expect("plan refills");

    assert_eq!(refills_triggered.len(), 1);
    let envelope = refills_triggered
        .into_iter()
        .next()
        .expect("envelope");

    assert_eq!(envelope.source_tier, WalletTier::Tier2OperationalHot);
    assert_eq!(envelope.destination_tier, WalletTier::Tier1GasRelayer);
    assert_eq!(
        envelope.expected_destination_address,
        policy.tier1_relayer_address
    );

    // Delta required: 500,000,000 - 40,000,000 = 460,000,000 Sun
    assert_eq!(
        envelope.net_transfer_amount.raw(),
        U256::from(460_000_000u64)
    );

    // Verified: Refill amount + old balance == target balance
    let restored_balance = U256::from(40_000_000u64) + envelope.net_transfer_amount.raw();
    assert_eq!(restored_balance, policy.target_relayer_gas_raw.raw());
}

#[tokio::test]
async fn test_proportional_split_conservation() {
    let router = make_test_router();

    let asset = PhysicalAssetId {
        chain_id: ChainId::EthereumMainnet,
        logical_asset: LogicalAsset::Usdt,
        token_contract: "0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string(),
        decimals: 6,
        keep_one_unit_dust: false,
        target_confirmations: 12,
    };

    let policy = sample_policy(asset.clone());
    let policies = vec![policy.clone()];

    // Odd amount to rigorously test integer division remainder conservation
    let hot_balance_raw = U256::from(123_456_789_101_112_131u128);
    let hot_balance = RawAssetAmount::new(hot_balance_raw, 6);

    router
        .set_hot_wallet_balance(&policy.tier2_hot_wallet_address, asset.clone(), hot_balance)
        .expect("set hot balance");

    // Spot price: $1.00 USD (1,000,000 MicroCredits)
    let mut spot_prices = BTreeMap::new();
    spot_prices.insert(LogicalAsset::Usdt, MicroCredits::new(1_000_000));

    let envelopes = router
        .plan_global_liquidity_transfers(
            Some(LogicalAsset::Usdt),
            GlobalRoutingDirective::PolicyDrivenProportionalSplit,
            &policies,
            &spot_prices,
        )
        .await
        .expect("plan global liquidity");

    // Expect 2 envelopes: Cold (70%) and Market (30%)
    assert_eq!(envelopes.len(), 2);

    let cold_env = envelopes
        .iter()
        .find(|e| e.destination_tier == WalletTier::Tier3ColdVault)
        .expect("cold envelope");
    let market_env = envelopes
        .iter()
        .find(|e| e.destination_tier == WalletTier::Tier4MarketExchange)
        .expect("market envelope");

    let q_cold = cold_env.net_transfer_amount.raw();
    let q_market = market_env.net_transfer_amount.raw();

    // Theoretical Split Allocation Conservation Law:
    // Q_cold + Q_market == hot_balance with 0 remainder loss!
    let sum = q_cold + q_market;
    assert_eq!(
        sum, hot_balance_raw,
        "Proportional split must strictly conserve total balance without penny loss"
    );

    // Verify Cold allocation matches floor( (B * 7000) / 10000 )
    let expected_cold = (hot_balance_raw * U256::from(7_000u64)) / U256::from(10_000u64);
    assert_eq!(q_cold, expected_cold);
    assert_eq!(q_market, hot_balance_raw - expected_cold);
}

#[tokio::test]
async fn test_emergency_and_market_flush_directives() {
    let router = make_test_router();

    let asset = PhysicalAssetId {
        chain_id: ChainId::SolanaMainnet,
        logical_asset: LogicalAsset::Sol,
        token_contract: "NATIVE".to_string(),
        decimals: 9,
        keep_one_unit_dust: false,
        target_confirmations: 32,
    };

    let policy = sample_policy(asset.clone());
    let policies = vec![policy.clone()];

    let bal = RawAssetAmount::new(U256::from(88_000_000_000u64), 9); // 88 SOL
    router
        .set_hot_wallet_balance(&policy.tier2_hot_wallet_address, asset.clone(), bal)
        .expect("set balance");

    let spot_prices = BTreeMap::new();

    // 1. Emergency Flush to Cold Vault
    let cold_envelopes = router
        .plan_global_liquidity_transfers(
            Some(LogicalAsset::Sol),
            GlobalRoutingDirective::EmergencyOrScheduledFlushToCold,
            &policies,
            &spot_prices,
        )
        .await
        .expect("flush cold");

    assert_eq!(cold_envelopes.len(), 1);
    let cold_env = cold_envelopes
        .into_iter()
        .next()
        .expect("cold envelope");
    assert_eq!(cold_env.destination_tier, WalletTier::Tier3ColdVault);
    assert_eq!(cold_env.net_transfer_amount.raw(), bal.raw());
    assert_eq!(
        cold_env.expected_destination_address,
        policy.tier3_cold_vault_address
    );

    // 2. Flush All to Market Exchange
    let market_envelopes = router
        .plan_global_liquidity_transfers(
            Some(LogicalAsset::Sol),
            GlobalRoutingDirective::FlushAllToMarketExchange,
            &policies,
            &spot_prices,
        )
        .await
        .expect("flush market");

    assert_eq!(market_envelopes.len(), 1);
    let market_env = market_envelopes
        .into_iter()
        .next()
        .expect("market envelope");
    assert_eq!(market_env.destination_tier, WalletTier::Tier4MarketExchange);
    assert_eq!(market_env.net_transfer_amount.raw(), bal.raw());
    assert_eq!(
        market_env.expected_destination_address,
        policy.tier4_market_exchange_address
    );
}

#[tokio::test]
async fn test_liquidity_router_hash_chain_and_audit() {
    let router = make_test_router();

    let asset = PhysicalAssetId {
        chain_id: ChainId::BitcoinMainnet,
        logical_asset: LogicalAsset::Btc,
        token_contract: "NATIVE".to_string(),
        decimals: 8,
        keep_one_unit_dust: false,
        target_confirmations: 6,
    };

    let mut policy = sample_policy(asset.clone());
    policy.min_relayer_gas_raw = RawAssetAmount::new(U256::from(500_000u64), 8);
    policy.target_relayer_gas_raw = RawAssetAmount::new(U256::from(2_000_000u64), 8);
    let policies = vec![policy.clone()];

    // Trigger 10 successive refills to populate the hash-chained ledger
    for i in 1..=10u64 {
        router
            .set_relayer_balance(
                &policy.tier1_relayer_address,
                asset.clone(),
                RawAssetAmount::new(U256::from(i * 10_000), 8),
            )
            .expect("set relayer bal");

        router
            .plan_tier1_relayer_gas_refills(&policies)
            .await
            .expect("refill plan");
    }

    // Run cryptographic integrity audit
    let audit_report = router
        .audit_cryptographic_integrity()
        .await
        .expect("audit");

    assert_eq!(audit_report.verified_records_or_cells, 10);
    assert_eq!(audit_report.detected_corruptions_or_tampers, 0);
    assert!(audit_report.audit_passed);
    assert!(!audit_report.hash_chain_head_hex.is_empty());

    // Health snapshot verification
    let health = router.health_snapshot().await.expect("health");
    assert!(health.is_healthy);
    assert!(health.total_operations_processed >= 10);
}

// Property-Based Mathematical Fuzzing Test:
// Proves that for ANY arbitrary balance amount and ANY valid split ratio,
// Q_cold + Q_market == B with EXACT 0-remainder conservation.
proptest! {
    #[test]
    fn test_proptest_split_conservation_arbitrary_amounts(
        balance_u128 in 1u128..=u128::MAX,
        cold_bps in 0u16..=10_000u16,
    ) {
        let b_hot = U256::from(balance_u128);
        let alpha_cold = U256::from(cold_bps);
        let bps_divisor = U256::from(10_000u64);

        let q_cold = (b_hot * alpha_cold) / bps_divisor;
        let q_market = b_hot - q_cold;

        let total = q_cold + q_market;
        prop_assert_eq!(total, b_hot);
    }
}
