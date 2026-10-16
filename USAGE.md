# Sovereign Rust Cryptocurrency Payment Gateway & Unified Treasury Engine (v9.0)
## Complete Developer Manual, Operational Runbook & API Integration Guide

> **Document Type:** Production Engineering Manual & Developer Integration Guide  
> **Target Framework:** Sovereign Rust Crypto Gateway & Unified Treasury Engine (Version 9.0)  
> **Toolchain Compliance:** Rust Edition 2024, `rustc 1.85+`, `#![forbid(unsafe_code)]`, strict Clippy denial fencing (`float_arithmetic`, `cast_possible_truncation`, `cast_sign_loss`, `unwrap_used`, `indexing_slicing`).

---

## Table of Contents

- [1. Introduction & Architectural Topology](#1-introduction--architectural-topology)
  - [1.1. Core System Capabilities](#11-core-system-capabilities)
  - [1.2. The Eleven Subsystem Engine Traits](#12-the-eleven-subsystem-engine-traits)
  - [1.3. Security Zone Isolation Boundaries](#13-security-zone-isolation-boundaries)
- [2. Getting Started & Workspace Setup](#2-getting-started--workspace-setup)
  - [2.1. Prerequisites & Toolchain](#21-prerequisites--toolchain)
  - [2.2. Cargo Workspace Configuration](#22-cargo-workspace-configuration)
  - [2.3. Linking the oberer-lite Submodule](#23-linking-the-oberer-lite-submodule)
- [3. Dependency Injection SDK (`sovereign-gateway-engine`)](#3-dependency-injection-sdk-sovereign-gateway-engine)
  - [3.1. SovereignGatewayDiBuilder Fluent Configuration](#31-sovereigngatewaydibuilder-fluent-configuration)
  - [3.2. SovereignDiContainer Architecture](#32-sovereigndicontainer-architecture)
  - [3.3. In-Memory Mock Swapping for Zero-I/O Tests](#33-in-memory-mock-swapping-for-zero-io-tests)
  - [3.4. Axum Web Integration via FromRef Sub-State Extraction](#34-axum-web-integration-via-fromref-sub-state-extraction)
  - [3.5. Production Axum REST API Controller Examples](#35-production-axum-rest-api-controller-examples)
- [4. Zero-Raw-SQL Relational Storage (`gw-storage-oberer`)](#4-zero-raw-sql-relational-storage-gw-storage-oberer)
  - [4.1. Dual-Tier HybridConn Engine (:memory: L1 + Disk WAL L2)](#41-dual-tier-hybridconn-engine-memory-l1--disk-wal-l2)
  - [4.2. Running Schema Migrations (bootstrap_sovereign_schema)](#42-running-schema-migrations-bootstrap_sovereign_schema)
  - [4.3. Table Handle Instantiation & Typed CRUD Operations](#43-table-handle-instantiation--typed-crud-operations)
  - [4.4. Compare-And-Swap (CAS) Optimistic Locking (update_with_version)](#44-compare-and-swap-cas-optimistic-locking-update_with_version)
  - [4.5. Read-Only Projections (TableView & ActiveWatchlistView)](#45-read-only-projections-tableview--activewatchlistview)
  - [4.6. Reactive Database Triggers & CDC Auditing (TableTrigger)](#46-reactive-database-triggers--cdc-auditing-tabletrigger)
  - [4.7. Keyed Blake3 Row MAC Signing & Constant-Time Verification](#47-keyed-blake3-row-mac-signing--constant-time-verification)
- [5. In-Memory Priority-Redundant Cache Engine (`gw-imprce`)](#5-in-memory-priority-redundant-cache-engine-gw-imprce)
  - [5.1. Priority Tier Allocations (P0 through P4)](#51-priority-tier-allocations-p0-through-p4)
  - [5.2. Sharded Multi-Arena Mirror Placement](#52-sharded-multi-arena-mirror-placement)
  - [5.3. Cache Hydration from SQLite Storage](#53-cache-hydration-from-sqlite-storage)
  - [5.4. Quorum Reads & Autonomous Bit-Rot Self-Healing](#54-quorum-reads--autonomous-bit-rot-self-healing)
  - [5.5. Priority Quota Defense & Sacrificial Flooding Protection](#55-priority-quota-defense--sacrificial-flooding-protection)
  - [5.6. Blocked Bloom Filter Batch Testing](#56-blocked-bloom-filter-batch-testing)
- [6. Multi-Chain Watch-Only Adapters (`gw-chain-adapters`)](#6-multi-chain-watch-only-adapters-gw-chain-adapters)
  - [6.1. Bitcoin BIP-84 Native SegWit Derivation](#61-bitcoin-bip-84-native-segwit-derivation)
  - [6.2. EVM Counterfactual CREATE2 Address Derivation](#62-evm-counterfactual-create2-address-derivation)
  - [6.3. Tron TVM CREATE2 & 1-Sun Dust Keeper Protocol](#63-tron-tvm-create2--1-sun-dust-keeper-protocol)
  - [6.4. Solana PDA & Unique Reference Transaction Routing](#64-solana-pda--unique-reference-transaction-routing)
  - [6.5. TON Actor Workchain with 40-Bit Feistel Obfuscated Memos](#65-ton-actor-workchain-with-40-bit-feistel-obfuscated-memos)
  - [6.6. Sui Move Coin<T> & Object ID Routing](#66-sui-move-coint--object-id-routing)
- [7. Payment Processing & 5-Gate Invoice Verification (`gw-engine`)](#7-payment-processing--5-gate-invoice-verification-gw-engine)
  - [7.1. Invoice Creation & Dual-Oracle Divergence Protection](#71-invoice-creation--dual-oracle-divergence-protection)
  - [7.2. Blockchain Block Header Ingestion & Consensus Continuity](#72-blockchain-block-header-ingestion--consensus-continuity)
  - [7.3. Transfer Event Normalization](#73-transfer-event-normalization)
  - [7.4. The 5-Gate Verification Execution Pipeline](#74-the-5-gate-verification-execution-pipeline)
  - [7.5. Typestate Lifecycle Monotonic Transitions](#75-typestate-lifecycle-monotonic-transitions)
  - [7.6. Deep Reorganization Detection & LCA Backtracking Rollback](#76-deep-reorganization-detection--lca-backtracking-rollback)
- [8. Unified Treasury & Chronological FIFO Tax Lots (`gw-treasury`)](#8-unified-treasury--chronological-fifo-tax-lots-gw-treasury)
  - [8.1. Multi-Tier Institutional Vault Structure](#81-multi-tier-institutional-vault-structure)
  - [8.2. Real-Time Unified Portfolio Valuation (18-Dec Canonical Rollup)](#82-real-time-unified-portfolio-valuation-18-dec-canonical-rollup)
  - [8.3. Ingesting Settled Invoices into FIFO Cost Basis Lots](#83-ingesting-settled-invoices-into-fifo-cost-basis-lots)
  - [8.4. Chronological FIFO Lot Consumption Algorithm](#84-chronological-fifo-lot-consumption-algorithm)
  - [8.5. Cryptographic Realized P&L Ledger Hash-Chaining (H_n)](#85-cryptographic-realized-pl-ledger-hash-chaining-h_n)
- [9. Cross-Network Liquidity Router & Sweep Engine (`gw-router`)](#9-cross-network-liquidity-router--sweep-engine-gw-router)
  - [9.1. Economical Sweep Threshold Inequality Evaluation](#91-economical-sweep-threshold-inequality-evaluation)
  - [9.2. Tier 1 Operational Gas Relayer Auto-Refill Engine](#92-tier-1-operational-gas-relayer-auto-refill-engine)
  - [9.3. Multi-Tier Proportional Split Planning (70/30 Invariant)](#93-multi-tier-proportional-split-planning-7030-invariant)
  - [9.4. Emergency and Market Flush Sweep Directives](#94-emergency-and-market-flush-sweep-directives)
- [10. Zone C Hardware Enclave Signing & Policy Guard (`gw-kms-enclave`)](#10-zone-c-hardware-enclave-signing--policy-guard-gw-kms-enclave)
  - [10.1. Stateless Pre-Sign AST Policy Guard Inspection](#101-stateless-pre-sign-ast-policy-guard-inspection)
  - [10.2. Multi-Chain Payload AST Parsers (PSBT, RLP, BOC, BCS)](#102-multi-chain-payload-ast-parsers-psbt-rlp-boc-bcs)
  - [10.3. Multi-Curve Enclave Key Management (Secp256k1 & Ed25519)](#103-multi-curve-enclave-key-management-secp256k1--ed25519)
  - [10.4. Ephemeral RAM Volatile Zeroization (ZeroizeOnDrop)](#104-ephemeral-ram-volatile-zeroization-zeroizeondrop)
- [11. Master Runtime Orchestrator Daemon (`gw-runtime`)](#11-master-runtime-orchestrator-daemon-gw-runtime)
  - [11.1. Deterministic System Startup Sequence](#111-deterministic-system-startup-sequence)
  - [11.2. The 60-Second Cryptographic Integrity Self-Audit Daemon](#112-the-60-second-cryptographic-integrity-self-audit-daemon)
  - [11.3. Graceful Shutdown & Buffer Flush Handling](#113-graceful-shutdown--buffer-flush-handling)
  - [11.4. Disaster Recovery & Crash Replay Linearizability](#114-disaster-recovery--crash-replay-linearizability)
- [12. End-to-End Operational Code Recipes](#12-end-to-end-operational-code-recipes)
  - [12.1. Recipe 1: Complete Commercial Payment-to-Treasury Lifecycle](#121-recipe-1-complete-commercial-payment-to-treasury-lifecycle)
  - [12.2. Recipe 2: Setting Up a Standalone Axum Payment Microservice](#122-recipe-2-setting-up-a-standalone-axum-payment-microservice)
  - [12.3. Recipe 3: Multi-Chain Block Indexer Worker Loop](#123-recipe-3-multi-chain-block-indexer-worker-loop)
  - [12.4. Recipe 4: Automated Sweep & Lot Disposal Cron Worker](#124-recipe-4-automated-sweep--lot-disposal-cron-worker)
- [13. Production Deployment & Operational Runbooks](#13-production-deployment--operational-runbooks)
  - [13.1. SQLite WAL Production PRAGMA Configuration](#131-sqlite-wal-production-pragma-configuration)
  - [13.2. Operational Health Snapshot Monitoring](#132-operational-health-snapshot-monitoring)
  - [13.3. Incident Response: Corrupted Row Recovery & Tamper Containment](#133-incident-response-corrupted-row-recovery--tamper-containment)
  - [13.4. Production Pre-Flight Checklist](#134-production-pre-flight-checklist)

---

## 1. Introduction & Architectural Topology

The Sovereign Rust Cryptocurrency Payment Gateway & Unified Treasury Engine (Version 9.0) provides an institutional-grade, zero-trust infrastructure for processing multi-chain digital asset payments and managing enterprise cryptocurrency reserves.

### 1.1. Core System Capabilities

- **Zero-Raw-SQL Type Safety:** 100% of relational schemas, indexes, column constraints, views, and reactive triggers are compiled via `oberer-orm` phantom-typed schema builders.
- **Strict Closed Semirings:** No IEEE-754 floating-point operations. All fiat valuations use fixed-scale `MicroCredits` ($1.00 = 1,000,000\text{ }\mu\text{credits}$), and token amounts operate over $\mathbb{Z}_{2^{256}}$ with checked integer arithmetic.
- **Multi-Chain Native Compatibility:** Built-in address derivation, transaction validation, and sweep assembly for Bitcoin, Ethereum/EVM, Solana, Tron TVM, TON, and Sui Move.
- **5-Gate Settlement Engine:** Affine linear typestate machine proving transaction success, recipient/contract binding, decimal-normalized tolerance ($\pm 50\text{ bps}$), timestamp validity, and reorg-aware confirmation depth.
- **FIFO Tax Lot Compliance:** Real-time chronological lot matching, 18-decimal canonical valuation rollup, and unbroken SHA-256 cryptographic hash-chained realized P&L ledgers.
- **Hardware Enclave Key Isolation:** Complete zero-key isolation in public and stateful zones (Axiom 1). Private keys reside exclusively within hardware enclaves (Zone C) protected by stateless pre-sign AST policy guards and volatile RAM zeroization.

### 1.2. The Eleven Subsystem Engine Traits

The system is decoupled into 11 polymorphic object-oriented engine traits rooted in the base `SovereignEngine` lifecycle contract:

```mermaid
flowchart TD
    Base["SovereignEngine (Root Lifecycle Trait)"]
    
    Base --> T1["ObererOrmStorageEngine (Dual-Tier HybridConn SQLite)"]
    Base --> T2["PriorityRedundantCacheEngine (IMPRCE P0..P4 Cache)"]
    Base --> T3["WatchOnlyDerivationEngine (Multi-Chain Address Derivation)"]
    Base --> T4["ConsensusIndexingEngine (Block Ingestion & LCA Reorg Engine)"]
    Base --> T5["InvoiceVerificationEngine (5-Gate Typestate Invoice Settlement)"]
    Base --> T6["PreSignPolicyGuardEngine (Stateless Pre-Sign AST Policy Guard)"]
    Base --> T7["EnclaveKeySignerEngine (Multi-Curve Hardware Enclave Signer)"]
    Base --> T8["UnsignedSweepAssemblerEngine (Counterfactual Sweep Assembler)"]
    Base --> T9["TreasuryPortfolioEngine (Chronological FIFO Lots & P&L Ledgers)"]
    Base --> T10["CrossNetworkLiquidityRouterEngine (Sweep Inequalities & Split Planner)"]
    Base --> T11["MasterRuntimeOrchestratorEngine (Composition Root Daemon & 60s Audit)"]
```

Every engine implements:
```rust
#[async_trait]
pub trait SovereignEngine: Send + Sync {
    fn engine_id(&self) -> &'static str;
    fn security_zone(&self) -> SecurityZone;
    async fn initialize_engine(&self) -> Result<(), SovereignError>;
    async fn audit_cryptographic_integrity(&self) -> Result<EngineIntegrityReport, SovereignError>;
    async fn health_snapshot(&self) -> Result<EngineHealthSnapshot, SovereignError>;
    async fn shutdown_gracefully(&self) -> Result<(), SovereignError>;
}
```

### 1.3. Security Zone Isolation Boundaries

The gateway architecture enforces strict physical and operational boundaries across three security zones:

```text
Security Boundary Architecture:
┌────────────────────────────────────────────────────────────────────────┐
│ Zone A: Public Watch-Only Network DMZ (Zero Private Keys)              │
│ - Crates: gw-chain-adapters, gw-engine (indexing), gw-imprce           │
│ - Untrusted RPC Ingestion, Mempool Monitoring, Bloom Filter Scanning   │
│ - Counterfactual Address Derivation (BIP-84, CREATE2, PDA, Feistel)   │
└──────────────────────────────────┬─────────────────────────────────────┘
                                   │ Verified Transfer Events
┌──────────────────────────────────▼─────────────────────────────────────┐
│ Zone B: Stateful Persisted Treasury Core (Zero Private Keys)           │
│ - Crates: gw-storage-oberer, gw-engine (settlement), gw-treasury,      │
│           gw-router, sovereign-gateway-engine, gw-runtime               │
│ - Dual-Tier HybridConn Storage (:memory: Working DB + Disk WAL DB)     │
│ - 5-Gate Typestate Invoice Verification & CAS State Transitions        │
│ - FIFO Tax Lot Accounting & Cryptographic Realized P&L Hash Chains     │
│ - Liquidity Routing & Unsigned Sweep Assembly                          │
└──────────────────────────────────┬─────────────────────────────────────┘
                                   │ Unsigned Transaction Envelopes
┌──────────────────────────────────▼─────────────────────────────────────┐
│ Zone C: Hardware Security Enclave (Private Signing Keys Isolated)       │
│ - Crate: gw-kms-enclave                                                │
│ - Multi-Curve Key Stores (Secp256k1 & Ed25519)                         │
│ - Stateless Pre-Sign AST Policy Guard (Bytecode & Whitelist Parser)    │
│ - Volatile Memory Write Barriers (ZeroizeOnDrop)                       │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Getting Started & Workspace Setup

### 2.1. Prerequisites & Toolchain

The gateway is built on modern Rust toolchain standards:
- **Rust Toolchain:** Version 1.85+ (Edition 2024)
- **C Compiler:** `gcc` or `clang` (required for SQLite C FFI compilation)
- **Target OS:** Linux (Ubuntu 22.04+, RHEL 9+), macOS (13+), or Windows (MinGW-w64 / MSVC)
- **Architecture:** `x86_64` or `aarch64`

Install the Rust toolchain:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default 1.85.0
```

### 2.2. Cargo Workspace Configuration

The root `Cargo.toml` manages all 11 crates as a virtual workspace:

```toml
[workspace]
resolver = "2"
members = [
    "crates/gw-primitives",
    "crates/gw-storage-oberer",
    "crates/gw-imprce",
    "crates/gw-enterprise-traits",
    "crates/gw-chain-adapters",
    "crates/gw-engine",
    "crates/gw-kms-enclave",
    "crates/gw-treasury",
    "crates/gw-router",
    "crates/sovereign-gateway-engine",
    "crates/gw-runtime",
]
exclude = [
    "oberer-lite",
]
```

Compiler lint fencing is declared in `.cargo/config.toml`:

```toml
[target.'cfg(all())']
rustflags = [
    "-D", "unsafe_code",
    "-D", "clippy::float_arithmetic",
    "-D", "clippy::cast_possible_truncation",
    "-D", "clippy::cast_sign_loss",
    "-D", "clippy::unwrap_used",
    "-D", "clippy::indexing_slicing",
]
```

### 2.3. Linking the oberer-lite Submodule

The workspace links to `oberer-lite` as a local path dependency in `crates/gw-storage-oberer/Cargo.toml`:

```toml
[dependencies]
oberer-orm = { path = "../../oberer-lite/oberer-orm" }
oberer-orm-derive = { path = "../../oberer-lite/oberer-orm-derive" }
```

Ensure submodules are initialized:
```bash
git submodule update --init --recursive
```

---

## 3. Dependency Injection SDK (`sovereign-gateway-engine`)

The `sovereign-gateway-engine` crate serves as the unified library SDK, providing fluent container assembly, dependency injection (DI), and seamless web framework integration.

### 3.1. SovereignGatewayDiBuilder Fluent Configuration

`SovereignGatewayDiBuilder` provides a type-safe fluent interface for registering subsystem engines and ensuring orderly initialization:

```rust
use sovereign_gateway_engine::prelude::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), SovereignError> {
    // 1. Instantiate Storage Engine (Persistent WAL mode with 256-bit Row MAC key)
    let row_mac_key = [0x55; 32];
    let storage_engine = Arc::new(
        ObererHybridStorageEngineInstance::persistent_wal("sovereign.db", row_mac_key)?
    );

    // 2. Instantiate In-Memory Cache (256 MB RAM ceiling with 3 mirror arenas)
    let cache_mac_key = [0xAA; 32];
    let cache_engine = Arc::new(
        SovereignImprceEngine::new(cache_mac_key, 256 * 1024 * 1024)
    );

    // 3. Assemble Full DI Container
    let container = SovereignGatewayDiBuilder::new()
        .with_hybrid_storage(storage_engine)
        .with_imprce_cache(cache_engine)
        .with_verification_engine(Arc::new(SovereignInvoiceVerificationEngine::new(...)))
        .with_treasury_engine(Arc::new(SovereignTreasuryPortfolioEngine::new(...)))
        .with_liquidity_router(Arc::new(SovereignCrossNetworkLiquidityRouterEngine::new(...)))
        .build()?;

    // 4. Initialize All Engines in Topological Order
    container.initialize_all_engines().await?;

    Ok(())
}
```

### 3.2. SovereignDiContainer Architecture

`SovereignDiContainer` encapsulates all active subsystem engine instances in thread-safe `Arc<dyn EngineTrait>` wrappers:

```rust
pub struct SovereignDiContainer {
    storage_engine: Arc<dyn ObererOrmStorageEngine>,
    cache_engine: Arc<dyn PriorityRedundantCacheEngine>,
    verification_engine: Arc<dyn InvoiceVerificationEngine>,
    treasury_engine: Arc<dyn TreasuryPortfolioEngine>,
    liquidity_router_engine: Arc<dyn CrossNetworkLiquidityRouterEngine>,
}
```

The container strictly prevents Zone C private key components (`EnclaveKeyStore`, `SigningKeyEnclave`) from being registered, mathematically preserving the Axiom 1 zero-key boundary.

### 3.3. In-Memory Mock Swapping for Zero-I/O Tests

For unit testing and continuous integration, the DI container allows swapping persistent disk storage with lightning-fast in-memory engines:

```rust
#[tokio::test]
async fn test_in_memory_container_lifecycle() {
    let mock_storage = Arc::new(
        ObererHybridStorageEngineInstance::in_memory_testing([0x77; 32]).unwrap()
    );
    let mock_cache = Arc::new(
        SovereignImprceEngine::new([0x88; 32], 64 * 1024 * 1024)
    );

    let container = SovereignGatewayDiBuilder::new()
        .with_hybrid_storage(mock_storage)
        .with_imprce_cache(mock_cache)
        .with_verification_engine(...)
        .build()
        .unwrap();

    // Runs in < 5 ms with zero disk I/O
    container.initialize_all_engines().await.unwrap();
    
    let report = container.audit_all_cryptographic_integrity().await.unwrap();
    assert!(report.iter().all(|r| r.audit_passed));
}
```

### 3.4. Axum Web Integration via FromRef Sub-State Extraction

`sovereign-gateway-engine` implements Axum's `FromRef` trait, allowing HTTP route handlers to extract specific engine dependencies without cloning the entire container state:

```rust
use axum::extract::{FromRef, State};
use std::sync::Arc;

// Zero-copy sub-state extractors
impl FromRef<SovereignDiContainer> for Arc<dyn InvoiceVerificationEngine> {
    fn from_ref(container: &SovereignDiContainer) -> Self {
        container.verification_engine()
    }
}

impl FromRef<SovereignDiContainer> for Arc<dyn TreasuryPortfolioEngine> {
    fn from_ref(container: &SovereignDiContainer) -> Self {
        container.treasury_engine()
    }
}

impl FromRef<SovereignDiContainer> for Arc<dyn ObererOrmStorageEngine> {
    fn from_ref(container: &SovereignDiContainer) -> Self {
        container.storage_engine()
    }
}
```

### 3.5. Production Axum REST API Controller Examples

Constructing high-performance, non-blocking HTTP endpoints with Axum:

```rust
use axum::{Router, routing::{get, post}, Json, http::StatusCode};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateInvoiceDto {
    pub customer_id: Option<uuid::Uuid>,
    pub chain_id: u64,
    pub token_symbol: String,
    pub fiat_amount_usd_cents: i64,
}

#[derive(Serialize)]
pub struct InvoiceResponseDto {
    pub invoice_id: i64,
    pub deposit_address: String,
    pub expected_amount_tokens: String,
    pub expires_at_unix: i64,
}

// Controller Handler: Extracts only Arc<dyn InvoiceVerificationEngine>
async fn create_invoice_endpoint(
    State(engine): State<Arc<dyn InvoiceVerificationEngine>>,
    Json(payload): Json<CreateInvoiceDto>,
) -> Result<Json<InvoiceResponseDto>, StatusCode> {
    let fiat_mc = MicroCredits::new(payload.fiat_amount_usd_cents * 10_000);
    
    // Create locked invoice via 5-Gate engine
    let invoice = engine.create_and_persist_locked_invoice(
        payload.customer_id,
        target_asset,
        fiat_mc,
        cex_oracle_price,
        chain_oracle_price,
        50,   // 50 bps tolerance
        1800, // 30-minute quote TTL
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(InvoiceResponseDto {
        invoice_id: invoice.invoice_id,
        deposit_address: invoice.route.deposit_address,
        expected_amount_tokens: invoice.expected_amount_raw.to_string(),
        expires_at_unix: invoice.expires_at_unix,
    }))
}

pub fn create_gateway_router(container: SovereignDiContainer) -> Router {
    Router::new()
        .route("/api/v1/invoices", post(create_invoice_endpoint))
        .with_state(container)
}
```

---

## 4. Zero-Raw-SQL Relational Storage (`gw-storage-oberer`)

`gw-storage-oberer` encapsulates the relational database engine, leveraging native `oberer-orm` schema builders, views, and reactive triggers.

### 4.1. Dual-Tier HybridConn Engine (:memory: L1 + Disk WAL L2)

`HybridConn` provides two SQLite execution contexts:
1. **`:memory:` Tier (L1):** Operates entirely in RAM. Serves all real-time read and write queries with sub-millisecond latency.
2. **Disk WAL Tier (L2):** Persists data on disk using Write-Ahead Logging (WAL), ensuring full durability across restarts and power loss.

```rust
use oberer_orm::conn::{HybridConn, SyncPolicy};

// SyncPolicy::OnDirty(100) automatically flushes dirty pages after every 100 mutations
let hybrid = HybridConn::new("sovereign.db", SyncPolicy::OnDirty(100))?;
let storage = ObererHybridStorageEngineInstance::new(hybrid, row_mac_key);
```

### 4.2. Running Schema Migrations (bootstrap_sovereign_schema)

All 11 relational schemas, foreign keys, views, and triggers are applied via `bootstrap_sovereign_schema`:

```rust
use gw_storage_oberer::migration::bootstrap_sovereign_schema;

// Bootstraps schema idempotently across both memory and disk connections
bootstrap_sovereign_schema(storage.memory_conn())?;
bootstrap_sovereign_schema(storage.file_conn())?;
```

### 4.3. Table Handle Instantiation & Typed CRUD Operations

To perform database operations, create a `Table<T>` handle from an `OrmConn` reference:

```rust
use oberer_orm::Table;
use gw_storage_oberer::tables::InvoiceRow;

let table = Table::<InvoiceRow>::new(storage.memory_conn().clone());

// 1. Insert Record
let pk = table.insert(&invoice_record).unenvelope()?;

// 2. Query Record by Primary Key
let maybe_invoice = table.find_by_pk(pk).unenvelope()?;

// 3. Fast Full-Table Scan
let all_invoices = table.find_all().unenvelope()?;

// 4. Delete Record
table.delete(pk).unenvelope()?;
```

### 4.4. Compare-And-Swap (CAS) Optimistic Locking (update_with_version)

To prevent lost updates under multi-threaded concurrency, `update_with_version()` validates version counters atomically:

```rust
let current_version = invoice.version;
let mut updated = invoice.clone();
updated.status = InvoiceLifecycleStatus::Detected;
updated.version = current_version.saturating_add(1);

// Atomic CAS write: Succeeds ONLY if version in SQLite matches current_version
let res = table.update_with_version(invoice.id, current_version, &updated).unenvelope();

match res {
    Ok(()) => println!("Invoice status advanced successfully"),
    Err(oberer_orm::error::OrmError::OptimisticLockConflict { expected, actual }) => {
        eprintln!("Concurrency conflict: expected v{}, found v{}", expected, actual);
    }
    Err(e) => eprintln!("Database error: {:?}", e),
}
```

### 4.5. Read-Only Projections (TableView & ActiveWatchlistView)

The `ActiveWatchlistView` projection allows the blockchain scanner to query pending deposit addresses without deserializing unused payload bytes:

```rust
use oberer_orm::TableView;
use gw_storage_oberer::views::ActiveWatchlistView;

let view_handle = TableView::<ActiveWatchlistView>::new(storage.memory_conn().clone());
let active_watches: Vec<ActiveWatchlistView> = view_handle.find_all().unenvelope()?;

for watch in active_watches {
    println!("Watching deposit address: {} on chain {}", watch.deposit_address, watch.chain_id);
}
```

### 4.6. Reactive Database Triggers & CDC Auditing (TableTrigger)

The `trg_invoice_state_audit` trigger executes inside SQLite on every `InvoiceRow` update, logging status transitions directly to `InvoiceStateAuditRow` without application-layer overhead:

```rust
// Verified via Phase 2 audit tests:
let audit_table = Table::<InvoiceStateAuditRow>::new(storage.memory_conn().clone());
let trail = audit_table.find_all().unenvelope()?;

for entry in trail {
    println!(
        "Audit Event: Invoice #{} moved from {:?} (v{}) to {:?} (v{}) at {}",
        entry.invoice_id, entry.old_status, entry.old_version,
        entry.new_status, entry.new_version, entry.recorded_at
    );
}
```

### 4.7. Keyed Blake3 Row MAC Signing & Constant-Time Verification

Every critical financial row carries a 32-byte keyed Blake3 MAC protecting against offline database file tampering:

```rust
// 1. Sign Row Before Database Insertion
let mac = storage.row_mac_signer().compute_row_mac(
    InvoiceRow::TABLE_LABEL,
    &invoice.pk_string(),
    invoice.version,
    &invoice.canonical_security_bytes(),
);
invoice.assign_row_mac(mac);
table.insert(&invoice).unenvelope()?;

// 2. Constant-Time Verification During Audits
let is_valid = storage.row_mac_signer().verify_row_mac(
    InvoiceRow::TABLE_LABEL,
    &invoice.pk_string(),
    invoice.version,
    &invoice.canonical_security_bytes(),
    &invoice.row_mac,
);
assert!(is_valid.is_ok(), "Tampering detected in SQLite database file!");
```

---

## 5. In-Memory Priority-Redundant Cache Engine (`gw-imprce`)

The `gw-imprce` crate implements the In-Memory Priority-Redundant Cache Engine (IMPRCE), providing sub-microsecond lookups and self-healing memory arenas.

### 5.1. Priority Tier Allocations (P0 through P4)

Memory cells are categorized into five priority tiers:
- **$P_0$ (Critical Consensus):** Block headers, whitelisted assets. $k=3$ replication. No memory cap.
- **$P_1$ (Active Payment Watch):** Pending invoice routes. $k=2$ replication. $80\%$ cap.
- **$P_2$ (Pool Lease Cooldown):** Address pool leases. $k=2$ replication. $70\%$ cap.
- **$P_3$ (Oracle Quote Locks):** CEX/DEX price feeds. $k=1$ replication. $10\%$ cap.
- **$P_4$ (Telemetry & Idempotency):** Request traces. $k=1$ replication. $5\%$ cap.

### 5.2. Sharded Multi-Arena Mirror Placement

Keys are routed across three independent mirror arenas (`Arena 0`, `Arena 1`, `Arena 2`), each partitioned into 64 lock-striped shards:

```rust
let cache = SovereignImprceEngine::new(seed_key, 256 * 1024 * 1024);

// Primary Write API
cache.put(
    "watchlist",
    "42161:0x71c676d634298150499696b01479860b83c50931",
    PriorityTier::P1ActivePaymentWatch,
    1,                                  // Version counter
    Duration::from_secs(1800),          // TTL: 30 minutes
    invoice_payload_bytes,
)?;
```

### 5.3. Cache Hydration from SQLite Storage

On system boot, `hydrate_from_oberer_hybrid()` loads 50,000 active records into memory in $< 10\text{ ms}$:

```rust
let records_hydrated = cache.hydrate_from_oberer_hybrid(storage.as_ref()).await?;
println!("Successfully warmed IMPRCE cache with {} records", records_hydrated);
```

### 5.4. Quorum Reads & Autonomous Bit-Rot Self-Healing

When reading a key, the cache probes `Arena 0`. If hardware bit-rot or memory corruption is detected via CRC32 or Blake3 MAC failure, the engine automatically queries surviving mirror arenas (`Arena 1` and `Arena 2`), extracts the uncorrupted cell, and atomically heals `Arena 0` in $< 350\text{ ns}$:

```rust
// Attempt read: transparently self-heals if corruption is detected
let maybe_payload = cache.get("watchlist", "42161:0x71c...")?;
```

### 5.5. Priority Quota Defense & Sacrificial Flooding Protection

During transaction surges, low-priority $P_4$ and $P_3$ writes are rejected once their respective memory caps are reached, guaranteeing that $P_0$ consensus data and $P_1$ invoices cannot be evicted:

```rust
// Returns Err(ImprceError::QuotaExceeded) if P4 memory ceiling is reached
cache.put("telemetry", "trace_101", PriorityTier::P4TelemetryIdempotency, 1, Duration::from_secs(60), trace)?;
```

### 5.6. Blocked Bloom Filter Batch Testing

A SIMD-accelerated 256-bit Blocked Bloom Filter screens incoming transfer events in $< 5\text{ ns}$ per test, discarding irrelevant blockchain transactions before querying memory arenas:

```rust
if !cache.bloom_filter_contains("42161", &event.recipient_address) {
    // Transaction not in active watchlist: discard immediately
    return Ok(());
}
```

---

## 6. Multi-Chain Watch-Only Adapters (`gw-chain-adapters`)

`gw-chain-adapters` handles watch-only address derivation, counterfactual address calculation, and sweep assembly across all supported blockchains.

### 6.1. Bitcoin BIP-84 Native SegWit Derivation

Derives Native SegWit (`bc1q...`) deposit addresses from an extended public key (`zpub`):

```rust
use gw_chain_adapters::bip84::Bip84DerivationEngine;

let adapter = Bip84DerivationEngine::new("zpub6rFR7y4Q2AijBEqTUquhMRMmTNQ4yKEwy2vUGWTMtd9cm364EFEDCKZ8nvWBnL42S21uFoxWJTSXM5HeBy6LGAmmETMt35o26hZxozET18v")?;
let deposit_address = adapter.derive_address(1001)?;
assert!(deposit_address.starts_with("bc1q"));
```

### 6.2. EVM Counterfactual CREATE2 Address Derivation

Computes counterfactual payment addresses without on-chain contract deployment, binding the cold vault destination into the bytecode (Axiom 2):

```rust
use gw_chain_adapters::create2::EvmCreate2Factory;

let factory_address = "0x1111111111111111111111111111111111111111";
let cold_vault = "0x9999999999999999999999999999999999999999";
let init_code_hash = hex::decode("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855").unwrap();

let factory = EvmCreate2Factory::new(factory_address, cold_vault, &init_code_hash)?;
let deposit_address = factory.derive_counterfactual_address(invoice_id)?;
```

### 6.3. Tron TVM CREATE2 & 1-Sun Dust Keeper Protocol

The Tron adapter retains 1 Sun ($0.000001\text{ TRX}$) during sweeps, keeping the account active and saving $50.4\%$ on subsequent activation fees (65,000 Energy vs. 131,000 Energy):

```rust
use gw_chain_adapters::sweep::TronSweepAssembler;

let assembler = TronSweepAssembler::new(cold_vault_address);
let unsigned_tx = assembler.assemble_sweep_with_dust_keeper(
    deposit_address,
    total_balance_sun,
    1, // Retain 1 Sun dust
)?;
```

### 6.4. Solana PDA & Unique Reference Transaction Routing

Derives Program Derived Addresses (PDAs) off-curve and verifies unique transaction reference public keys:

```rust
use gw_chain_adapters::solana::SolanaAdapter;

let adapter = SolanaAdapter::new(program_id, cold_vault_pubkey)?;
let (pda_address, bump) = adapter.derive_counterfactual_pda(invoice_id)?;
```

### 6.5. TON Actor Workchain with 40-Bit Feistel Obfuscated Memos

Encrypts internal invoice IDs into user-facing 40-bit numerical memos using a Feistel cipher, preventing invoice enumeration attacks while ensuring bijective memo decoding:

```rust
use gw_chain_adapters::ton::TonFeistelMemoEngine;

let memo_engine = TonFeistelMemoEngine::new([0x42; 16]); // 128-bit key
let user_memo: u64 = memo_engine.encrypt_invoice_id(1001);
let decoded_id: i64 = memo_engine.decrypt_memo(user_memo)?;
assert_eq!(decoded_id, 1001);
```

### 6.6. Sui Move Coin<T> & Object ID Routing

Routes payments on the Sui network using programmable transaction blocks (PTBs) and Coin<T> object split/merge operations.

---

## 7. Payment Processing & 5-Gate Invoice Verification (`gw-engine`)

`gw-engine` implements the 5-Gate invoice verification state machine, ensuring mathematical correctness and protection against chain reorganizations.

### 7.1. Invoice Creation & Dual-Oracle Divergence Protection

Invoices are created with locked pricing validated across dual price oracles:
- If CEX price diverges from On-Chain Oracle by $> 150\text{ bps}$, creation is rejected by the circuit breaker:

```rust
let invoice = verification_engine.create_and_persist_locked_invoice(
    customer_uuid,
    target_asset,
    MicroCredits::new(100_000_000), // $100.00
    MicroCredits::new(2_000_000_000), // CEX Price: $2,000.00
    MicroCredits::new(2_005_000_000), // Chain Price: $2,005.00 (25 bps diff -> OK)
    50,   // Tolerance: 50 bps
    1800, // TTL: 1800 seconds
).await?;
```

### 7.2. Blockchain Block Header Ingestion & Consensus Continuity

The consensus engine validates the block DAG hash chain:
$$P(b_h) \equiv H(b_{h-1})$$

```rust
let continuity_ok = consensus_engine.ingest_block_header(
    chain_id,
    block_number,
    block_hash,
    parent_hash,
    timestamp_unix,
).await?;
```

### 7.3. Transfer Event Normalization

Incoming transfers are parsed into canonical `NormalizedTransferEvent` structs:

```rust
let event = NormalizedTransferEvent {
    chain_id: ChainId::EthereumMainnet,
    tx_hash: "0xabcdef...".to_string(),
    block_header: current_header,
    from_address: "0x111...".to_string(),
    to_address: invoice.route.deposit_address.clone(),
    token_contract: invoice.asset.token_contract.clone(),
    net_credited_amount: credited_raw_tokens,
    confirmations: 12,
    reverted: false,
};
```

### 7.4. The 5-Gate Verification Execution Pipeline

```rust
let outcome = verification_engine.evaluate_pending_invoice(
    pending_invoice,
    &event,
    continuity_ok,
).await?;

match outcome {
    VerifiedTransitionOutcome::MovedToVerifiedComplete(complete) => {
        println!("Invoice settled! Fulfill token: {}", complete.state.fulfillment_idempotency_token);
    }
    VerifiedTransitionOutcome::RemainsDetected(detected) => {
        println!("Confirmations pending: {} / {}", detected.state.current_confirmations, detected.state.target_confirmations);
    }
    VerifiedTransitionOutcome::MovedToPartial(partial) => {
        println!("Partial payment received! Shortfall: {:?}", partial.state.shortfall_raw);
    }
    VerifiedTransitionOutcome::MovedToExpired(expired) => {
        println!("Invoice expired before settlement");
    }
}
```

### 7.5. Typestate Lifecycle Monotonic Transitions

Typestate invoices consume `self` by value. Attempting to re-verify a settled invoice fails at compile time because the `StatePending` handle has already been consumed.

### 7.6. Deep Reorganization Detection & LCA Backtracking Rollback

If a reorganization occurs, `rollback_reorged_invoices_above_lca(lca_height)` automatically rolls back detected invoices to `StatePending` via atomic CAS updates:

```rust
let rolled_back_invoices = verification_engine
    .rollback_reorged_invoices_above_lca(lowest_common_ancestor_height)
    .await?;
println!("Safely rolled back {} reorged invoices", rolled_back_invoices.len());
```

---

## 8. Unified Treasury & Chronological FIFO Tax Lots (`gw-treasury`)

`gw-treasury` tracks institutional reserves, enforces FIFO tax lot accounting, and maintains cryptographic ledger hash chains.

### 8.1. Multi-Tier Institutional Vault Structure

Institutional wallets are organized across four tiers:
- **Tier 0:** Counterfactual Hot Deposit Addresses.
- **Tier 1:** Operational Gas Relayers.
- **Tier 2:** Warm Treasury Vaults.
- **Tier 3:** Deep Cold Storage Vaults.

### 8.2. Real-Time Unified Portfolio Valuation (18-Dec Canonical Rollup)

Computes total holdings across all monitored blockchains normalized to 18 decimal places:

```rust
let summaries = treasury_engine.compute_unified_portfolio_summary(&prices).await?;

for summary in summaries {
    println!(
        "Asset: {:?} | Canonical 18-Dec Balance: {} | Market Valuation: ${:.2} | Unrealized P&L: ${:.2}",
        summary.logical_asset,
        summary.total_canonical_18_amount,
        summary.total_market_valuation_mc.get() as f64 / 1_000_000.0,
        summary.unrealized_pnl_microcredits as f64 / 1_000_000.0
    );
}
```

### 8.3. Ingesting Settled Invoices into FIFO Cost Basis Lots

When an invoice settles, the treasury generates a new cost basis lot:

```rust
let lot = FifoCostBasisLot {
    lot_id: 1,
    lot_uuid: uuid::Uuid::new_v4(),
    source_invoice_id: invoice.invoice_id,
    asset: invoice.asset.clone(),
    initial_amount: invoice.expected_amount_raw,
    remaining_amount: invoice.expected_amount_raw,
    acquisition_price_mc: invoice.locked_price_mc,
    acquired_at_unix: chrono::Utc::now().timestamp(),
    version: 1,
};
treasury_engine.insert_cost_basis_lot(lot)?;
```

### 8.4. Chronological FIFO Lot Consumption Algorithm

When funds are swept or liquidated, lots are consumed in strict chronological order:

```rust
let settlement_report = treasury_engine.execute_fifo_lot_consumption(
    &target_asset,
    disposed_amount,
    execution_price_mc,
    network_fee_mc,
).await?;

println!("Gross Proceeds: ${}", settlement_report.gross_proceeds_mc.get() as f64 / 1e6);
println!("Cost Basis:     ${}", settlement_report.total_cost_basis_mc.get() as f64 / 1e6);
println!("Net Realized P&L: ${}", settlement_report.net_realized_pnl_mc.get() as f64 / 1e6);
```

### 8.5. Cryptographic Realized P&L Ledger Hash-Chaining (H_n)

The realized P&L ledger forms an unbroken cryptographic hash chain:

```rust
let (ledger_len, is_unbroken) = treasury_engine.verify_realized_pnl_hash_chain()?;
assert!(is_unbroken, "Realized P&L hash chain broken!");
println!("Hash chain verified: {} contiguous blocks", ledger_len);
```

---

## 9. Cross-Network Liquidity Router & Sweep Engine (`gw-router`)

`gw-router` manages automated sweeps and gas relayer balances.

### 9.1. Economical Sweep Threshold Inequality Evaluation

Verifies whether accumulated funds justify network sweep fees ($\beta_{\text{fee}} \le 200\text{ bps}$):

```rust
let is_economical = router_engine.evaluate_sweep_viability(
    &asset,
    accumulated_balance,
    estimated_gas_fee_usd,
)?;

if is_economical {
    println!("Sweep is economical: proceed with transaction assembly");
} else {
    println!("Sweep uneconomical: wait for higher balance or lower gas fees");
}
```

### 9.2. Tier 1 Operational Gas Relayer Auto-Refill Engine

Monitors gas relayers and schedules automated refills when balances fall below threshold:

```rust
if let Some(refill_plan) = router_engine.check_and_plan_gas_refill(chain_id, current_relayer_balance).await? {
    println!("Gas relayer refill scheduled: transfer {} from Tier 2", refill_plan.refill_amount);
}
```

### 9.3. Multi-Tier Proportional Split Planning (70/30 Invariant)

Splits swept funds between cold vaults ($70\%$) and warm treasury ($30\%$) with exact remainder dust conservation:

```rust
let split_plan = router_engine.plan_proportional_split(total_swept_tokens, 7000, 3000)?;
assert_eq!(split_plan.tier3_cold_amount + split_plan.tier2_warm_amount, total_swept_tokens);
```

### 9.4. Emergency and Market Flush Sweep Directives

During market volatility, administrators can dispatch emergency flush directives bypassing standard economic thresholds:

```rust
let emergency_sweeps = router_engine.assemble_emergency_flush_directives(chain_id).await?;
```

---

## 10. Zone C Hardware Enclave Signing & Policy Guard (`gw-kms-enclave`)

`gw-kms-enclave` enforces security policies on all signing requests.

### 10.1. Stateless Pre-Sign AST Policy Guard Inspection

Before an enclave signs a transaction, `PreSignPolicyGuardEngine` validates:
1. Destination address is present in the pinned whitelist.
2. Network fee does not exceed configured safety caps.
3. Transaction payload is fresh (anti-replay nonce).

```rust
let auth_res = policy_guard.inspect_and_authorize_raw_payload(
    &unsigned_envelope,
    &enclave_attestation_report,
).await;

assert!(auth_res.is_ok(), "Policy guard rejected unauthorized transaction!");
```

### 10.2. Multi-Chain Payload AST Parsers (PSBT, RLP, BOC, BCS)

Built-in AST decoders inspect raw transaction payloads:
- **Bitcoin:** Decodes PSBT outputs and change destinations.
- **EVM:** Parses RLP bytecode and ERC-20 `transfer(address,uint256)` calldata.
- **Solana:** Decodes serialized instruction accounts and system transfers.
- **TON:** Parses Bag of Cells (BOC) message bodies.
- **Sui:** Parses Binary Canonical Serialization (BCS) programmable transaction blocks.

### 10.3. Multi-Curve Enclave Key Management (Secp256k1 & Ed25519)

The enclave manages keys across multiple curves:
- **Secp256k1:** Bitcoin, EVM, Tron.
- **Ed25519:** Solana, TON, Sui.

```rust
let signature_bytes = enclave_signer.authorize_and_sign_envelope(
    &unsigned_envelope,
    "m/84'/60'/0'/0/0",
).await?;
```

### 10.4. Ephemeral RAM Volatile Zeroization (ZeroizeOnDrop)

Private keys implement `ZeroizeOnDrop`. Memory buffers are zeroed with volatile write barriers immediately after signing, preventing key recovery from memory dumps.

---

## 11. Master Runtime Orchestrator Daemon (`gw-runtime`)

`gw-runtime` is the composition root managing the gateway's lifecycle.

### 11.1. Deterministic System Startup Sequence

```rust
use gw_runtime::daemon::MasterRuntimeDaemon;

#[tokio::main]
async fn main() -> Result<(), SovereignError> {
    let daemon = MasterRuntimeDaemon::boot_from_config("sovereign.toml").await?;
    daemon.run_until_shutdown().await?;
    Ok(())
}
```

The boot sequence:
1. Boot SQLite storage and replay WAL journal.
2. Warm-hydrate in-memory caches ($<10\text{ ms}$).
3. Verify realized P&L hash chain continuity.
4. Launch consensus blockchain indexers.
5. Start the 60-second cryptographic self-audit loop.

### 11.2. The 60-Second Cryptographic Integrity Self-Audit Daemon

Every 60 seconds, the daemon validates:
- SQLite Row MACs (offline tamper detection).
- IMPRCE memory arena CRCs and MACs.
- Realized P&L ledger hash chain.

```rust
let audit_report = daemon.execute_cryptographic_audit().await?;
assert!(audit_report.audit_passed);
```

### 11.3. Graceful Shutdown & Buffer Flush Handling

On `SIGTERM` or `SIGINT`, the daemon flushes dirty pages to disk, zeroes memory buffers, and exits cleanly:

```rust
daemon.shutdown_gracefully().await?;
```

### 11.4. Disaster Recovery & Crash Replay Linearizability

If the process terminates abruptly (`SIGKILL`), SQLite's WAL journal ensures complete crash recovery on restart without partial commits or unhashed ledger mutations.

---

## 12. End-to-End Operational Code Recipes

### 12.1. Recipe 1: Complete Commercial Payment-to-Treasury Lifecycle

```rust
// 1. Customer Requests $150 USDT Quote on Arbitrum
let invoice = verification_engine.create_and_persist_locked_invoice(
    Some(customer_id),
    usdt_arbitrum,
    MicroCredits::new(150_000_000), // $150.00
    MicroCredits::new(1_000_000),   // $1.00 USDT
    MicroCredits::new(1_000_000),
    50,
    1800,
).await?;

// 2. Register Invoice in Cache Watchlist
imprce_cache.put("watchlist", &invoice.route.deposit_address, PriorityTier::P1ActivePaymentWatch, 1, Duration::from_secs(1800), payload)?;

// 3. Blockchain Event Arrives
let outcome = verification_engine.evaluate_pending_invoice(invoice, &transfer_event, true).await?;

if let VerifiedTransitionOutcome::MovedToVerifiedComplete(settled) = outcome {
    // 4. Ingest into FIFO Treasury Lots
    treasury_engine.insert_cost_basis_lot(FifoCostBasisLot::from_settled(&settled))?;

    // 5. Sweep Funds to Cold Vault
    let unsigned_sweep = sweep_assembler.assemble_tier0_deposit_sweep(&[settled.route.deposit_address], cold_vault, WalletTier::Tier3ColdVault, &usdt_arbitrum, relayer_address).await?;

    // 6. Enclave Authorizes and Signs
    let signed_tx = enclave_signer.authorize_and_sign_envelope(&unsigned_sweep, "m/84'/60'/0'/0/0").await?;

    // 7. Consume FIFO Lots & Update Hash-Chained Ledger
    let report = treasury_engine.execute_fifo_lot_consumption(&usdt_arbitrum, settled.cumulative_received_raw, execution_price, network_fee).await?;
    println!("Payment processed and swept. Realized P&L ledger block #{} committed.", report.sequence_no);
}
```

### 12.2. Recipe 2: Setting Up a Standalone Axum Payment Microservice

```rust
use axum::{Router, routing::get, extract::State};
use sovereign_gateway_engine::prelude::*;

pub fn build_microservice(container: SovereignDiContainer) -> Router {
    Router::new()
        .route("/health", get(health_check_handler))
        .route("/api/v1/portfolio", get(portfolio_summary_handler))
        .with_state(container)
}

async fn health_check_handler(
    State(storage): State<Arc<dyn ObererOrmStorageEngine>>,
) -> String {
    let snapshot = storage.health_snapshot().await.unwrap();
    format!("Status: OK | Storage Operations: {}", snapshot.total_operations_processed)
}
```

### 12.3. Recipe 3: Multi-Chain Block Indexer Worker Loop

```rust
async fn run_indexer_worker(
    chain_id: ChainId,
    consensus_engine: Arc<dyn ConsensusIndexingEngine>,
    verification_engine: Arc<dyn InvoiceVerificationEngine>,
    cache: Arc<dyn PriorityRedundantCacheEngine>,
) {
    loop {
        let block = fetch_latest_block(chain_id).await;
        
        let continuity_ok = consensus_engine.ingest_block_header(
            chain_id, block.number, block.hash, block.parent_hash, block.timestamp
        ).await.unwrap();

        for tx in block.transactions {
            if cache.bloom_filter_contains(&chain_id.to_string(), &tx.to_address) {
                // Match confirmed: process transfer event through 5-Gate pipeline
                process_candidate_transaction(&tx, &verification_engine, continuity_ok).await;
            }
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}
```

### 12.4. Recipe 4: Automated Sweep & Lot Disposal Cron Worker

```rust
async fn run_sweep_cron(
    router: Arc<dyn CrossNetworkLiquidityRouterEngine>,
    treasury: Arc<dyn TreasuryPortfolioEngine>,
    enclave: Arc<dyn EnclaveKeySignerEngine>,
) {
    let mut interval = tokio::time::interval(Duration::from_secs(300));
    loop {
        interval.tick().await;
        
        for asset in get_monitored_assets() {
            let balance = treasury.get_tier0_balance(&asset).await.unwrap();
            let gas_fee = router.estimate_gas_fee(&asset).await.unwrap();
            
            if router.evaluate_sweep_viability(&asset, balance, gas_fee).unwrap() {
                execute_automated_sweep(&asset, balance, &router, &treasury, &enclave).await;
            }
        }
    }
}
```

---

## 13. Production Deployment & Operational Runbooks

### 13.1. SQLite WAL Production PRAGMA Configuration

Configure SQLite for concurrent production workloads:

```sql
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA busy_timeout = 5000;
PRAGMA cache_size = -64000;
PRAGMA foreign_keys = ON;
PRAGMA temp_store = MEMORY;
```

### 13.2. Operational Health Snapshot Monitoring

Poll `health_snapshot()` across all engines to export Prometheus metrics:

```rust
let snapshot = engine.health_snapshot().await?;
// Export: sovereign_engine_p99_latency_us{engine_id="IMPRCE"} 15
// Export: sovereign_engine_operations_total{engine_id="IMPRCE"} 150240
```

### 13.3. Incident Response: Corrupted Row Recovery & Tamper Containment

If the 60-second audit flags database tampering:
1. `audit_cryptographic_integrity()` returns `detected_corruptions_or_tampers > 0`.
2. The runtime daemon triggers immediate emergency quarantine.
3. Administrators identify affected rows via Blake3 Row MAC mismatches and restore from the latest verified WAL snapshot.

### 13.4. Production Pre-Flight Checklist

Before deploying to mainnet:
- [x] Verify all 48 mathematical audit tests pass (`cargo test --workspace`).
- [x] Confirm zero Clippy warnings under strict denial flags.
- [x] Verify Zone A/B dependency trees contain zero private key symbols.
- [x] Ensure Zone C enclave policy guards enforce pinned cold vault destinations.
- [x] Verify SQLite WAL mode and backup schedules are active.

---

## 14. Disaster Recovery & Deep Reorganization Handling

### 14.1. Handling Deep Chain Reorganizations (Reorg > Finality Horizon)

In rare multi-block reorg events where a block hash continuity check fails:

```rust
use gw_engine::prelude::*;
use gw_chain_adapters::prelude::*;

pub async fn handle_deep_reorg_event(
    network_id: u32,
    reorg_depth: u64,
    fork_block: u64,
    verification_engine: &Arc<dyn VerificationEngine>,
    storage: &Arc<dyn StorageBackend>,
) -> Result<ReorgRecoverySummary, EngineError> {
    tracing::warn!(
        network = network_id,
        depth = reorg_depth,
        fork_block = fork_block,
        "Deep chain reorganization detected exceeding soft finality horizon"
    );

    // Step 1: Pause incoming transaction processing for affected network
    verification_engine.pause_network_ingestion(network_id).await?;

    // Step 2: Query all transactions confirmed between fork_block and current tip
    let affected_txs = storage.query_transactions_in_block_range(network_id, fork_block, u64::MAX).await?;

    let mut rolled_back = 0u64;
    let mut re_verified = 0u64;

    for mut tx in affected_txs {
        // Invalidate current confirmation status
        tx.confirmation_status = ConfirmationStatus::MempoolOrphaned;
        tx.confirmed_block = None;
        storage.update_transaction(&tx).await?;
        rolled_back += 1;

        // Verify if transaction exists on the new canonical fork
        let canonical_lookup = verification_engine.inspect_canonical_inclusion(network_id, &tx.tx_hash).await?;
        if let Some(canonical_block) = canonical_lookup {
            tx.confirmed_block = Some(canonical_block.height);
            tx.confirmation_status = ConfirmationStatus::Confirmed(canonical_block.confirmations);
            storage.update_transaction(&tx).await?;
            re_verified += 1;
        }
    }

    // Step 3: Resume ingestion once canonical chain alignment is verified
    verification_engine.resume_network_ingestion(network_id).await?;

    Ok(ReorgRecoverySummary {
        affected_count: rolled_back,
        re_verified_count: re_verified,
        orphaned_count: rolled_back.saturating_sub(re_verified),
    })
}
```

### 14.2. Zero-Downtime Enclave Master Key Rotation

The Gateway Enclave supports zero-downtime key rotation via dual-key signing slots:

```rust
use gw_kms_enclave::prelude::*;

pub async fn rotate_enclave_signing_key(
    enclave: &Arc<dyn EnclaveKeySignerEngine>,
    auth_token: &HardwareSecurityToken,
    new_seed_entropy: &[u8; 64],
) -> Result<KeyRotationEvent, EnclaveSecurityError> {
    // 1. Authorize rotation with dual hardware security token signatures
    enclave.verify_multisig_authorization(auth_token)?;

    // 2. Stage new key in secondary active slot without evicting primary
    let new_key_id = enclave.stage_secondary_master_key(new_seed_entropy)?;

    // 3. Perform self-diagnostic test sign with secondary key
    let test_payload = b"sovereign_enclave_canary_test";
    let test_sig = enclave.sign_arbitrary_hash_slot(new_key_id, test_payload)?;
    enclave.verify_signature(new_key_id, test_payload, &test_sig)?;

    // 4. Promote secondary key to primary slot (atomic pointer swap)
    let rotation_event = enclave.promote_secondary_key_to_primary(new_key_id)?;

    tracing::info!(
        rotated_key_id = %rotation_event.primary_key_id,
        retired_key_id = %rotation_event.retired_key_id,
        "Enclave master key successfully rotated with zero downtime"
    );

    Ok(rotation_event)
}
```

---

## 15. Observability, Prometheus Metrics & OpenTelemetry Catalog

The Sovereign Crypto Engine exports standardized, Prometheus-compatible metrics for real-time monitoring and automated alerting.

### 15.1. Prometheus Metric Definitions

| Metric Name | Type | Labels | Description | Alert Threshold |
| :--- | :--- | :--- | :--- | :--- |
| `sovereign_imprce_rate_calc_duration_us` | Histogram | `quote_asset,base_asset` | IMPRCE rate calculation latency in microseconds | `> 50 μs (P99)` |
| `sovereign_imprce_vwap_spread_bps` | Gauge | `quote_asset,base_asset` | Current cross-source VWAP spread in basis points | `> 150 bps (Anomaly)` |
| `sovereign_imprce_stale_feed_count` | Counter | `source_id` | Count of stale exchange/feed ticks rejected | `> 5 / min` |
| `sovereign_gate_eval_duration_us` | Histogram | `gate_id` | Latency for Gate 1-5 evaluation in microseconds | `> 100 μs (P99)` |
| `sovereign_gate_rejection_total` | Counter | `gate_id,reason` | Total rejections per gate | Warning if sudden spike |
| `sovereign_audit_mac_violations_total` | Counter | `table_name` | Row cryptographic HMAC verification failures | `> 0 (CRITICAL ALERT)` |
| `sovereign_audit_balance_delta_satoshis` | Gauge | `asset_id` | Discrepancy between ledger and physical balance | `!= 0 (CRITICAL ALERT)` |
| `sovereign_treasury_tier0_balance` | Gauge | `asset_id` | Hot wallet operational balance | `< minimum_hot_reserve` |
| `sovereign_treasury_tier1_balance` | Gauge | `asset_id` | Warm sweep vault balance | `> sweep_batch_threshold` |
| `sovereign_enclave_signing_operations` | Counter | `network_id,status` | Hardware enclave sign requests (success/error) | Error count `> 0` |

### 15.2. Example Alerting Rule Configuration (Alertmanager)

```yaml
groups:
  - name: sovereign_crypto_gateway_alerts
    rules:
      - alert: DatabaseIntegrityViolationDetected
        expr: sovereign_audit_mac_violations_total > 0
        for: 0m
        labels:
          severity: critical
        annotations:
          summary: "CRITICAL: Database HMAC tamper or corruption detected!"
          description: "Table {{ $labels.table_name }} failed cryptographic Row MAC verification. Instant containment triggered."

      - alert: TreasuryLedgerMismatch
        expr: sovereign_audit_balance_delta_satoshis != 0
        for: 0m
        labels:
          severity: critical
        annotations:
          summary: "CRITICAL: Mathematical Treasury Ledger Invariant Broken!"
          description: "Ledger lots do not sum to total asset holdings for {{ $labels.asset_id }}."

      - alert: IMPRCEStaleFeedsWarning
        expr: rate(sovereign_imprce_stale_feed_count[5m]) > 2
        for: 2m
        labels:
          severity: warning
        annotations:
          summary: "IMPRCE Price Oracles Failing Freshness Checks"
          description: "Source {{ $labels.source_id }} has exceeded allowable latency window."
```

---

## 16. Comprehensive End-to-End Simulation Test Harness

Below is a complete test harness demonstrating end-to-end invoice generation, simulated mempool confirmation, 5-Gate validation, treasury FIFO allocation, and enclave sweep:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use gw_storage_oberer::prelude::*;
    use gw_imprce::prelude::*;
    use gw_engine::prelude::*;
    use gw_treasury::prelude::*;
    use gw_router::prelude::*;
    use gw_kms_enclave::prelude::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_full_lifecycle_invoice_to_sweep() {
        // 1. Initialize in-memory SQLite storage with cryptographic integrity checks
        let storage = Arc::new(ObererStorageBackend::new_in_memory().await.unwrap());
        storage.run_migrations().await.unwrap();

        // 2. Initialize IMPRCE rate calculation engine with deterministic quotes
        let imprce = Arc::new(MockImprceEngine::new(
            rust_decimal_macros::dec!(50_000.00), // BTC price = $50,000
        ));

        // 3. Initialize 5-Gate verification engine
        let verification = Arc::new(FiveGateVerificationEngine::new(storage.clone()));

        // 4. Initialize Treasury FIFO tracking engine
        let treasury = Arc::new(FifoTreasuryEngine::new(storage.clone()));

        // 5. Initialize Liquidity Router
        let router = Arc::new(CrossNetworkRouter::new(treasury.clone()));

        // 6. Initialize Enclave Signer (Mocked isolated zone C)
        let enclave = Arc::new(MockIsolatedEnclave::new());

        // -------------------------------------------------------------
        // STEP 1: Customer creates $100.00 USD checkout invoice
        // -------------------------------------------------------------
        let quote = imprce.calculate_crypto_quote(
            AssetId::Bitcoin,
            FiatCurrency::Usd,
            rust_decimal_macros::dec!(100.00),
            600, // 10 minute quote guarantee
        ).await.unwrap();

        assert_eq!(quote.crypto_amount_atomic, 200_000); // 0.002 BTC = 200,000 satoshis

        let invoice_id = storage.insert_invoice(&InvoiceRecord {
            invoice_id: "INV-SIM-001".to_string(),
            expected_asset: AssetId::Bitcoin,
            expected_amount_atomic: 200_000,
            receiving_address: "bc1qtestpaymentaddress99887766554433221100".to_string(),
            expiry_timestamp: 1776345600,
            status: InvoiceStatus::Pending,
        }).await.unwrap();

        // -------------------------------------------------------------
        // STEP 2: Simulated blockchain transaction arrives
        // -------------------------------------------------------------
        let incoming_tx = ChainTransaction {
            tx_hash: "0x33445566778899aabbccddeeff00112233445566778899aabbccddeeff001122".to_string(),
            network_id: 1, // Bitcoin Mainnet
            destination_address: "bc1qtestpaymentaddress99887766554433221100".to_string(),
            atomic_amount: 200_000,
            block_height: 850_000,
            block_hash: "0x00000000000000000002a8b9c...".to_string(),
            parent_block_hash: "0x0000000000000000000197a8b...".to_string(),
            confirmations: 6,
        };

        // -------------------------------------------------------------
        // STEP 3: Pass transaction through 5-Gate pipeline
        // -------------------------------------------------------------
        let gate_result = verification.evaluate_transaction_gates(&incoming_tx, &invoice_id).await.unwrap();
        assert!(gate_result.passed_all_gates());

        // Update invoice to Settled
        storage.update_invoice_status(&invoice_id, InvoiceStatus::Settled).await.unwrap();

        // -------------------------------------------------------------
        // STEP 4: Ingest settled transaction into Treasury FIFO tracking
        // -------------------------------------------------------------
        let lot_id = treasury.record_deposit_lot(
            AssetId::Bitcoin,
            200_000,
            &incoming_tx.tx_hash,
            rust_decimal_macros::dec!(50_000.00),
        ).await.unwrap();

        assert_eq!(treasury.get_total_available_balance(&AssetId::Bitcoin).await.unwrap(), 200_000);

        // -------------------------------------------------------------
        // STEP 5: Route automated sweep to cold storage
        // -------------------------------------------------------------
        let sweep_decision = router.evaluate_sweep_viability(
            &AssetId::Bitcoin,
            200_000,
            1_000, // 1,000 satoshi network fee
        ).unwrap();

        assert!(sweep_decision);

        let sweep_tx = router.build_sweep_transaction(
            &AssetId::Bitcoin,
            200_000,
            "bc1qcoldsafekeepingvault11223344556677889900",
        ).await.unwrap();

        // Sign inside Enclave
        let signature = enclave.sign_transaction(&sweep_tx.signing_payload).await.unwrap();
        assert!(!signature.is_empty());

        // Dispose FIFO lot
        let disposed = treasury.dispose_fifo_lots(&AssetId::Bitcoin, 200_000).await.unwrap();
        assert_eq!(disposed.lots.len(), 1);
        assert_eq!(disposed.lots[0].lot_id, lot_id);
        assert_eq!(treasury.get_total_available_balance(&AssetId::Bitcoin).await.unwrap(), 0);
    }
}
```

---

*Authored by the Sovereign Gateway & Treasury Engine Architecture Team.*  
*Document ID: `USAGE-SOVEREIGN-CRYPTO-GATEWAY-V9`*

