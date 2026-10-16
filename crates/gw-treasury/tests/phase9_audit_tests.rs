#![forbid(unsafe_code)]
#![deny(
    clippy::float_arithmetic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::indexing_slicing
)]

use alloy_primitives::U256;
use gw_enterprise_traits::derivation_engine_trait::CheckoutDepositRoute;
use gw_enterprise_traits::treasury_engine_trait::TreasuryPortfolioEngine;
use gw_enterprise_traits::verification_engine_trait::{StateVerifiedComplete, TypestateInvoice};
use gw_enterprise_traits::SovereignEngine;
use gw_primitives::{
    ChainId, LogicalAsset, PhysicalAssetId, RawAssetAmount, SovereignError, WalletTier,
};
use gw_treasury::SovereignTreasuryPortfolioEngine;
use oberer_orm::MicroCredits;
use std::collections::BTreeMap;
use uuid::Uuid;

fn sample_asset(chain_id: ChainId, decimals: u8) -> PhysicalAssetId {
    PhysicalAssetId {
        chain_id,
        logical_asset: match chain_id {
            ChainId::EthereumMainnet => LogicalAsset::Eth,
            ChainId::TronMainnet => LogicalAsset::Trx,
            ChainId::BitcoinMainnet => LogicalAsset::Btc,
            _ => LogicalAsset::Usdt,
        },
        token_contract: "0x0000000000000000000000000000000000000000".to_string(),
        decimals,
        keep_one_unit_dust: false,
        target_confirmations: 12,
    }
}

fn sample_verified_invoice(
    invoice_id: i64,
    received_amount: RawAssetAmount,
    locked_price_mc: MicroCredits,
    settled_at_unix: i64,
) -> TypestateInvoice<StateVerifiedComplete> {
    TypestateInvoice {
        invoice_id,
        customer_id: Some(Uuid::new_v4()),
        asset: sample_asset(ChainId::EthereumMainnet, received_amount.decimals()),
        route: CheckoutDepositRoute {
            chain_id: ChainId::EthereumMainnet,
            deposit_address: "0x1111111111111111111111111111111111111111".to_string(),
            routing_reference: None,
            derivation_locator: "m/44'/60'/0'/0/0".to_string(),
            counterfactual_commitment_hash: "0xabcdef".to_string(),
        },
        fiat_amount_mc: MicroCredits::new(100_000_000), // $100.00
        locked_price_mc,
        expected_amount_raw: received_amount,
        cumulative_received_raw: received_amount,
        tolerance_bps: 50,
        expires_at_unix: settled_at_unix.saturating_add(3600),
        version: 1,
        row_mac: vec![0xaa; 32],
        state: StateVerifiedComplete {
            settlement_tx_hashes: vec!["0xdeadbeef".to_string()],
            settled_at_unix,
            is_overpaid: false,
            surplus_raw: RawAssetAmount::zero(received_amount.decimals()),
            fulfillment_idempotency_token: format!("FULFILL_{}", invoice_id),
        },
    }
}

/// Audit Test 1: Strict Chronological FIFO Lot Slicing & Weighted Cost-Basis Calculus.
#[tokio::test]
async fn test_fifo_chronological_lot_consumption() -> Result<(), Box<dyn std::error::Error>> {
    let engine = SovereignTreasuryPortfolioEngine::new([0x42u8; 32]);
    let asset = sample_asset(ChainId::EthereumMainnet, 18);

    // Lot 1: Acquired at t=1000, 1.0 ETH at $2,000 (2_000_000_000 mc)
    let inv1 = sample_verified_invoice(
        1,
        RawAssetAmount::new(U256::from(1_000_000_000_000_000_000_u64), 18),
        MicroCredits::new(2_000_000_000),
        1000,
    );
    engine.record_verified_invoice_fifo_lot(&inv1).await?;

    // Lot 2: Acquired at t=2000, 2.0 ETH at $2,500 (2_500_000_000 mc)
    let inv2 = sample_verified_invoice(
        2,
        RawAssetAmount::new(U256::from(2_000_000_000_000_000_000_u64), 18),
        MicroCredits::new(2_500_000_000),
        2000,
    );
    engine.record_verified_invoice_fifo_lot(&inv2).await?;

    // Lot 3: Acquired at t=3000, 5.0 ETH at $3,000 (3_000_000_000 mc)
    let inv3 = sample_verified_invoice(
        3,
        RawAssetAmount::new(U256::from(5_000_000_000_000_000_000_u64), 18),
        MicroCredits::new(3_000_000_000),
        3000,
    );
    engine.record_verified_invoice_fifo_lot(&inv3).await?;

    // Disposal: Sell 1.5 ETH at execution price $3,500 (3_500_000_000 mc), network fee $10 (10_000_000 mc)
    // Slices consumed:
    // - 1.0 ETH from Lot 1 (cost: 1.0 * $2,000 = $2,000)
    // - 0.5 ETH from Lot 2 (cost: 0.5 * $2,500 = $1,250)
    // Total Cost Basis = $2,000 + $1,250 = $3,250 (3_250_000_000 mc)
    // Gross Proceeds = 1.5 * $3,500 = $5,250 (5_250_000_000 mc)
    // Net Realized P&L = $5,250 - $3,250 - $10 = $1,990 (1_990_000_000 mc)
    let disposed = RawAssetAmount::new(U256::from(1_500_000_000_000_000_000_u64), 18);
    let exec_price = MicroCredits::new(3_500_000_000);
    let fee = MicroCredits::new(10_000_000);

    let report = engine
        .execute_fifo_lot_consumption(&asset, disposed, exec_price, fee)
        .await?;

    assert_eq!(report.disposed_amount, disposed);
    assert_eq!(report.consumed_lot_slices.len(), 2);
    assert_eq!(report.total_cost_basis_mc.get(), 3_250_000_000);
    assert_eq!(report.gross_proceeds_mc.get(), 5_250_000_000);
    assert_eq!(report.network_fee_burned_mc.get(), 10_000_000);
    assert_eq!(report.net_realized_pnl_microcredits, 1_990_000_000);

    // Verify Remaining Lot Balances
    let active_lots = engine.get_active_lots()?;
    assert_eq!(active_lots.len(), 2, "Lot 1 should be exhausted and excluded");

    let lot2 = active_lots.iter().find(|l| l.lot_id == 2).ok_or("Lot 2 missing")?;
    assert_eq!(
        lot2.remaining_amount.raw(),
        U256::from(1_500_000_000_000_000_000_u64),
        "Lot 2 should have 1.5 ETH remaining"
    );

    let lot3 = active_lots.iter().find(|l| l.lot_id == 3).ok_or("Lot 3 missing")?;
    assert_eq!(
        lot3.remaining_amount.raw(),
        U256::from(5_000_000_000_000_000_000_u64),
        "Lot 3 should have all 5.0 ETH untouched"
    );

    Ok(())
}

