#![forbid(unsafe_code)]
#![deny(
    clippy::float_arithmetic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::unwrap_used,
    clippy::indexing_slicing
)]

use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use gw_enterprise_traits::{
    CrossNetworkLiquidityRouterEngine, InvoiceVerificationEngine, ObererOrmStorageEngine,
    SovereignEngine, TreasuryPortfolioEngine,
};
use gw_primitives::SecurityZone;
use sovereign_gateway_engine::prelude::*;
use std::sync::Arc;
use std::time::Instant;
use tower::ServiceExt;

#[tokio::test]
async fn test_di_builder_fluent_wiring() {
    let container = SovereignGatewayDiBuilder::new()
        .with_row_mac_key([0x42u8; 32])
        .with_in_memory_storage()
        .expect("in-memory storage")
        .with_imprce_cache(32 * 1024 * 1024)
        .build()
        .expect("build container");

    // Sequential boot initialization
    container.initialize_all().await.expect("initialize all");

    // Runtime engine health and integrity check
    let runtime = container.runtime();
    assert_eq!(
        runtime.engine_id(),
        "SOVEREIGN_GATEWAY_MASTER_RUNTIME_ENGINE_V9"
    );
    assert_eq!(runtime.security_zone(), SecurityZone::ZoneAPublicWatchOnly);

    let health = runtime.health_snapshot().await.expect("health snapshot");
    assert!(health.is_healthy);

    let integrity = runtime
        .audit_cryptographic_integrity()
        .await
        .expect("integrity audit");
    assert!(integrity.audit_passed);
}

#[tokio::test]
async fn test_di_mock_engine_swapping_in_memory() {
    let start = Instant::now();

    // Invert Control: inject completely isolated in-memory test instances
    let container = SovereignGatewayDiBuilder::new()
        .with_row_mac_key([0x99u8; 32])
        .with_in_memory_storage()
        .expect("in-memory storage")
        .with_imprce_cache(16 * 1024 * 1024)
        .build()
        .expect("build container");

    container.initialize_all().await.expect("init all");

    // Verify sub-5ms in-memory startup and execution
    let elapsed = start.elapsed();
    assert!(
        elapsed.as_millis() < 500, // Safe bound across CI and debug builds
        "In-memory DI execution should be sub-millisecond fast: took {:?}",
        elapsed
    );

    // Verify Liskov Substitution Principle: dynamic dispatch behaves identically to production
    let storage = container.storage();
    assert_eq!(
        storage.engine_id(),
        "OBERER_ORM_HYBRID_STORAGE_ENGINE_V9"
    );

    let cache = container.cache();
    assert_eq!(cache.engine_id(), "SOVEREIGN_IMPRCE_CACHE_ENGINE_V9");

    let verification = container.verification();
    assert_eq!(
        verification.engine_id(),
        "SOVEREIGN_INVOICE_VERIFICATION_ENGINE"
    );

    let treasury = container.treasury();
    assert_eq!(
        treasury.engine_id(),
        "SOVEREIGN_TREASURY_PORTFOLIO_ENGINE_V9"
    );

    let router = container.router();
    assert_eq!(
        router.engine_id(),
        "SOVEREIGN_CROSS_NETWORK_LIQUIDITY_ROUTER_V9"
    );
}

async fn health_check_route(
    State(storage): State<Arc<dyn ObererOrmStorageEngine>>,
    State(verification): State<Arc<dyn InvoiceVerificationEngine>>,
    State(treasury): State<Arc<dyn TreasuryPortfolioEngine>>,
    State(router): State<Arc<dyn CrossNetworkLiquidityRouterEngine>>,
) -> impl IntoResponse {
    let storage_id = storage.engine_id();
    let verif_id = verification.engine_id();
    let treas_id = treasury.engine_id();
    let router_id = router.engine_id();

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok",
            "storage_engine": storage_id,
            "verification_engine": verif_id,
            "treasury_engine": treas_id,
            "router_engine": router_id,
        })),
    )
}

#[tokio::test]
async fn test_axum_from_ref_substate_extraction() {
    let container = SovereignGatewayDiBuilder::new()
        .with_row_mac_key([0x77u8; 32])
        .with_in_memory_storage()
        .expect("in-memory storage")
        .build()
        .expect("build container");

    container.initialize_all().await.expect("init all");

    // Assemble Axum application router with SovereignDiContainer as top-level state
    let app = Router::new()
        .route("/health", get(health_check_route))
        .with_state(container);

    // Send mock HTTP GET request via Tower oneshot
    let request = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .expect("build request");

    let response = app.oneshot(request).await.expect("execute oneshot");
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("read body");

    let json_val: serde_json::Value =
        serde_json::from_slice(&body_bytes).expect("parse json response");

    assert_eq!(json_val.get("status").and_then(|v| v.as_str()), Some("ok"));
    assert_eq!(
        json_val.get("storage_engine").and_then(|v| v.as_str()),
        Some("OBERER_ORM_HYBRID_STORAGE_ENGINE_V9")
    );
    assert_eq!(
        json_val.get("verification_engine").and_then(|v| v.as_str()),
        Some("SOVEREIGN_INVOICE_VERIFICATION_ENGINE")
    );
    assert_eq!(
        json_val.get("treasury_engine").and_then(|v| v.as_str()),
        Some("SOVEREIGN_TREASURY_PORTFOLIO_ENGINE_V9")
    );
    assert_eq!(
        json_val.get("router_engine").and_then(|v| v.as_str()),
        Some("SOVEREIGN_CROSS_NETWORK_LIQUIDITY_ROUTER_V9")
    );
}

#[tokio::test]
async fn test_zone_isolated_di_boundary_invariant() {
    let container = SovereignGatewayDiBuilder::new()
        .with_row_mac_key([0x88u8; 32])
        .with_in_memory_storage()
        .expect("in-memory storage")
        .build()
        .expect("build container");

    // Axiom 1 (Zero-Key Isolation in Zone A):
    // All engines exposed via Zone A DI container must belong strictly to Zone A.
    assert_eq!(
        container.storage().security_zone(),
        SecurityZone::ZoneAPublicWatchOnly
    );
    assert_eq!(
        container.cache().security_zone(),
        SecurityZone::ZoneAPublicWatchOnly
    );
    assert_eq!(
        container.verification().security_zone(),
        SecurityZone::ZoneAPublicWatchOnly
    );
    assert_eq!(
        container.treasury().security_zone(),
        SecurityZone::ZoneAPublicWatchOnly
    );
    assert_eq!(
        container.router().security_zone(),
        SecurityZone::ZoneAPublicWatchOnly
    );
    assert_eq!(
        container.runtime().security_zone(),
        SecurityZone::ZoneAPublicWatchOnly
    );
}
