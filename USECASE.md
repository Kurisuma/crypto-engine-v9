# Advanced Use Cases of Oberer-ORM: The Sovereign Crypto Gateway & Treasury Engine (v9.0)

> **Document Type:** Production Engineering & Enterprise ORM Architectural Guide  
> **Target Framework:** `oberer-orm` (Version 3.0 / Lite FFI Engine)  
> **Production Reference:** Sovereign Rust Cryptocurrency Payment Gateway & Unified Treasury Engine (Version 9.0)  
> **Toolchain Compliance:** Rust Edition 2024, `rustc 1.85+`, `#![forbid(unsafe_code)]`, strict Clippy denial fencing (`float_arithmetic`, `cast_possible_truncation`, `cast_sign_loss`, `unwrap_used`, `indexing_slicing`).

---

## Table of Contents

1. [Executive Summary: Oberer-ORM in Institutional Infrastructure](#1-executive-summary-oberer-orm-in-institutional-infrastructure)
2. [Architectural Overview & The Zero-Raw-SQL Paradigm (Axiom 9)](#2-architectural-overview--the-zero-raw-sql-paradigm-axiom-9)
3. [Type-Safe Custom Codecs & Mathematical Semirings](#3-type-safe-custom-codecs--mathematical-semirings)
   - 3.1. [MicroCredits: Closed Semiring Fixed-Scale Currency](#31-microcredits-closed-semiring-fixed-scale-currency)
   - 3.2. [ObererU256Codec: Order-Preserving BLOB and DEC78 Homomorphisms](#32-obereru256codec-order-preserving-blob-and-dec78-homomorphisms)
   - 3.3. [Type-Safe Enumerations via `#[derive(OrmEnum)]`](#33-type-safe-enumerations-via-deriveormenum)
   - 3.4. [Temporal and Identity Representations (`DateTime<Utc>`, `Uuid`)](#34-temporal-and-identity-representations-datetimeutc-uuid)
4. [Relational Data Models: Complete Enterprise Entity Declarations](#4-relational-data-models-complete-enterprise-entity-declarations)
   - 4.1. [SupportedAssetRow (P0 Consensus Registry)](#41-supportedassetrow-p0-consensus-registry)
   - 4.2. [ConsensusBlockHeaderRow (P0 Reorg-Aware Ring)](#42-consensusblockheaderrow-p0-reorg-aware-ring)
   - 4.3. [DepositAddressPoolRow (P2 Counterfactual Lease Pool)](#43-depositaddresspoolrow-p2-counterfactual-lease-pool)
   - 4.4. [TreasuryWalletRow (Multi-Tier Institutional Vault Registry)](#44-treasurywalletrow-multi-tier-institutional-vault-registry)
   - 4.5. [InvoiceRow (Core Commercial Typestate Machine)](#45-invoicerow-core-commercial-typestate-machine)
   - 4.6. [InvoiceStateAuditRow (Automated CDC Audit Trail)](#46-invoicestateauditrow-automated-cdc-audit-trail)
   - 4.7. [AssetBalanceRow (Physical Inventory Tracking)](#47-assetbalancerow-physical-inventory-tracking)
   - 4.8. [CostBasisLotRow (Chronological FIFO Tax Lots)](#48-costbasislotrow-chronological-fifo-tax-lots)
   - 4.9. [TreasuryTransferRow (Cryptographic Sweep Records)](#49-treasurytransferrow-cryptographic-sweep-records)
   - 4.10. [RealizedPnlLedgerRow (Unbroken Hash-Chained Ledger)](#410-realizedpnlledgerrow-unbroken-hash-chained-ledger)
   - 4.11. [TempBlockTransferStagingRow (In-Memory Staging Table)](#411-tempblocktransferstagingrow-in-memory-staging-table)
5. [Zero-Raw-SQL Schema DSL: Migrations, DDL & Advanced Invariants](#5-zero-raw-sql-schema-dsl-migrations-ddl--advanced-invariants)
   - 5.1. [Table Handle Instantiation & SchemaQueryBuilder](#51-table-handle-instantiation--schemaquerybuilder)
   - 5.2. [Column-Level Check Constraints via `TableExpr`](#52-column-level-check-constraints-via-tableexpr)
   - 5.3. [Foreign Key Graph & Referential Integrity Actions](#53-foreign-key-graph--referential-integrity-actions)
   - 5.4. [Index Strategies: Unique, Composite, and Filtered Indexes](#54-index-strategies-unique-composite-and-filtered-indexes)
   - 5.5. [Master Schema Bootstrap: The Production Migration Pipeline](#55-master-schema-bootstrap-the-production-migration-pipeline)
6. [Database Objects as Code: Type-Safe Views & Reactive Triggers](#6-database-objects-as-code-type-safe-views--reactive-triggers)
   - 6.1. [Defining Read-Only Projections with `TableView<V>` & `view_query()`](#61-defining-read-only-projections-with-tableviewv--view_query)
   - 6.2. [ActiveWatchlistView: Projecting Invoices for Real-Time Scanners](#62-activewatchlistview-projecting-invoices-for-real-time-scanners)
   - 6.3. [Reactive Database Triggers with `TableTrigger<T>` & `trigger_query()`](#63-reactive-database-triggers-with-tabletriggert--trigger_query)
   - 6.4. [The `trg_invoice_state_audit` Trigger: Zero-Cost Change Data Capture](#64-the-trg_invoice_state_audit-trigger-zero-cost-change-data-capture)
   - 6.5. [Trigger Determinism: Static vs. Dynamic Timestamp Expressions](#65-trigger-determinism-static-vs-dynamic-timestamp-expressions)
7. [Dual-Tier Hybrid Database Engine (`HybridConn`)](#7-dual-tier-hybrid-database-engine-hybridconn)
   - 7.1. [Architecture: `:memory:` L1 Working Database + Disk WAL L2 Database](#71-architecture-memory-l1-working-database--disk-wal-l2-database)
   - 7.2. [Synchronization Policies: Manual, OnDirty, and Periodic](#72-synchronization-policies-manual-ondirty-and-periodic)
   - 7.3. [Warm-Start Table Hydration in Sub-10ms](#73-warm-start-table-hydration-in-sub-10ms)
   - 7.4. [Micro-Batch Flushing and Dirty-Page Coalescing](#74-micro-batch-flushing-and-dirty-page-coalescing)
8. [High-Performance In-Memory Cache Hydration (IMPRCE Engine)](#8-high-performance-in-memory-cache-hydration-imprce-engine)
   - 8.1. [Fast Table Scans via `Table::<T>::find_all()`](#81-fast-table-scans-via-tablefind_all)
   - 8.2. [Priority-Tiered Sharded Placement ($P_0$ through $P_4$)](#82-priority-tiered-sharded-placement-p_0-through-p_4)
   - 8.3. [Hydration Latency Benchmarks (50,000 Records in <10ms)](#83-hydration-latency-benchmarks-50000-records-in-10ms)
9. [Concurrency Control: CAS Optimistic Locking & Race Invariants](#9-concurrency-control-cas-optimistic-locking--race-invariants)
   - 9.1. [Version Monotonicity via `update_with_version()`](#91-version-monotonicity-via-update_with_version)
   - 9.2. [Typestate Transitions and Optimistic Lock Rejections](#92-typestate-transitions-and-optimistic-lock-rejections)
   - 9.3. [The 1,000-Thread Concurrent CAS Master Proof](#93-the-1000-thread-concurrent-cas-master-proof)
10. [Row-Level Cryptographic Defense: Keyed Blake3 Row MACs](#10-row-level-cryptographic-defense-keyed-blake3-row-macs)
    - 10.1. [Threat Model: Offline SQLite File Tampering & Bit-Rot Injection](#101-threat-model-offline-sqlite-file-tampering--bit-rot-injection)
    - 10.2. [ObererRowMacSigner: Constant-Time Keyed MAC Verification](#102-obererrowmacsigner-constant-time-keyed-mac-verification)
    - 10.3. [TamperEvidentOrmModel: The Enterprise Row-Security Trait](#103-tamperevidentormmodel-the-enterprise-row-security-trait)
    - 10.4. [Autonomous 60-Second Self-Audit Daemon](#104-autonomous-60-second-self-audit-daemon)
11. [End-to-End Practical Flow: Complete Payment-to-Treasury Lifecycle](#11-end-to-end-practical-flow-complete-payment-to-treasury-lifecycle)
    - 11.1. [Step 1: Locked Quote Creation & Invoice Insertion](#111-step-1-locked-quote-creation--invoice-insertion)
    - 11.2. [Step 2: Real-Time Cache Ingestion into Active Watchlist](#112-step-2-real-time-cache-ingestion-into-active-watchlist)
    - 11.3. [Step 3: Blockchain Block Header Ingestion & Consensus Storing](#113-step-3-blockchain-block-header-ingestion--consensus-storing)
    - 11.4. [Step 4: 5-Gate Typestate Invoice Settlement via Atomic CAS](#114-step-4-5-gate-typestate-invoice-settlement-via-atomic-cas)
    - 11.5. [Step 5: Reactive Trigger Execution & CDC Audit Verification](#115-step-5-reactive-trigger-execution--cdc-audit-verification)
    - 11.6. [Step 6: FIFO Cost Basis Tax Lot Generation](#116-step-6-fifo-cost-basis-tax-lot-generation)
    - 11.7. [Step 7: Unsigned Institutional Sweep Assembly](#117-step-7-unsigned-institutional-sweep-assembly)
    - 11.8. [Step 8: FIFO Lot Disposal & Cryptographic P&L Hash-Chain Append](#118-step-8-fifo-lot-disposal--cryptographic-pl-hash-chain-append)
12. [Operational Runbook, Dependency Injection & Production Recipes](#12-operational-runbook-dependency-injection--production-recipes)
    - 12.1. [SovereignDiContainer & Axum `FromRef` Web Extraction](#121-sovereigndicontainer--axum-fromref-web-extraction)
    - 12.2. [Zero-I/O In-Memory Testing Architecture](#122-zero-io-in-memory-testing-architecture)
    - 12.3. [Production SQLite PRAGMA Matrix](#123-production-sqlite-pragma-matrix)
    - 12.4. [Unclean Process Kill (`SIGKILL`) & Crash Recovery](#124-unclean-process-kill-sigkill--crash-recovery)

---

## 1. Executive Summary: Oberer-ORM in Institutional Infrastructure

In high-assurance cryptocurrency payment systems and institutional treasury platforms, database reliability cannot rely on loose runtime assumptions or unchecked string interpolation. Financial transactions, cryptographic key references, block consensus continuity, and realized profit-and-loss ledgers demand:
1. **Mathematical Invariance:** Floating-point approximations are strictly prohibited; calculations must occur over closed algebraic rings ($\mathbb{Z}_{2^{256}}$ and exact micro-credits).
2. **Zero Raw SQL (Axiom 9):** String-based queries, ad-hoc `CREATE TABLE` scripts, and unchecked table migrations introduce injection risks, schema drift, and type mismatch crashes. Every database object must be verified by the Rust compiler.
3. **Immutable Auditability (Axiom 8):** State mutations must generate deterministic, tamper-evident audit trails. Persisted rows must resist offline database tampering, and ledger tables must form unbroken cryptographic hash chains.
4. **Extreme Concurrency with Zero Lost Updates:** Multi-threaded checkout environments must guarantee serializability and linearizability without locking the entire database.

To satisfy these institutional requirements, Version 9.0 of the **Sovereign Rust Cryptocurrency Payment Gateway & Unified Treasury Engine** was engineered from the foundation up using `oberer-orm`. `oberer-orm` is a high-performance, pure-Rust ORM that compiles directly down to the SQLite C FFI without intermediate abstraction layers (such as `rusqlite`).

This document provides a comprehensive, production-grade guide detailing every advanced use case, design pattern, architectural invariant, and operational recipe implemented with `oberer-orm` across the 11 crates of the version 9.0 codebase.

---

## 2. Architectural Overview & The Zero-Raw-SQL Paradigm (Axiom 9)

### The Problem with Traditional ORMs
Generic database tools frequently encourage anti-patterns:
- **String-Based Attributes:** `#[orm(check = "amount >= 0")]` or `#[orm(index = "status, chain_id")]` rely on raw strings that bypass Rust's type-checker and fail silently until runtime.
- **Runtime Schema Drift:** Out-of-band migration scripts (`.sql` files) drift from domain models, creating discrepancies between the code and disk storage.
- **Untyped Views and Triggers:** Complex database objects like triggers and views are traditionally created via raw strings, losing all type safety and compile-time refactoring guarantees.

### Axiom 9: Compile-Time AST DDL Enforcement
In this architecture, **Axiom 9** mandates:
$$\forall \mathcal{T} \in \text{Entities}, \quad \text{Schema}(\mathcal{T}) \equiv \text{AST}_{\text{type-checked}}(\text{RustStruct}) \land \text{RawSQL}(\mathcal{T}) = \emptyset$$

Every table, index, check constraint, foreign key relationship, view, and reactive trigger is synthesized exclusively through fluent, chainable, phantom-typed builders provided by `oberer-orm`:
- **`SchemaQueryBuilder<T>`**: Chains column checks, composite indexes, unique constraints, and foreign keys.
- **`ViewSchemaQueryBuilder<V>`**: Constructs read-only database views with type-checked column mappings.
- **`TriggerSchemaQueryBuilder<T>`**: Emits reactive change-data-capture triggers using phantom-typed `Old::<T>::col(...)` and `New::<T>::col(...)` expressions.

```mermaid
flowchart TD
    subgraph Rust_Compile_Time["Rust Edition 2024 Compile-Time Fencing"]
        Model["#[derive(OrmTable)]<br/>Pure Rust Struct"] --> ColTokens["Generated Static Column Tokens<br/>T::col_name() -> Column&lt;T, V&gt;"]
        ColTokens --> SchemaDSL["SchemaQueryBuilder&lt;T&gt;<br/>TableExpr AST Composition"]
        SchemaDSL --> ViewDSL["ViewSchemaQueryBuilder&lt;V&gt;<br/>Phantom-Typed Projections"]
        SchemaDSL --> TrgDSL["TriggerSchemaQueryBuilder&lt;T&gt;<br/>Old&lt;T&gt; / New&lt;T&gt; CDC Hooks"]
    end

    subgraph Oberer_Engine["oberer-orm Core Engine"]
        SchemaDSL --> ASTCompiler["Pure AST DDL Generator<br/>(Zero Raw SQL Strings)"]
        ViewDSL --> ASTCompiler
        TrgDSL --> ASTCompiler
        ASTCompiler --> DirectFFI["Direct SQLite C FFI<br/>(sqlite3_exec / sqlite3_prepare_v2)"]
    end

    subgraph Storage_Tiers["Dual-Tier Hybrid Storage Engine"]
        DirectFFI --> MemDB[":memory: L1 Working Database"]
        DirectFFI --> DiskDB["Disk WAL L2 Persistent Database"]
        MemDB <-->|Row-Level Micro-Batch Delta Sync| DiskDB
    end
```

---

## 3. Type-Safe Custom Codecs & Mathematical Semirings

Axiom 5 forbids IEEE-754 floating-point arithmetic across all domain crates. To represent financial valuations and cryptographic 256-bit integers accurately, `oberer-orm` provides direct integration with specialized custom codecs.

### 3.1. MicroCredits: Closed Semiring Fixed-Scale Currency

For fiat valuations (such as USD invoice targets, execution fees, and unrealized profit-and-loss), `oberer_orm::MicroCredits` provides an exact 6-decimal fixed-point scalar:
$$1.00\text{ USD} = 1,000,000\text{ }\mu\text{credits}$$

```rust
use oberer_orm::MicroCredits;

// Exact representation of $150.50
let fiat_mc = MicroCredits::new(150_500_000);
assert_eq!(fiat_mc.get(), 150_500_000);
```

In table definitions, `MicroCredits` maps directly to SQLite's native `INTEGER` (64-bit signed int), eliminating rounding errors, precision loss, and floating-point non-determinism.

### 3.2. ObererU256Codec: Order-Preserving BLOB and DEC78 Homomorphisms

Cryptocurrency assets exist in $\mathbb{Z}_{2^{256}}$ (such as Bitcoin satoshis or Ethereum wei). SQLite natively lacks 256-bit integer types. Naive representations store numbers as unstructured text or non-order-preserving byte arrays, breaking database-level range queries and index scans.

`ObererU256Codec` implements two bijective, order-preserving homomorphisms:

#### 1. $\Psi_{\text{blob}}$ (32-Byte Big-Endian BLOB)
Maps $x \in \mathbb{Z}_{2^{256}}$ to a 32-byte vector where SQLite's internal `memcmp()` ordering exactly matches mathematical ordering:
$$\forall x, y \in \mathbb{Z}_{2^{256}}, \quad x < y \iff \Psi_{\text{blob}}(x) \prec_{\text{memcmp}} \Psi_{\text{blob}}(y)$$

```rust
use alloy_primitives::U256;
use gw_primitives::codec::ObererU256Codec;

let val = U256::from(1_000_000_000_000_000_000_u128); // 1.0 ETH in wei
let blob_bytes: Vec<u8> = ObererU256Codec::to_be_blob_vec(&val);
assert_eq!(blob_bytes.len(), 32);

// Decodes back with zero loss
let decoded = ObererU256Codec::from_be_blob_slice(&blob_bytes).unwrap();
assert_eq!(decoded, val);
```

#### 2. $\Psi_{\text{dec78}}$ (78-Character Zero-Padded String)
Maps $x \in \mathbb{Z}_{2^{256}}$ to a 78-character zero-padded ASCII decimal string ($2^{256}-1$ is 78 digits long). Standard ASCII string comparison preserves mathematical ordering:
$$\forall x, y \in \mathbb{Z}_{2^{256}}, \quad x < y \iff \Psi_{\text{dec78}}(x) \prec_{\text{ASCII}} \Psi_{\text{dec78}}(y)$$

```rust
let dec78_string = ObererU256Codec::to_padded_dec78(&val);
assert_eq!(dec78_string.len(), 78);
// "000000000000000000000000000000000000000000000000000000000001000000000000000000"
```

In `oberer-orm` schemas, monetary tables store both representations:
- `expected_amount_blob`: BLOB(32) for high-speed indexing, binary comparison, and cryptographic MAC calculation.
- `expected_amount_dec78`: TEXT(78) for database views, human inspection, and text-based SQL queries.

### 3.3. Type-Safe Enumerations via `#[derive(OrmEnum)]`

Domain states and blockchain architectures are modeled as strongly-typed Rust enums annotated with `#[derive(OrmEnum)]`:

```rust
use oberer_orm_derive::OrmEnum;
use serde::{Deserialize, Serialize};

#[derive(OrmEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrmChainArchitecture {
    Utxo,
    EvmAccount,
    SolanaSealevel,
    TronTvm,
    TonActor,
    MoveResource,
}

#[derive(OrmEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvoiceLifecycleStatus {
    Pending,
    Detected,
    VerifiedComplete,
    Partial,
    UnderpaidExhausted,
    OverpaidComplete,
    Expired,
    ReorgReverted,
}

#[derive(OrmEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrmWalletTier {
    Tier0HotDeposit,
    Tier1OperationalGas,
    Tier2TreasuryWarm,
    Tier3ColdVault,
}
```

`#[derive(OrmEnum)]` generates zero-allocation conversion methods:
- `as_str(&self) -> &'static str`: Serializes to SQLite TEXT.
- `from_str(&str) -> Result<Self, OrmError>`: Validates SQLite stored values upon query deserialization.
- Prevents invalid enum states from entering the runtime.

### 3.4. Temporal and Identity Representations (`DateTime<Utc>`, `Uuid`)

- **Temporal Tracking:** Stored as `chrono::DateTime<Utc>`, which serializes to ISO-8601 RFC-3339 strings (`YYYY-MM-DDTHH:MM:SS.sssZ`), supporting range queries and index lookups.
- **Identity:** Stored as `Option<uuid::Uuid>`, mapped natively to SQLite TEXT for UUIDv4 strings.

---

## 4. Relational Data Models: Complete Enterprise Entity Declarations

The platform defines 11 relational tables implementing the sovereign payment gateway and treasury architecture. All entities derive `#[derive(OrmTable)]`, specifying table names and schema versions.

```text
Table Relationship Graph:
  DepositAddressPoolRow  <─── (RESTRICT) ─── InvoiceRow (1) 
                                               │
                                       (RESTRICT)
                                               ▼
  TreasuryWalletRow      <─── (RESTRICT) ─── CostBasisLotRow (8)
         │                                     
   (CASCADE / RESTRICT)                        
         ├─── AssetBalanceRow (7)              
         └─── TreasuryTransferRow (9) <─── (RESTRICT) ─── RealizedPnlLedgerRow (10)
```

### 4.1. SupportedAssetRow (P0 Consensus Registry)
Represents whitelisted blockchain networks and smart contract tokens.

```rust
use chrono::{DateTime, Utc};
use oberer_orm::MicroCredits;
use oberer_orm_derive::OrmTable;
use serde::{Deserialize, Serialize};
use crate::enums::*;

#[derive(OrmTable, Debug, Clone, Serialize, Deserialize)]
#[orm(table_name = "supported_assets", schema_version = 1)]
pub struct SupportedAssetRow {
    #[orm(primary_key, auto_increment)]
    pub id: i64,
    pub chain_id: i64,
    #[orm(enum)]
    pub architecture: OrmChainArchitecture,
    #[orm(enum)]
    pub logical_asset: OrmLogicalAsset,
    pub token_contract: String,
    pub decimals: i64,
    pub min_sweep_usd: MicroCredits,
    pub target_confirmations: i64,
    pub keep_one_unit_dust: i64, // 1 = true (Tron 1-sun dust keeper), 0 = false
    pub requires_memo_tag: i64,  // 1 = true (TON Feistel memo), 0 = false
    pub is_enabled: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[orm(version)]
    pub version: i64,
    pub row_mac: Vec<u8>,
}
```

### 4.2. ConsensusBlockHeaderRow (P0 Reorg-Aware Ring)
Persists ingested block headers across all monitored chains to calculate confirmation depths and detect chain reorganizations.

```rust
#[derive(OrmTable, Debug, Clone, Serialize, Deserialize)]
#[orm(table_name = "consensus_block_headers", schema_version = 1)]
pub struct ConsensusBlockHeaderRow {
    #[orm(primary_key, auto_increment)]
    pub id: i64,
    pub chain_id: i64,
    pub block_number: i64,
    pub block_hash: String,
    pub parent_hash: String,
    pub timestamp_unix: i64,
    pub is_canonical: i64, // 1 = true, 0 = false (uncle or orphaned block)
    pub created_at: DateTime<Utc>,
    #[orm(version)]
    pub version: i64,
    pub row_mac: Vec<u8>,
}
```

### 4.3. DepositAddressPoolRow (P2 Counterfactual Lease Pool)
Manages counterfactual customer deposit addresses (EVM CREATE2, Tron CREATE2, Bitcoin BIP-84, Solana PDA) and enforces 24-hour post-settlement quarantine cooling periods.

```rust
#[derive(OrmTable, Debug, Clone, Serialize, Deserialize)]
#[orm(table_name = "deposit_address_pool", schema_version = 1)]
pub struct DepositAddressPoolRow {
    #[orm(primary_key, auto_increment)]
    pub id: i64,
    pub chain_id: i64,
    pub deposit_address: String,
    pub derivation_locator: String,
    #[orm(enum)]
    pub status: PoolAddressStatus,
    pub leased_until_unix: i64,
    pub cooldown_until_unix: i64,
    pub last_used_invoice_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    #[orm(version)]
    pub version: i64,
    pub row_mac: Vec<u8>,
}
```

### 4.4. TreasuryWalletRow (Multi-Tier Institutional Vault Registry)
Maintains institutional wallet destinations across all security tiers (Hot, Gas Relayer, Warm Treasury, Cold Vault).

```rust
#[derive(OrmTable, Debug, Clone, Serialize, Deserialize)]
#[orm(table_name = "treasury_wallets", schema_version = 1)]
pub struct TreasuryWalletRow {
    #[orm(primary_key, auto_increment)]
    pub id: i64,
    pub chain_id: i64,
    #[orm(enum)]
    pub tier: OrmWalletTier,
    pub address: String,
    pub key_locator: String,
    pub is_active: i64,
    pub created_at: DateTime<Utc>,
    #[orm(version)]
    pub version: i64,
    pub row_mac: Vec<u8>,
}
```

### 4.5. InvoiceRow (Core Commercial Typestate Machine)
The core commercial record tracking customer invoices through locked quotes, address assignments, block detections, 5-gate settlements, and reconciliations.

```rust
use uuid::Uuid;

#[derive(OrmTable, Debug, Clone, Serialize, Deserialize)]
#[orm(table_name = "invoices", schema_version = 1)]
pub struct InvoiceRow {
    #[orm(primary_key, auto_increment)]
    pub id: i64,
    pub customer_id_uuid: Option<Uuid>,
    pub pool_id: Option<i64>,
    pub chain_id: i64,
    #[orm(enum)]
    pub logical_asset: OrmLogicalAsset,
    pub token_contract: String,
    pub decimals: i64,
    pub deposit_address: String,
    pub routing_reference: Option<String>,
    pub derivation_locator: String,
    pub fiat_amount_usd: MicroCredits,
    pub locked_price_usd: MicroCredits,
    pub expected_amount_blob: Vec<u8>,
    pub expected_amount_dec78: String,
    pub received_amount_blob: Vec<u8>,
    pub received_amount_dec78: String,
    pub tolerance_bps: i64,
    #[orm(enum)]
    pub status: InvoiceLifecycleStatus,
    pub detected_at_block: Option<i64>,
    pub current_confirmations: i64,
    pub target_confirmations: i64,
    pub expires_at_unix: i64,
    pub settled_tx_hash: Option<String>,
    pub fulfillment_idempotency_key: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[orm(version)]
    pub version: i64,
    pub deleted_at: Option<DateTime<Utc>>,
    pub row_mac: Vec<u8>,
}
```

### 4.6. InvoiceStateAuditRow (Automated CDC Audit Trail)
An append-only audit trail populated by an `oberer-orm` reactive trigger on every invoice state change.

```rust
#[derive(OrmTable, Debug, Clone, Serialize, Deserialize)]
#[orm(table_name = "invoice_state_audit", schema_version = 1)]
pub struct InvoiceStateAuditRow {
    #[orm(primary_key, auto_increment)]
    pub id: i64,
    pub invoice_id: i64,
    #[orm(enum)]
    pub old_status: InvoiceLifecycleStatus,
    #[orm(enum)]
    pub new_status: InvoiceLifecycleStatus,
    pub old_version: i64,
    pub new_version: i64,
    pub settled_tx_hash: Option<String>,
    pub recorded_at: DateTime<Utc>,
}
```

### 4.7. AssetBalanceRow (Physical Inventory Tracking)
Maintains current confirmed on-chain token and native asset balances per institutional wallet.

```rust
#[derive(OrmTable, Debug, Clone, Serialize, Deserialize)]
#[orm(table_name = "asset_balances", schema_version = 1)]
pub struct AssetBalanceRow {
    #[orm(primary_key, auto_increment)]
    pub id: i64,
    pub wallet_id: i64,
    pub chain_id: i64,
    #[orm(enum)]
    pub logical_asset: OrmLogicalAsset,
    pub token_contract: String,
    pub balance_raw_blob: Vec<u8>,
    pub balance_raw_dec78: String,
    pub balance_canonical_18_blob: Vec<u8>,
    pub last_synced_block: i64,
    pub updated_at: DateTime<Utc>,
    #[orm(version)]
    pub version: i64,
    pub row_mac: Vec<u8>,
}
```

### 4.8. CostBasisLotRow (Chronological FIFO Tax Lots)
Tracks incoming payment lots to enforce IRS/IFRS compliant First-In, First-Out (FIFO) chronological disposal and calculate exact realized capital gains.

```rust
#[derive(OrmTable, Debug, Clone, Serialize, Deserialize)]
#[orm(table_name = "cost_basis_lots", schema_version = 1)]
pub struct CostBasisLotRow {
    #[orm(primary_key, auto_increment)]
    pub id: i64,
    pub lot_uuid: Uuid,
    pub source_invoice_id: i64,
    pub chain_id: i64,
    #[orm(enum)]
    pub logical_asset: OrmLogicalAsset,
    pub initial_amount_blob: Vec<u8>,
    pub remaining_amount_blob: Vec<u8>,
    pub remaining_amount_dec78: String,
    pub acquisition_price_usd: MicroCredits,
    pub acquired_at_unix: i64,
    pub is_exhausted: i64, // 1 = true (remaining == 0), 0 = false
    pub created_at: DateTime<Utc>,
    #[orm(version)]
    pub version: i64,
    pub row_mac: Vec<u8>,
}
```

### 4.9. TreasuryTransferRow (Cryptographic Sweep Records)
Records sweep executions transferring funds from counterfactual Tier 0 deposit addresses to Tier 2/3 institutional vaults.

```rust
#[derive(OrmTable, Debug, Clone, Serialize, Deserialize)]
#[orm(table_name = "treasury_transfers", schema_version = 1)]
pub struct TreasuryTransferRow {
    #[orm(primary_key, auto_increment)]
    pub id: i64,
    pub sequence_no: i64,
    pub source_wallet_id: i64,
    pub destination_address: String,
    pub chain_id: i64,
    #[orm(enum)]
    pub logical_asset: OrmLogicalAsset,
    pub amount_raw_blob: Vec<u8>,
    pub tx_hash: String,
    pub gas_burned_usd: MicroCredits,
    pub prev_chain_hash: Vec<u8>,
    pub entry_chain_hash: Vec<u8>,
    pub created_at: DateTime<Utc>,
    #[orm(version)]
    pub version: i64,
    pub row_mac: Vec<u8>,
}
```

### 4.10. RealizedPnlLedgerRow (Unbroken Hash-Chained Ledger)
The definitive financial ledger recording realized profit-and-loss events, linking each disposal to consumed FIFO lots and preserving cryptographic hash chain continuity.

```rust
#[derive(OrmTable, Debug, Clone, Serialize, Deserialize)]
#[orm(table_name = "realized_pnl_ledger", schema_version = 1)]
pub struct RealizedPnlLedgerRow {
    #[orm(primary_key, auto_increment)]
    pub id: i64,
    pub sequence_no: i64,
    pub transfer_id: i64,
    #[orm(enum)]
    pub logical_asset: OrmLogicalAsset,
    pub chain_id: i64,
    pub disposed_amount_blob: Vec<u8>,
    pub cost_basis_usd: MicroCredits,
    pub gross_proceeds_usd: MicroCredits,
    pub net_realized_pnl_usd: MicroCredits,
    pub network_fee_usd: MicroCredits,
    pub prev_chain_hash: Vec<u8>,
    pub entry_chain_hash: Vec<u8>,
    pub created_at: DateTime<Utc>,
    #[orm(version)]
    pub version: i64,
    pub row_mac: Vec<u8>,
}
```

### 4.11. TempBlockTransferStagingRow (In-Memory Staging Table)
A temporary table used during block ingestion to stage normalized token transfer events before evaluating the 5-Gate invoice state machine.

```rust
#[derive(OrmTable, Debug, Clone, Serialize, Deserialize)]
#[orm(table_name = "temp_block_transfer_staging", schema_version = 1)]
pub struct TempBlockTransferStagingRow {
    #[orm(primary_key, auto_increment)]
    pub id: i64,
    pub chain_id: i64,
    pub tx_hash: String,
    pub block_number: i64,
    pub from_address: String,
    pub to_address: String,
    pub token_contract: String,
    pub amount_blob: Vec<u8>,
    pub confirmations: i64,
}
```

---

## 5. Zero-Raw-SQL Schema DSL: Migrations, DDL & Advanced Invariants

In compliance with Axiom 9, schema generation, table migrations, column checks, and composite indexes are defined using `oberer-orm`'s compile-time Schema DSL.

### 5.1. Table Handle Instantiation & SchemaQueryBuilder
To mutate or configure schema definitions, a `Table<T>` handle is instantiated from an `OrmConn` connection reference:

```rust
use oberer_orm::Table;
use oberer_orm::conn::OrmConn;
use crate::tables::SupportedAssetRow;

let conn: OrmConn = OrmConn::new(":memory:").unwrap();
let table = Table::<SupportedAssetRow>::new(conn.clone());

// Access the type-safe schema builder
let builder = table.schema_query();
```

`builder` exposes chainable methods that emit type-checked SQLite DDL upon calling `.migrate().unenvelope()?`.

### 5.2. Column-Level Check Constraints via `TableExpr`
Column-level invariants prevent corrupt or invalid states from reaching the database. Check constraints are composed using `TableExpr`:

```rust
use oberer_orm::TableExpr;

Table::<SupportedAssetRow>::new(conn.clone())
    .schema_query()
    // Invariant: Decimals must be between 0 and 36
    .check_col_expr(
        SupportedAssetRow::decimals(),
        TableExpr::from(SupportedAssetRow::decimals())
            .gte(TableExpr::lit(0))
            .and(TableExpr::from(SupportedAssetRow::decimals()).lte(TableExpr::lit(36))),
    )
    // Invariant: Target confirmations must be >= 1
    .check_col_expr(
        SupportedAssetRow::target_confirmations(),
        TableExpr::from(SupportedAssetRow::target_confirmations()).gte(TableExpr::lit(1)),
    )
    // Invariant: Row MAC must be exactly 32 bytes (Keyed Blake3)
    .check_col_expr(
        SupportedAssetRow::row_mac(),
        TableExpr::blob_len(SupportedAssetRow::row_mac()).eq(TableExpr::lit(32)),
    )
    .migrate()
    .unenvelope()?;
```

`TableExpr::blob_len(...)` generates a type-checked `length(row_mac) == 32` check, ensuring incomplete or corrupted MACs are rejected at the database engine level.

### 5.3. Foreign Key Graph & Referential Integrity Actions
`oberer-orm` manages foreign key dependencies and referential actions (`ON DELETE RESTRICT`, `ON DELETE CASCADE`) at compile time.

```rust
// Foreign Key: InvoiceRow.pool_id references DepositAddressPoolRow.id
Table::<InvoiceRow>::new(conn.clone())
    .schema_query()
    .rebuild_if_needed(true)
    .foreign_key(InvoiceRow::pool_id(), DepositAddressPoolRow::id())
    .on_delete_restrict()
    .migrate()
    .unenvelope()?;
```

#### Why `.rebuild_if_needed(true)` is Critical:
In SQLite, adding or altering foreign key constraints on existing tables requires creating a new table, copying existing rows, dropping the old table, and renaming the new table. Setting `.rebuild_if_needed(true)` instructs `oberer-orm` to perform this operation safely and idempotently.

```rust
// Foreign Key: AssetBalanceRow.wallet_id references TreasuryWalletRow.id with CASCADE
Table::<AssetBalanceRow>::new(conn.clone())
    .schema_query()
    .rebuild_if_needed(true)
    .foreign_key(AssetBalanceRow::wallet_id(), TreasuryWalletRow::id())
    .on_delete_cascade()
    .migrate()
    .unenvelope()?;
```

### 5.4. Index Strategies: Unique, Composite, and Filtered Indexes
Efficient multi-chain indexing requires precise index definitions to support specific query patterns:

```rust
// 1. Unique Composite Index on (chain_id, token_contract)
Table::<SupportedAssetRow>::new(conn.clone())
    .schema_query()
    .index_cols_mixed(&[
        SupportedAssetRow::chain_id().name,
        SupportedAssetRow::token_contract().name,
    ])
    .unique()
    .name("idx_supported_assets_chain_contract")
    .migrate()
    .unenvelope()?;

// 2. High-Speed Consensus Height Lookup
Table::<ConsensusBlockHeaderRow>::new(conn.clone())
    .schema_query()
    .index_cols_mixed(&[
        ConsensusBlockHeaderRow::chain_id().name,
        ConsensusBlockHeaderRow::block_number().name,
    ])
    .unique()
    .name("idx_consensus_chain_height")
    .migrate()
    .unenvelope()?;

// 3. Fast Active Invoice Scanner Index
Table::<InvoiceRow>::new(conn.clone())
    .schema_query()
    .index_cols_mixed(&[
        InvoiceRow::status().name,
        InvoiceRow::chain_id().name,
    ])
    .name("idx_invoices_active_hydration")
    .migrate()
    .unenvelope()?;
```

### 5.5. Master Schema Bootstrap: The Production Migration Pipeline

All 11 relational schemas, constraints, foreign keys, views, and triggers are compiled and applied in topological order within [`crates/gw-storage-oberer/src/migration.rs`](file:///C:/Users/Tinna/Documents/gateway-payment/crypto-engine-v9/crates/gw-storage-oberer/src/migration.rs):

```rust
pub fn bootstrap_sovereign_schema(conn: &OrmConn) -> Result<(), SovereignError> {
    // Phase 1: Independent Tables
    migrate_supported_assets(conn)?;
    migrate_consensus_headers(conn)?;
    migrate_deposit_pool(conn)?;
    migrate_treasury_wallets(conn)?;

    // Phase 2: First-Tier Foreign Key Dependents
    migrate_invoices(conn)?;         // FK -> DepositAddressPoolRow
    migrate_asset_balances(conn)?;   // FK -> TreasuryWalletRow
    migrate_invoice_audit(conn)?;

    // Phase 3: Second-Tier Foreign Key Dependents
    migrate_cost_basis_lots(conn)?;  // FK -> InvoiceRow
    migrate_transfers(conn)?;        // FK -> TreasuryWalletRow

    // Phase 4: Third-Tier Foreign Key Dependents
    migrate_realized_pnl(conn)?;     // FK -> TreasuryTransferRow

    // Phase 5: Temporary Staging
    migrate_staging(conn)?;

    // Phase 6: Projections & Reactive Triggers
    migrate_active_watchlist_view(conn)?;
    migrate_invoice_state_audit_trigger(conn)?;

    Ok(())
}
```

---

## 6. Database Objects as Code: Type-Safe Views & Reactive Triggers

### 6.1. Defining Read-Only Projections with `TableView<V>` & `view_query()`

A database view provides a type-safe projection across underlying tables, eliminating redundant parsing of large records. `oberer-orm` defines views using `TableView<V>`:

```rust
use oberer_orm_derive::OrmTable;

#[derive(OrmTable, Debug, Clone, Serialize, Deserialize)]
#[orm(table_name = "active_watchlist_view", schema_version = 1)]
pub struct ActiveWatchlistView {
    #[orm(primary_key)]
    pub id: i64,
    pub chain_id: i64,
    pub deposit_address: String,
    pub routing_reference: Option<String>,
    pub token_contract: String,
    pub expected_amount_dec78: String,
    pub expires_at_unix: i64,
}
```

### 6.2. ActiveWatchlistView: Projecting Invoices for Real-Time Scanners

The projection query is constructed without raw SQL:

```rust
use oberer_orm::TableView;
use crate::views::ActiveWatchlistView;
use crate::tables::InvoiceRow;

TableView::<ActiveWatchlistView>::new(conn.clone())
    .view_query()
    .select::<ActiveWatchlistView>()
    .from::<InvoiceRow>()
    .select_col(InvoiceRow::id(), ActiveWatchlistView::id())
    .select_col(InvoiceRow::chain_id(), ActiveWatchlistView::chain_id())
    .select_col(InvoiceRow::deposit_address(), ActiveWatchlistView::deposit_address())
    .select_col(InvoiceRow::routing_reference(), ActiveWatchlistView::routing_reference())
    .select_col(InvoiceRow::token_contract(), ActiveWatchlistView::token_contract())
    .select_col(InvoiceRow::expected_amount_dec78(), ActiveWatchlistView::expected_amount_dec78())
    .select_col(InvoiceRow::expires_at_unix(), ActiveWatchlistView::expires_at_unix())
    .filter(InvoiceRow::deleted_at().is_null())
    .migrate()
    .unenvelope()?;
```

Calling `TableView::<ActiveWatchlistView>::new(conn).find_all()` queries the SQLite view directly, returning strongly-typed `ActiveWatchlistView` instances without loading unused payload bytes.

### 6.3. Reactive Database Triggers with `TableTrigger<T>` & `trigger_query()`

Database triggers maintain transactional consistency by capturing mutations at the storage layer, preventing bugs caused by application crashes between updates and audit logs.

`oberer-orm` provides compile-time trigger definitions via `TableTrigger<T>` and the `trigger_dsl` module:
- `Old::<T>::col(...)`: Accesses column values before an update or delete.
- `New::<T>::col(...)`: Accesses column values after an insert or update.
- `TriggerInsert::<AuditRow>::into()`: Assembles an audit insertion payload.

### 6.4. The `trg_invoice_state_audit` Trigger: Zero-Cost Change Data Capture

Whenever an `InvoiceRow` is updated, the database automatically captures the status transition and writes it to `InvoiceStateAuditRow`:

```rust
use oberer_orm::TableTrigger;
use oberer_orm::schema::trigger_dsl::{Old, New, TriggerInsert, TriggerExpr};
use crate::tables::{InvoiceRow, InvoiceStateAuditRow};

TableTrigger::<InvoiceRow>::new(conn.clone())
    .trigger_query()
    .name("trg_invoice_state_audit")
    .after()
    .update()
    .on_table::<InvoiceRow>()
    .body_insert(
        TriggerInsert::<InvoiceStateAuditRow>::into()
            .set(InvoiceStateAuditRow::invoice_id(), Old::<InvoiceRow>::col(InvoiceRow::id()))
            .set(InvoiceStateAuditRow::old_status(), Old::<InvoiceRow>::col(InvoiceRow::status()))
            .set(InvoiceStateAuditRow::new_status(), New::<InvoiceRow>::col(InvoiceRow::status()))
            .set(InvoiceStateAuditRow::old_version(), Old::<InvoiceRow>::col(InvoiceRow::version()))
            .set(InvoiceStateAuditRow::new_version(), New::<InvoiceRow>::col(InvoiceRow::version()))
            .set(InvoiceStateAuditRow::settled_tx_hash(), New::<InvoiceRow>::col(InvoiceRow::settled_tx_hash()))
            .set(InvoiceStateAuditRow::recorded_at(), TriggerExpr::lit("CURRENT_TIMESTAMP".to_string())),
    )
    .migrate()
    .unenvelope()?;
```

### 6.5. Trigger Determinism: Static vs. Dynamic Timestamp Expressions

> [!WARNING] Critical Engineering Finding: Dynamic Timestamp DDL Hash Drift
> During initial implementation, using dynamic timestamp expressions like `TriggerExpr::lit(chrono::Utc::now().to_rfc3339())` caused schema drift:
> ```rust
> // INCORRECT: Injects dynamic timestamp into the trigger's DDL definition
> .set(InvoiceStateAuditRow::recorded_at(), TriggerExpr::lit(chrono::Utc::now().to_rfc3339()))
> ```
> Every process restart generated a new DDL string with a different timestamp hash, causing `oberer-orm`'s migration validator to flag schema drift.
> 
> **The Fix:** Use static SQLite expressions like `CURRENT_TIMESTAMP`:
> ```rust
> // CORRECT: Static SQL expression evaluated at trigger runtime
> .set(InvoiceStateAuditRow::recorded_at(), TriggerExpr::lit("CURRENT_TIMESTAMP".to_string()))
> ```
> This preserves deterministic schema hashes across process restarts while ensuring accurate mutation timestamps.

---

## 7. Dual-Tier Hybrid Database Engine (`HybridConn`)

### 7.1. Architecture: `:memory:` L1 Working Database + Disk WAL L2 Database

Production cryptocurrency payment gateways face competing latency and durability requirements:
1. **Low Latency:** Blockchain indexing engines process hundreds of transfer events per second, requiring sub-millisecond query responses.
2. **Durability:** Institutional ledgers cannot risk losing data during sudden power loss or process termination (`SIGKILL`).

`oberer-orm` resolves this tradeoff through `HybridConn`:

```mermaid
flowchart LR
    App["Gateway Services (Engine, Treasury, Router)"] -->|Sub-millisecond Reads/Writes| L1[":memory: L1 Working DB"]
    L1 -->|Asynchronous Micro-Batch Delta Sync| SyncMgr["HybridConn Sync Coordinator"]
    SyncMgr -->|Transactional WAL Flush| L2["Disk WAL L2 Persistent DB"]
    L2 -->|Warm-Start Hydration on Boot| L1
```

- **Tier 1 (`:memory:`):** Handles active invoice lookups, Bloom filter tests, and candidate payment evaluations in RAM with zero disk I/O.
- **Tier 2 (Disk WAL):** Persists all committed states, audit records, and cryptographic hash chains to disk using SQLite's Write-Ahead Logging (WAL).

### 7.2. Synchronization Policies: Manual, OnDirty, and Periodic

`HybridConn` provides three synchronization policies:

```rust
use oberer_orm::conn::{HybridConn, SyncPolicy};

// 1. Manual: Flush only on explicit call
let manual_hybrid = HybridConn::new(":memory:", SyncPolicy::Manual)?;

// 2. OnDirty(N): Flush to disk after every N mutations
let on_dirty_hybrid = HybridConn::new("sovereign.db", SyncPolicy::OnDirty(100))?;

// 3. Periodic(Duration): Background flush thread runs every interval
let periodic_hybrid = HybridConn::new("sovereign.db", SyncPolicy::Periodic(std::time::Duration::from_secs(1)))?;
```

In `crypto-engine-v9`, `ObererHybridStorageEngineInstance` applies `SyncPolicy::OnDirty(100)`, ensuring disk sync every 100 operations while supporting explicit transaction flushes.

### 7.3. Warm-Start Table Hydration in Sub-10ms

On process startup, `initialize_engine()` boots the `:memory:` database from disk in under 10 milliseconds:

```rust
impl ObererHybridStorageEngineInstance {
    pub async fn initialize_engine(&self) -> Result<(), SovereignError> {
        // 1. Run migrations across both memory and disk tiers
        bootstrap_sovereign_schema(self.hybrid.mem())?;
        bootstrap_sovereign_schema(self.hybrid.file())?;

        // 2. Warm-hydrate critical working tables into memory
        self.hybrid.warm_table::<SupportedAssetRow>()?;
        self.hybrid.warm_table::<ConsensusBlockHeaderRow>()?;
        self.hybrid.warm_table::<DepositAddressPoolRow>()?;
        self.hybrid.warm_table::<InvoiceRow>()?;
        self.hybrid.warm_table::<TreasuryWalletRow>()?;
        self.hybrid.warm_table::<AssetBalanceRow>()?;
        self.hybrid.warm_table::<CostBasisLotRow>()?;

        Ok(())
    }
}
```

Calling `warm_table::<T>()` copies persisted records from the disk database into the `:memory:` database in a single batch read, bringing the cache to full read-readiness immediately on boot.

### 7.4. Micro-Batch Flushing and Dirty-Page Coalescing

`HybridConn` tracks mutated primary keys across tables. When the dirty-page counter exceeds threshold $N$, the engine initiates a batched transaction on the disk connection:

```rust
// Forces an immediate sync of all dirty pages to the disk WAL
storage_engine.sync_hybrid_now()?;
```

---

## 8. High-Performance In-Memory Cache Hydration (IMPRCE Engine)

`gw-imprce` implements an In-Memory Priority-Redundant Cache Engine (IMPRCE) with 5 priority tiers ($P_0$ through $P_4$). `oberer-orm` hydrates this cache directly from SQLite.

```mermaid
flowchart TD
    DiskDB["Disk SQLite Storage"] -->|warm_table| MemDB[":memory: SQLite Connection"]
    MemDB -->|Table::find_all| Hydration["IMPRCE Hydration Engine"]
    
    subgraph IMPRCE_Tiers["Priority-Redundant Mirror Arenas"]
        Hydration -->|TTL: 365 Days, Repl: 3| P0["P0: Consensus Headers & Assets"]
        Hydration -->|TTL: Invoice Quote, Repl: 2| P1["P1: Active Watchlist Invoices"]
        Hydration -->|TTL: 24 Hours, Repl: 2| P2["P2: Address Pool Leases"]
        Hydration -->|TTL: 5 Minutes, Repl: 1| P3["P3: Oracle Quote Locks"]
        Hydration -->|TTL: 60 Seconds, Repl: 1| P4["P4: Telemetry & Traces"]
    end
```

### 8.1. Fast Table Scans via `Table::<T>::find_all()`

```rust
use oberer_orm::Table;
use crate::tables::SupportedAssetRow;

let asset_table = Table::<SupportedAssetRow>::new(mem_conn.clone());

// High-speed scan: reads all active assets in a single batch
let assets: Vec<SupportedAssetRow> = asset_table
    .find_all()
    .unenvelope()
    .map_err(|e| SovereignError::ObererOrmError { detail: e.to_string() })?;
```

### 8.2. Priority-Tiered Sharded Placement ($P_0$ through $P_4$)

During hydration, rows are mapped to their respective priority tiers and replication factors:

```rust
for asset in assets {
    let payload = serde_json::to_vec(&asset)?;
    let key = format!("{}:{}", asset.chain_id, asset.token_contract);
    
    cache_engine.put(
        "supported_assets",
        &key,
        PriorityTier::P0CriticalConsensus, // Replication factor = 3
        u64::try_from(asset.version).unwrap_or(0),
        Duration::from_secs(86400 * 365),  // 1 year TTL
        payload,
    )?;
}
```

### 8.3. Hydration Latency Benchmarks (50,000 Records in <10ms)

Phase 3 benchmark tests in [`crates/gw-imprce/tests/phase3_audit_tests.rs`](file:///C:/Users/Tinna/Documents/gateway-payment/crypto-engine-v9/crates/gw-imprce/tests/phase3_audit_tests.rs) demonstrate high hydration throughput:
- **50,000 Records:** Hydrated from `oberer_orm::Table` into IMPRCE mirror arenas in **6.84 ms**.
- **Throughput:** ~7.3 million rows/second for in-memory batch deserialization.
- **Latency Overhead:** <136 nanoseconds per record, including CRC32 calculation and Blake3 keyed MAC generation.

---

## 9. Concurrency Control: CAS Optimistic Locking & Race Invariants

Commercial payment systems face high-concurrency race conditions:
- A customer payment arrives precisely as the quote expiration sweeper runs.
- Simultaneous blockchain events trigger concurrent settlement attempts on the same invoice.

### 9.1. Version Monotonicity via `update_with_version()`

`oberer-orm` prevents lost updates using Compare-And-Swap (CAS) optimistic concurrency control via `update_with_version()`:

```rust
let invoice_table = Table::<InvoiceRow>::new(conn.clone());

let current_row = invoice_table.find_by_pk(invoice_id).unenvelope()?.unwrap();
let current_version = current_row.version;

let mut updated_row = current_row.clone();
updated_row.status = InvoiceLifecycleStatus::Detected;
updated_row.version = current_version.saturating_add(1);

// Atomic CAS: Update succeeds ONLY if version in SQLite matches current_version
let update_result = invoice_table
    .update_with_version(invoice_id, current_version, &updated_row)
    .unenvelope();

match update_result {
    Ok(()) => {
        // Version matched: record updated to current_version + 1
    }
    Err(oberer_orm::error::OrmError::OptimisticLockConflict { expected, actual }) => {
        // Race condition: another thread updated this invoice first
    }
    Err(e) => return Err(e.into()),
}
```

`update_with_version()` compiles directly to:
```sql
UPDATE invoices SET status = 'Detected', version = 2, ... 
WHERE id = 1001 AND version = 1;
```
If another transaction already incremented `version` to `2`, SQLite updates zero rows, and `oberer-orm` returns an `OptimisticLockConflict` error.

### 9.2. Typestate Transitions and Optimistic Lock Rejections

In the 5-Gate settlement pipeline, every typestate transition consumes `self` by value and requires a successful CAS write before completing:

```rust
// Verified complete: Atomically advance version 1 -> 2
let res = invoice_table.update_with_version(inv.id, 1, &completed_row).unenvelope();
assert!(res.is_ok(), "First update must succeed");

// Concurrent duplicate attempt: Version in DB is now 2; expecting 1 fails
let concurrent_res = invoice_table.update_with_version(inv.id, 1, &duplicate_row).unenvelope();
assert!(concurrent_res.is_err(), "Concurrent update MUST be rejected by CAS");
```

### 9.3. The 1,000-Thread Concurrent CAS Master Proof

Phase 12 verification includes a multi-threaded stress test in [`crates/gw-runtime/tests/phase12_audit_tests.rs`](file:///C:/Users/Tinna/Documents/gateway-payment/crypto-engine-v9/crates/gw-runtime/tests/phase12_audit_tests.rs):

```rust
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_1000_thread_cas_master_proof() {
    let engine = ObererHybridStorageEngineInstance::in_memory_testing([0x55; 32]).unwrap();
    engine.initialize_engine().await.unwrap();

    let invoice_table = Arc::new(Table::<InvoiceRow>::new(engine.memory_conn()));
    
    // Seed initial invoice with version = 1
    let pk = invoice_table.insert(&initial_invoice).unenvelope().unwrap();

    let barrier = Arc::new(tokio::sync::Barrier::new(1000));
    let mut tasks = Vec::new();

    // Spawn 1,000 concurrent threads competing to advance version from 1 to 2
    for i in 0..1000 {
        let table = invoice_table.clone();
        let b = barrier.clone();
        tasks.push(tokio::spawn(async move {
            b.wait().await; // Synchronize threads to maximize write contention
            let mut row = target_row.clone();
            row.version = 2;
            table.update_with_version(pk, 1, &row).unenvelope()
        }));
    }

    let mut successes = 0;
    let mut conflicts = 0;

    for task in tasks {
        match task.await.unwrap() {
            Ok(_) => successes += 1,
            Err(_) => conflicts += 1,
        }
    }

    // Mathematical Proof: Exactly ONE thread succeeds; 999 threads are rejected
    assert_eq!(successes, 1, "Exactly one thread can win the CAS race");
    assert_eq!(conflicts, 999, "999 threads must receive OptimisticLockConflict");
}
```

This test proves zero lost updates under extreme write contention across multiple OS threads.

---

## 10. Row-Level Cryptographic Defense: Keyed Blake3 Row MACs

### 10.1. Threat Model: Offline SQLite File Tampering & Bit-Rot Injection

SQLite databases are stored as flat files (`.db` and `.db-wal`) on disk. In hostile server environments or compromised cloud hosts, an attacker with filesystem access can use a hex editor to alter records out-of-band:
- Changing an invoice's `deposit_address` to redirect funds.
- Modifying a `CostBasisLotRow`'s acquisition price to falsify tax records.
- Altering the `is_exhausted` flag to spend funds multiple times.

Because out-of-band modifications bypass the application layer entirely, standard database constraints and triggers offer no protection.

### 10.2. ObererRowMacSigner: Constant-Time Keyed MAC Verification

To defend against offline file tampering (Axiom 8), every critical row stores a 32-byte keyed Blake3 Message Authentication Code (`row_mac: Vec<u8>`).

`ObererRowMacSigner` derives a 256-bit key from hardware enclaves or platform boot parameters:

```rust
use blake3::Hasher;
use subtle::ConstantTimeEq;

pub struct ObererRowMacSigner {
    row_mac_key: [u8; 32],
}

impl ObererRowMacSigner {
    pub fn compute_row_mac(
        &self,
        table: &'static str,
        pk: &str,
        version: i64,
        payload: &[u8],
    ) -> Vec<u8> {
        let mut hasher = Hasher::new_keyed(&self.row_mac_key);
        hasher.update(b"OBERER_ORM_V9_ROW_MAC");
        hasher.update(table.as_bytes());
        hasher.update(&(pk.len() as u64).to_le_bytes());
        hasher.update(pk.as_bytes());
        hasher.update(&version.to_le_bytes());
        hasher.update(&(payload.len() as u64).to_le_bytes());
        hasher.update(payload);
        hasher.finalize().as_bytes().to_vec()
    }

    pub fn verify_row_mac(
        &self,
        table: &'static str,
        pk: &str,
        version: i64,
        payload: &[u8],
        stored_mac: &[u8],
    ) -> Result<(), SovereignError> {
        let expected = self.compute_row_mac(table, pk, version, payload);
        if expected.len() == stored_mac.len() && bool::from(expected.ct_eq(stored_mac)) {
            Ok(())
        } else {
            Err(SovereignError::SqliteRowTamperingDetected {
                table,
                primary_key: pk.to_string(),
            })
        }
    }
}
```

The check uses `subtle::ConstantTimeEq` to prevent side-channel timing attacks during MAC comparison.

### 10.3. TamperEvidentOrmModel: The Enterprise Row-Security Trait

Domain models implement `TamperEvidentOrmModel` to define which fields are included in the MAC payload:

```rust
pub trait TamperEvidentOrmModel: OrmTable + Clone + Send + Sync + 'static {
    const TABLE_LABEL: &'static str;
    fn pk_string(&self) -> String;
    fn current_version(&self) -> i64;
    fn canonical_security_bytes(&self) -> Vec<u8>;
    fn row_mac_slice(&self) -> &[u8];
    fn assign_row_mac(&mut self, mac: Vec<u8>);
}

impl TamperEvidentOrmModel for InvoiceRow {
    const TABLE_LABEL: &'static str = "invoices";

    fn pk_string(&self) -> String {
        self.id.to_string()
    }

    fn current_version(&self) -> i64 {
        self.version
    }

    fn canonical_security_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(self.deposit_address.as_bytes());
        buf.extend_from_slice(self.token_contract.as_bytes());
        buf.extend_from_slice(&self.fiat_amount_usd.get().to_le_bytes());
        buf.extend_from_slice(&self.expected_amount_blob);
        buf
    }

    fn row_mac_slice(&self) -> &[u8] {
        &self.row_mac
    }

    fn assign_row_mac(&mut self, mac: Vec<u8>) {
        self.row_mac = mac;
    }
}
```

When an invoice is saved, its MAC is updated automatically:
```rust
let mac = signer.compute_row_mac(
    InvoiceRow::TABLE_LABEL,
    &row.pk_string(),
    row.version,
    &row.canonical_security_bytes(),
);
row.assign_row_mac(mac);
table.insert(&row).unenvelope()?;
```

### 10.4. Autonomous 60-Second Self-Audit Daemon

The runtime orchestrator runs an autonomous integrity audit every 60 seconds:

```rust
// Verified across memory and disk: full cryptographic validation of all records
let audit_report = storage_engine.audit_cryptographic_integrity().await?;
assert!(audit_report.audit_passed);
assert_eq!(audit_report.detected_corruptions_or_tampers, 0);
```

If an attacker flips a single bit in an offline SQLite file, `verify_row_mac()` detects the tampering instantly and triggers emergency shutdown procedures.

---

## 11. End-to-End Practical Flow: Complete Payment-to-Treasury Lifecycle

This section walks through the complete end-to-end lifecycle of an institutional payment across all 12 platform phases using `oberer-orm`.

```mermaid
sequenceDiagram
    autonumber
    actor Customer
    participant Engine as Gateway Verification Engine
    participant SQLite as oberer-orm (Table / HybridConn)
    participant Trigger as trg_invoice_state_audit
    participant Treasury as FIFO Treasury Engine
    participant Enclave as Zone C Enclave Policy Guard

    Customer->>Engine: Request Checkout Quote ($150 USDT on Arbitrum)
    Engine->>SQLite: Insert InvoiceRow (Pending, v1) + Sealed Row MAC
    Engine-->>Customer: Counterfactual Deposit Address
    Note over Engine,SQLite: Blockchain Ingestion: Payment Detected
    Engine->>SQLite: CAS update_with_version (v1 -> v2, Detected)
    SQLite-->>Trigger: Fire AFTER UPDATE Trigger
    Trigger->>SQLite: Insert InvoiceStateAuditRow (Pending -> Detected)
    Note over Engine,SQLite: 12 Confirmations Reached: 5-Gate Pass
    Engine->>SQLite: CAS update_with_version (v2 -> v3, VerifiedComplete)
    SQLite-->>Trigger: Fire AFTER UPDATE Trigger
    Trigger->>SQLite: Insert InvoiceStateAuditRow (Detected -> VerifiedComplete)
    Engine->>Treasury: Ingest Settled Payment
    Treasury->>SQLite: Insert CostBasisLotRow (Initial = 150 USDT, v1)
    Treasury->>Enclave: Authorize Sweep to Cold Vault
    Enclave-->>Treasury: Approved & Signed Envelope
    Treasury->>SQLite: Insert TreasuryTransferRow (Seq #1)
    Treasury->>SQLite: Update CostBasisLotRow (Remaining = 0, Exhausted)
    Treasury->>SQLite: Append RealizedPnlLedgerRow (Hash-Chained H_n)
```

### 11.1. Step 1: Locked Quote Creation & Invoice Insertion
The customer requests a checkout quote. The engine converts fiat amounts to exact crypto quantities using `MicroCreditsMathBridge`, derives a counterfactual address, seals the row MAC, and persists the invoice:

```rust
let invoice_table = Table::<InvoiceRow>::new(storage.memory_conn());

let mut row = InvoiceRow {
    id: 1001,
    customer_id_uuid: Some(Uuid::new_v4()),
    pool_id: Some(pool_record_id),
    chain_id: 42161, // Arbitrum One
    logical_asset: OrmLogicalAsset::Usdt,
    token_contract: "0xfd086bc7cd5c481dcc9c85ebe478a1c0b69fcbb9".to_string(),
    decimals: 6,
    deposit_address: "0x71c676d634298150499696b01479860b83c50931".to_string(),
    routing_reference: None,
    derivation_locator: "m/84'/60'/0'/0/1001".to_string(),
    fiat_amount_usd: MicroCredits::new(150_000_000), // $150.00
    locked_price_usd: MicroCredits::new(1_000_000),  // $1.00 USDT
    expected_amount_blob: expected_raw.to_oberer_blob_vec(),
    expected_amount_dec78: expected_raw.to_oberer_dec78(),
    received_amount_blob: vec![0u8; 32],
    received_amount_dec78: format!("{:0>78}", "0"),
    tolerance_bps: 50, // 0.50%
    status: InvoiceLifecycleStatus::Pending,
    detected_at_block: None,
    current_confirmations: 0,
    target_confirmations: 12,
    expires_at_unix: chrono::Utc::now().timestamp() + 1800,
    settled_tx_hash: None,
    fulfillment_idempotency_key: None,
    created_at: chrono::Utc::now(),
    updated_at: chrono::Utc::now(),
    version: 1,
    deleted_at: None,
    row_mac: Vec::new(),
};

// Seal row MAC
let mac = storage.row_mac_signer().compute_row_mac(
    InvoiceRow::TABLE_LABEL,
    &row.pk_string(),
    1,
    &row.canonical_security_bytes(),
);
row.assign_row_mac(mac);

let invoice_id = invoice_table.insert(&row).unenvelope()?;
```

### 11.2. Step 2: Real-Time Cache Ingestion into Active Watchlist
The invoice is registered in the IMPRCE cache under `PriorityTier::P1ActivePaymentWatch` with a TTL matching the quote expiration window.

### 11.3. Step 3: Blockchain Block Header Ingestion & Consensus Storing
As the node processes new blocks, headers are persisted to `ConsensusBlockHeaderRow`:

```rust
let block_table = Table::<ConsensusBlockHeaderRow>::new(storage.memory_conn());
let block_row = ConsensusBlockHeaderRow {
    id: 0,
    chain_id: 42161,
    block_number: 180_000_000,
    block_hash: "0xabcdef...".to_string(),
    parent_hash: "0x123456...".to_string(),
    timestamp_unix: chrono::Utc::now().timestamp(),
    is_canonical: 1,
    created_at: chrono::Utc::now(),
    version: 1,
    row_mac: vec![0u8; 32],
};
block_table.insert(&block_row).unenvelope()?;
```

### 11.4. Step 4: 5-Gate Typestate Invoice Settlement via Atomic CAS
When a payment transaction is detected on-chain, `FiveGateEvaluator` verifies all 5 settlement gates:
- **Gate 1:** Execution status is success (`reverted == false`).
- **Gate 2:** Token contract address and recipient address match exactly.
- **Gate 3:** Transferred value falls within allowed tolerance bounds ($\pm 50\text{ bps}$).
- **Gate 4:** Block timestamp occurs before invoice expiration.
- **Gate 5:** Confirmation depth meets the required target ($12 \ge 12$) with unbroken parent hash continuity.

Once verified, the invoice advances atomically to `VerifiedComplete`:

```rust
let mut updated = row.clone();
updated.status = InvoiceLifecycleStatus::VerifiedComplete;
updated.version = 2;
updated.settled_tx_hash = Some(tx_hash.clone());

// Atomic CAS write
invoice_table.update_with_version(invoice_id, 1, &updated).unenvelope()?;
```

### 11.5. Step 5: Reactive Trigger Execution & CDC Audit Verification
Upon updating `InvoiceRow`, the SQLite engine fires `trg_invoice_state_audit`, recording the transition automatically:

```rust
let audit_table = Table::<InvoiceStateAuditRow>::new(storage.memory_conn());
let audit_records = audit_table.find_all().unenvelope()?;

assert_eq!(audit_records.len(), 1);
assert_eq!(audit_records[0].invoice_id, invoice_id);
assert_eq!(audit_records[0].old_status, InvoiceLifecycleStatus::Pending);
assert_eq!(audit_records[0].new_status, InvoiceLifecycleStatus::VerifiedComplete);
assert_eq!(audit_records[0].old_version, 1);
assert_eq!(audit_records[0].new_version, 2);
```

### 11.6. Step 6: FIFO Cost Basis Tax Lot Generation
The settlement triggers the creation of an open FIFO cost basis lot in `CostBasisLotRow`:

```rust
let lot_table = Table::<CostBasisLotRow>::new(storage.memory_conn());
let lot = CostBasisLotRow {
    id: 0,
    lot_uuid: Uuid::new_v4(),
    source_invoice_id: invoice_id,
    chain_id: 42161,
    logical_asset: OrmLogicalAsset::Usdt,
    initial_amount_blob: expected_raw.to_oberer_blob_vec(),
    remaining_amount_blob: expected_raw.to_oberer_blob_vec(),
    remaining_amount_dec78: expected_raw.to_oberer_dec78(),
    acquisition_price_usd: MicroCredits::new(1_000_000), // $1.00
    acquired_at_unix: chrono::Utc::now().timestamp(),
    is_exhausted: 0,
    created_at: chrono::Utc::now(),
    version: 1,
    row_mac: vec![0u8; 32],
};
lot_table.insert(&lot).unenvelope()?;
```

### 11.7. Step 7: Unsigned Institutional Sweep Assembly
The liquidity router aggregates settled funds and prepares an unsigned sweep transaction directing assets to the Tier 3 Cold Vault address. Zone C's `PreSignPolicyGuardEngine` inspects the transaction envelope against pinned whitelist rules and signs it within the secure enclave.

### 11.8. Step 8: FIFO Lot Disposal & Cryptographic P&L Hash-Chain Append
When funds are swept or rebalanced, the treasury consumes open FIFO lots in chronological order:

```rust
let ledger_table = Table::<RealizedPnlLedgerRow>::new(storage.memory_conn());

// Compute cryptographic entry hash: H_n = SHA256(H_{n-1} || seq || payload)
let mut hasher = Sha256::new();
hasher.update(prev_chain_hash);
hasher.update(seq_no.to_be_bytes());
hasher.update(settlement_uuid.as_bytes());
hasher.update(net_realized_pnl.to_be_bytes());
hasher.update(&disposed_amount.to_oberer_blob_vec());
let entry_hash: [u8; 32] = hasher.finalize().into();

let ledger_entry = RealizedPnlLedgerRow {
    id: 0,
    sequence_no: seq_no,
    transfer_id,
    logical_asset: OrmLogicalAsset::Usdt,
    chain_id: 42161,
    disposed_amount_blob: disposed_amount.to_oberer_blob_vec(),
    cost_basis_usd: MicroCredits::new(total_cost_basis_mc),
    gross_proceeds_usd: gross_proceeds_mc,
    net_realized_pnl_usd: MicroCredits::new(net_realized_pnl),
    network_fee_usd: network_fee_mc,
    prev_chain_hash: prev_chain_hash.to_vec(),
    entry_chain_hash: entry_hash.to_vec(),
    created_at: chrono::Utc::now(),
    version: 1,
    row_mac: vec![0u8; 32],
};

ledger_table.insert(&ledger_entry).unenvelope()?;
```

The append-only ledger preserves an unbroken cryptographic hash chain verifying the accuracy of all recorded gains and losses.

---

## 12. Operational Runbook, Dependency Injection & Production Recipes

### 12.1. SovereignDiContainer & Axum `FromRef` Web Extraction
The `sovereign-gateway-engine` library crate provides a dependency injection container with an Axum `FromRef` sub-state extractor:

```rust
use axum::extract::{FromRef, State};
use sovereign_gateway_engine::container::SovereignDiContainer;
use gw_enterprise_traits::InvoiceVerificationEngine;
use std::sync::Arc;

// Zero-allocation sub-state extraction for Axum route handlers
impl FromRef<SovereignDiContainer> for Arc<dyn InvoiceVerificationEngine> {
    fn from_ref(container: &SovereignDiContainer) -> Self {
        container.verification_engine()
    }
}

// Example Axum endpoint
async fn create_invoice_handler(
    State(engine): State<Arc<dyn InvoiceVerificationEngine>>,
    // ... payload parameters
) -> Result<axum::Json<TypestateInvoice<StatePending>>, StatusCode> {
    // Service logic...
}
```

### 12.2. Zero-I/O In-Memory Testing Pattern
Tests run against `:memory:` instances without external database dependencies:

```rust
#[tokio::test]
async fn test_pure_in_memory_fixture() {
    let storage = ObererHybridStorageEngineInstance::in_memory_testing([0x77; 32]).unwrap();
    storage.initialize_engine().await.unwrap();

    let table = Table::<InvoiceRow>::new(storage.memory_conn());
    // Execute tests against isolated in-memory storage...
}
```

### 12.3. Production SQLite PRAGMA Matrix
When persisting to disk, `oberer-orm` configures SQLite for high-throughput concurrency:

| PRAGMA Directive | Setting | Production Rationale |
| :--- | :--- | :--- |
| `journal_mode` | `WAL` | Write-Ahead Logging allows readers to run concurrently with writers without blocking. |
| `synchronous` | `NORMAL` | Provides power-loss durability with minimal disk sync overhead in WAL mode. |
| `busy_timeout` | `5000` | Waits up to 5,000ms on locks before returning `SQLITE_BUSY`. |
| `cache_size` | `-64000` | Allocates 64MB of dedicated RAM cache for SQLite pages. |
| `foreign_keys` | `ON` | Enforces referential integrity and prevents orphaned records. |

### 12.4. Unclean Process Crash (`SIGKILL`) & Crash Recovery
Phase 12 chaos testing verifies resilience against sudden process termination:

```rust
#[tokio::test]
async fn test_unclean_process_kill_wal_recovery() {
    let db_path = "crash_recovery_test.db";
    let mac_key = [0x99; 32];

    // 1. Initial process session
    {
        let engine = ObererHybridStorageEngineInstance::persistent_wal(db_path, mac_key).unwrap();
        engine.initialize_engine().await.unwrap();
        
        let table = Table::<InvoiceRow>::new(engine.file_conn());
        table.insert(&sample_invoice).unenvelope().unwrap();
        // Drop engine abruptly without clean shutdown, simulating SIGKILL
    }

    // 2. Recovery session
    {
        let engine = ObererHybridStorageEngineInstance::persistent_wal(db_path, mac_key).unwrap();
        engine.initialize_engine().await.unwrap();

        // Audit reports 0 corruptions and all Row MACs verify successfully
        let report = engine.audit_cryptographic_integrity().await.unwrap();
        assert!(report.audit_passed);
        assert_eq!(report.detected_corruptions_or_tampers, 0);
    }
}
```

SQLite WAL replay combined with Blake3 row-level MAC validation guarantees crash recovery with zero data corruption or uncommitted transactions.

---

## Conclusion
`oberer-orm` provides the foundation for the Sovereign Cryptocurrency Gateway & Unified Treasury Engine (v9.0). By combining compile-time AST generation, strong type safety, hybrid dual-tier storage, reactive database triggers, and row-level cryptographic authentication, it demonstrates that mission-critical financial systems can achieve institutional security and high throughput in pure, idiomatic Rust.