/// Audit Test 2: Cryptographic Hash Chain Continuity & Tamper Evidence (Axiom 8).
#[tokio::test]
async fn test_realized_pnl_hash_chain_unbroken() -> Result<(), Box<dyn std::error::Error>> {
    let engine = SovereignTreasuryPortfolioEngine::new([0x33u8; 32]);
    let asset = sample_asset(ChainId::EthereumMainnet, 18);

    // Seed 10 lots of 1.0 ETH each
    for i in 1..=10 {
        let inv = sample_verified_invoice(
            i,
            RawAssetAmount::new(U256::from(1_000_000_000_000_000_000_u64), 18),
            MicroCredits::new(2_000_000_000),
            1000 + i,
        );
        engine.record_verified_invoice_fifo_lot(&inv).await?;
    }

    // Execute 10 separate disposals of 0.5 ETH each
    let slice_amount = RawAssetAmount::new(U256::from(500_000_000_000_000_000_u64), 18);
    for _ in 0..10 {
        engine
            .execute_fifo_lot_consumption(
                &asset,
                slice_amount,
                MicroCredits::new(2_500_000_000),
                MicroCredits::new(5_000_000),
            )
            .await?;
    }

    let (len, is_valid) = engine.verify_ledger_hash_chain()?;
    assert_eq!(len, 10);
    assert!(is_valid, "Ledger hash chain must be 100% unbroken");

    let integrity = engine.audit_cryptographic_integrity().await?;
    assert!(integrity.audit_passed);
    assert_eq!(integrity.detected_corruptions_or_tampers, 0);

    Ok(())
}

/// Audit Test 3: Mathematical FIFO Conservation Law Proof under Mixed Disposals.
#[tokio::test]
async fn test_portfolio_conservation_under_random_churn() -> Result<(), Box<dyn std::error::Error>> {
    let engine = SovereignTreasuryPortfolioEngine::new([0x55u8; 32]);
    let asset = sample_asset(ChainId::EthereumMainnet, 18);

    let mut total_injected = U256::ZERO;
    let mut total_disposed = U256::ZERO;

    // Inject 5 lots with varied amounts
    let amounts = [
        100_000_000_000_000_000_u64, // 0.1 ETH
        250_000_000_000_000_000_u64, // 0.25 ETH
        500_000_000_000_000_000_u64, // 0.5 ETH
        150_000_000_000_000_000_u64, // 0.15 ETH
        1_000_000_000_000_000_000_u64, // 1.0 ETH
    ];

    for (idx, &amt) in amounts.iter().enumerate() {
        let raw = U256::from(amt);
        total_injected += raw;

        let inv = sample_verified_invoice(
            (idx as i64) + 1,
            RawAssetAmount::new(raw, 18),
            MicroCredits::new(2_000_000_000),
            1000 + (idx as i64),
        );
        engine.record_verified_invoice_fifo_lot(&inv).await?;
    }

    // Execute 3 partial disposals
    let disposal_amounts = [
        200_000_000_000_000_000_u64, // 0.2 ETH
        400_000_000_000_000_000_u64, // 0.4 ETH
        100_000_000_000_000_000_u64, // 0.1 ETH
    ];

    for &amt in &disposal_amounts {
        let raw = U256::from(amt);
        total_disposed += raw;

        engine
            .execute_fifo_lot_consumption(
                &asset,
                RawAssetAmount::new(raw, 18),
                MicroCredits::new(2_500_000_000),
                MicroCredits::new(1_000_000),
            )
            .await?;
    }

    // Verify Mathematical Conservation Law: sum(remaining) + sum(disposed) == sum(injected)
    let active_lots = engine.get_active_lots()?;
    let sum_remaining: U256 = active_lots.iter().map(|l| l.remaining_amount.raw()).sum();

    assert_eq!(
        sum_remaining + total_disposed,
        total_injected,
        "FIFO Conservation Law violated: sum(remaining) + sum(disposed) must strictly equal sum(injected)"
    );

    Ok(())
}

