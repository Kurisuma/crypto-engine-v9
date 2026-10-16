#![forbid(unsafe_code)]

use alloy_primitives::U256;
use gw_primitives::{
    MicroCreditsMathBridge, ObererU256Codec, RawAssetAmount, SovereignError,
};
use oberer_orm::MicroCredits;
use proptest::prelude::*;

#[test]
fn test_u256_codec_roundtrip_boundaries() {
    let test_vectors = [
        U256::ZERO,
        U256::from(1_u64),
        U256::from(u64::MAX),
        U256::from(u128::MAX),
        U256::MAX - U256::from(1_u64),
        U256::MAX,
    ];

    for val in test_vectors {
        // 1. BLOB(32) Roundtrip
        let blob = ObererU256Codec::to_be_blob_vec(val);
        assert_eq!(blob.len(), 32, "BLOB encoding must be exactly 32 bytes");
        let decoded_blob = ObererU256Codec::from_be_blob_slice(&blob).expect("Failed to decode blob");
        assert_eq!(val, decoded_blob, "BLOB roundtrip failed for {val}");

        // 2. TEXT(78) Roundtrip
        let text = ObererU256Codec::to_padded_dec78(val);
        assert_eq!(text.len(), 78, "DEC78 encoding must be exactly 78 characters");
        let decoded_text = ObererU256Codec::from_padded_dec78(&text).expect("Failed to decode text");
        assert_eq!(val, decoded_text, "DEC78 roundtrip failed for {val}");
    }
}

#[test]
fn test_canonical_18_decimal_scaling() {
    // 6-decimal USDC: 1,000,000 (1 USDC) -> 10^18 in 18-decimal canonical units
    let usdc_1 = RawAssetAmount::new(U256::from(1_000_000_u64), 6);
    let canonical = usdc_1.to_canonical_18().expect("canonical 18 conversion failed");
    assert_eq!(canonical, U256::from(10_u64).pow(U256::from(18_u8)));

    // 18-decimal ETH: 10^18 -> 10^18
    let eth_1 = RawAssetAmount::new(U256::from(10_u64).pow(U256::from(18_u8)), 18);
    assert_eq!(eth_1.to_canonical_18().expect("18-decimal failed"), eth_1.raw());

    // 8-decimal BTC: 100,000,000 (1 BTC) -> 10^18
    let btc_1 = RawAssetAmount::new(U256::from(100_000_000_u64), 8);
    assert_eq!(btc_1.to_canonical_18().expect("8-decimal failed"), U256::from(10_u64).pow(U256::from(18_u8)));
}

#[test]
fn test_dual_oracle_circuit_breaker_boundary() {
    // Base mid-price: $2,000.00 = 2,000,000,000 microcredits
    let base_mc = MicroCredits::new(2_000_000_000);

    // 149 bps divergence: 149 / 10,000 * 2,000,000,000 = 29,800,000
    // Price = 2,029,800,000
    let p_149_bps = MicroCredits::new(2_029_800_000);
    let res_149 = MicroCreditsMathBridge::verify_dual_oracle_microcredits(base_mc, p_149_bps, 150);
    assert!(res_149.is_ok(), "149 bps should be accepted");

    // 150 bps divergence: 150 / 10,000 * 2,000,000,000 = 30,000,000
    // Price = 2,030,000,000
    let p_150_bps = MicroCredits::new(2_030_000_000);
    let res_150 = MicroCreditsMathBridge::verify_dual_oracle_microcredits(base_mc, p_150_bps, 150);
    assert!(res_150.is_ok(), "150 bps boundary should be accepted");

    // 151 bps divergence: 151 / 10,000 * 2,000,000,000 = 30,200,000
    // Price = 2,030_200_000
    let p_151_bps = MicroCredits::new(2_030_200_000);
    let res_151 = MicroCreditsMathBridge::verify_dual_oracle_microcredits(base_mc, p_151_bps, 150);
    match res_151 {
        Err(SovereignError::OracleDivergenceCircuitBreaker { divergence_bps, max_bps }) => {
            assert_eq!(divergence_bps, 151);
            assert_eq!(max_bps, 150);
        }
        other => panic!("Expected OracleDivergenceCircuitBreaker, got {other:?}"),
    }
}

#[test]
fn test_clean_locked_quote_dust_truncation_idempotency() {
    // $100.00 fiat = 100_000_000 microcredits
    let fiat_mc = MicroCredits::new(100_000_000);
    // ETH price $3,456.789012 = 3_456_789_012 microcredits
    let price_mc = MicroCredits::new(3_456_789_012);

    let quote1 = MicroCreditsMathBridge::to_clean_locked_quote(fiat_mc, price_mc, 18)
        .expect("Failed to quote");

    // Must be divisible by 10^12 (dust mask)
    let mask = U256::from(1_000_000_000_000_u64);
    assert_eq!(quote1.raw() % mask, U256::ZERO, "Dust was not cleanly truncated");

    // Idempotency: re-evaluating truncation yields identical raw units
    let re_truncated = (quote1.raw() / mask) * mask;
    assert_eq!(quote1.raw(), re_truncated, "Idempotency invariant violated");
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(5000))]

    #[test]
    fn test_u256_blob32_memcmp_isomorphism_proptest(
        val_a in any::<[u8; 32]>(),
        val_b in any::<[u8; 32]>(),
    ) {
        let x = U256::from_be_bytes(val_a);
        let y = U256::from_be_bytes(val_b);

        let blob_x = ObererU256Codec::to_be_blob_vec(x);
        let blob_y = ObererU256Codec::to_be_blob_vec(y);

        // SQLite memcmp equivalence on equal-length byte arrays
        let memcmp_ord = blob_x.cmp(&blob_y);
        let int_ord = x.cmp(&y);

        prop_assert_eq!(int_ord, memcmp_ord, "memcmp ordering must exactly match U256 integer ordering");
    }

    #[test]
    fn test_u256_dec78_lexicographical_isomorphism_proptest(
        val_a in any::<[u8; 32]>(),
        val_b in any::<[u8; 32]>(),
    ) {
        let x = U256::from_be_bytes(val_a);
        let y = U256::from_be_bytes(val_b);

        let text_x = ObererU256Codec::to_padded_dec78(x);
        let text_y = ObererU256Codec::to_padded_dec78(y);

        let ascii_ord = text_x.cmp(&text_y);
        let int_ord = x.cmp(&y);

        prop_assert_eq!(int_ord, ascii_ord, "ASCII string ordering must exactly match U256 integer ordering");
    }
}