/// Audit Test 4: Unified Multi-Network Portfolio Rollup & Exact Unrealized P&L Calculus.
#[tokio::test]
async fn test_unified_portfolio_rollup_and_unrealized_pnl() -> Result<(), Box<dyn std::error::Error>> {
    let engine = SovereignTreasuryPortfolioEngine::new([0x66u8; 32]);

    // Register a Cold Vault
    engine.register_wallet(
        1,
        WalletTier::Tier3ColdVault,
        ChainId::EthereumMainnet,
        "0x1111111111111111111111111111111111111111",
    )?;

    // Invoice 1: 2.0 ETH at $2,000 (Cost basis = $4,000 = 4_000_000_000 mc)
    let inv1 = sample_verified_invoice(
        1,
        RawAssetAmount::new(U256::from(2_000_000_000_000_000_000_u64), 18),
        MicroCredits::new(2_000_000_000),
        1000,
    );
    engine.record_verified_invoice_fifo_lot(&inv1).await?;

    // Invoice 2: 1.0 ETH at $2,500 (Cost basis = $2,500 = 2_500_000_000 mc)
    let inv2 = sample_verified_invoice(
        2,
        RawAssetAmount::new(U256::from(1_000_000_000_000_000_000_u64), 18),
        MicroCredits::new(2_500_000_000),
        2000,
    );
    engine.record_verified_invoice_fifo_lot(&inv2).await?;

    // Spot price: ETH = $3,000 (3_000_000_000 mc)
    // Holdings: 3.0 ETH
    // Total Market Valuation = 3.0 * $3,000 = $9,000 (9_000_000_000 mc)
    // Total FIFO Cost Basis = $4,000 + $2,500 = $6,500 (6_500_000_000 mc)
    // Exact Unrealized P&L = $9,000 - $6,500 = +$2,500 (+2_500_000_000 mc)
    let mut spot_prices = BTreeMap::new();
    spot_prices.insert(LogicalAsset::Eth, MicroCredits::new(3_000_000_000));

    let portfolio = engine.compute_unified_global_portfolio(&spot_prices).await?;
    let eth_summary = portfolio.get(&LogicalAsset::Eth).ok_or("ETH portfolio missing")?;

    assert_eq!(
        eth_summary.total_canonical_18_units,
        U256::from(3_000_000_000_000_000_000_u64)
    );
    assert_eq!(eth_summary.total_market_valuation_mc.get(), 9_000_000_000);
    assert_eq!(eth_summary.total_fifo_cost_basis_mc.get(), 6_500_000_000);
    assert_eq!(eth_summary.unrealized_pnl_microcredits, 2_500_000_000);

    // Verify Tier 3 Cold Vault valuation mapping
    let cold_val = eth_summary
        .valuation_by_tier_mc
        .get(&WalletTier::Tier3ColdVault)
        .ok_or("Tier 3 valuation missing")?;
    assert_eq!(cold_val.get(), 9_000_000_000);

    Ok(())
}

/// Audit Test 5: Insufficient Lot Inventory Blocks Disposal (Underflow Defense).
#[tokio::test]
async fn test_insufficient_lots_blocks_disposal() -> Result<(), Box<dyn std::error::Error>> {
    let engine = SovereignTreasuryPortfolioEngine::new([0x77u8; 32]);
    let asset = sample_asset(ChainId::EthereumMainnet, 18);

    // Only 1.0 ETH in inventory
    let inv = sample_verified_invoice(
        1,
        RawAssetAmount::new(U256::from(1_000_000_000_000_000_000_u64), 18),
        MicroCredits::new(2_000_000_000),
        1000,
    );
    engine.record_verified_invoice_fifo_lot(&inv).await?;

    // Attempt to dispose 2.0 ETH -> must fail with Underflow
    let excess_disposal = RawAssetAmount::new(U256::from(2_000_000_000_000_000_000_u64), 18);
    let res = engine
        .execute_fifo_lot_consumption(
            &asset,
            excess_disposal,
            MicroCredits::new(2_500_000_000),
            MicroCredits::new(10_000_000),
        )
        .await;

    match res {
        Err(SovereignError::Underflow { context }) => {
            assert!(
                context.contains("insufficient_open_lots"),
                "Expected insufficient_open_lots underflow context, got {context}"
            );
        }
        other => panic!("Expected SovereignError::Underflow, got {other:?}"),
    }

    Ok(())
}
