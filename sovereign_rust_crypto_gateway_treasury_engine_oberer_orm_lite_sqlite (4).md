# Master Enterprise Specification, Mathematical Foundations & Sovereign Rust `Engine` Trait Architecture

**Unified Watch-Only Cryptocurrency Payment Gateway, Priority-Redundant In-Memory Cache (`IMPRCE`), Zero-Trust Enclave Custody & Unified Treasury System Powered by `oberer-orm` (Oberer-lite SQLite)**

* **Document ID:** `ARCH-RUST-GW-OBERER-SQLITE-2026-V9`

* **Version:** `9.0.0-OBERER-ORM-ZERO-RAW-SQL-DI-LIB`

* **Target Implementation Language:** Rust (`Edition 2024`, `rustc 1.85+`, `#![forbid(unsafe_code)]`, `#![deny(clippy::float_arithmetic, clippy::unwrap_used, clippy::indexing_slicing)]`)

* **Persistence & State Backbone:**

  1. **`oberer-orm` & `oberer-orm-derive` (Oberer-lite SQLite Engine):** Built from first principles on direct SQLite C FFI (`#[link(name = "sqlite3")]`) with zero `rusqlite` overhead, dedicated Thread-Queue Connection Actors (`OrmConn`), Dual-Tier Hybrid Persistence (`HybridConn` pairing `:memory:` L1 with WAL disk L2 via `DirtyTracker`), sharded write-behind caching (`TableCached<T>`), compile-time phantom-typed predicates (`Column<T, V>`), exact integer accounting (`MicroCredits` / `MoneyMicro`), optimistic concurrency control (`#[orm(version)]` + `update_with_version`), reactive triggers (`TableTrigger<T>`), read-only SQL views (`TableView<T>`), V001/V002 enterprise migrations (`MigrationPlanBuilder`, `TableSwapSpec`, `IdempotencyService`, `AuditRecordBuilder`, `EncryptedSecret`), and 4.0 diagnostic execution envelopes (`OrmOk<T>` / `.unenvelope()`).

  2. **`IMPRCE` (`gw-imprce`):** Sovereign In-Memory Priority-Redundant Cache Engine with $k$-way disjoint keyed memory mirrors, hardware `CRC32C` + `HMAC-SHA256` / `Blake3` dual integrity tags, and `ZeroizeOnDrop` memory hygiene—operating in tandem with `oberer-orm`'s `HybridConn` and `TableCached<T>`.

* **Architectural Mandate:**

  * **Security is Priority #1:** Zero private keys in Zone A or Zone B. All persistence records in `oberer-orm` carry cryptographic row-level MACs (`row_mac: Vec<u8>`) and hash-chained audit sequence links (`prev_chain_hash` $\rightarrow$ `entry_chain_hash`), reinforced by `oberer-orm` native `TableTrigger<T>` audit trails and `CHECK` constraints.

  * **Zero-Raw-SQL Compile-Time Schema, Index & View Enforcement (`Schema::<T>` Chainable DSL):** String-based SQL fragments in struct attributes (`#[orm(index = "...")]`, `#[orm(check = "...")]`, `#[orm(view_sql = "...")]`) are **strictly forbidden**. Every table migration, composite/partial index, `CHECK` invariant, foreign key cascade, read-only `TableView<V>`, and reactive `TableTrigger<T>` MUST be constructed exclusively through fluent, chainable, phantom-typed `Schema::<T>::new(conn)` and `ViewSchema::<V>::new(conn)` builder methods verified by `rustc` at compile time.

  * **Pure Object-Oriented `Engine` Trait Hierarchy & First-Class Library Dependency Injection (`DI`):** Every subsystem—including `ObererOrmStorageEngine`, `PriorityRedundantCacheEngine`, multi-chain watch-only derivation engines, 5-Gate typestate verification engines, FIFO treasury engines, and Zone C enclave signing engines—inherits from a unified, lifecycle-managed, polymorphic OOP `SovereignEngine` trait architecture that can be embedded as a standalone Rust library crate (`sovereign-gateway-sdk`) and wired via a type-safe **Dependency Injection (`DI`) Container & Fluent Builder**.

## Table of Contents

1. [Axiomatic Security Foundations & Zero-Trust Threat Proofs](#1-axiomatic-security-foundations--zero-trust-threat-proofs)

2. [Mathematical Foundations of `oberer-orm` (Oberer-lite) & Lexicographical \$\\mathbb{Z}\_{2^{256}}$ Algebra](#2-mathematical-foundations-of-oberer-orm-oberer-lite--lexicographical-mathbbz_2256-algebra)

   * 2.1 [Direct SQLite C FFI & Thread-Queue Actor Serialization Math (`OrmConn`)](#21-direct-sqlite-c-ffi--thread-queue-actor-serialization-math-ormconn)

   * 2.2 [Dual-Tier `HybridConn` (`:memory:` L1 + Disk WAL L2) & `DirtyTracker` Delta Calculus](#22-dual-tier-hybridconn-memory-l1--disk-wal-l2--dirtytracker-delta-calculus)

   * 2.3 [Bijective Order-Preserving Homomorphism for 256-Bit Token Amounts in `oberer-orm`](#23-bijective-order-preserving-homomorphism-for-256-bit-token-amounts-in-oberer-orm)

   * 2.4 [Dual-Scale Financial Algebra: `oberer_orm::MicroCredits` (\$10^6$) $\\times$ On-Chain $\\mathbb{Z}\_{2^{256}}$ ($10^d$)](#24-dual-scale-financial-algebra-oberer_ormmicrocredits-106-times-on-chain-mathbbz_2256-10d)

   * 2.5 [1000-Thread CAS Optimistic Concurrency Proof (`#[orm(version)]`) & Cryptographic Row MACs](#25-1000-thread-cas-optimistic-concurrency-proof-ormversion--cryptographic-row-macs)

3. [Mathematical Foundations of Multi-Chain Cryptography, Quote Locking & 5-Gate Verification](#3-mathematical-foundations-of-multi-chain-cryptography-quote-locking--5-gate-verification)

   * 3.1 [Multi-Curve Watch-Only Derivation Algebra (BIP-84, `CREATE2`, SVM PDA, TON Feistel Memo)](#31-multi-curve-watch-only-derivation-algebra-bip-84-create2-svm-pda-ton-feistel-memo)

   * 3.2 [Dual-Oracle Circuit Breaker & Clean 18-Decimal Truncation](#32-dual-oracle-circuit-breaker--clean-18-decimal-truncation)

   * 3.3 [DAG Parent-Hash Continuity, LCA Reorg Rollback & 5-Gate Typestate Algebra](#33-dag-parent-hash-continuity-lca-reorg-rollback--5-gate-typestate-algebra)

4. [Object-Oriented `Engine` Trait Hierarchy & `oberer-orm` Integration Architecture](#4-object-oriented-engine-trait-hierarchy--oberer-orm-integration-architecture)

5. [Complete Enterprise Trait & `oberer-orm` Specification Catalog](#5-complete-enterprise-trait--oberer-orm-specification-catalog)

   * 5.1 [Core Sovereign Primitives, `MicroCredits` Bridge & `U256` Codec (`gw-primitives`)](#51-core-sovereign-primitives-microcredits-bridge--u256-codec-gw-primitives)

   * 5.2 [Root OOP Lifecycle Trait (`SovereignEngine`) & `ObererOrmStorageEngine`](#52-root-oop-lifecycle-trait-sovereignengine--obererormstorageengine)

   * 5.3 [Engine Trait I: `PriorityRedundantCacheEngine` (`IMPRCE` \$\\leftrightarrow$ `HybridConn` / `TableCached`)](#53-engine-trait-i-priorityredundantcacheengine-imprce-leftrightarrow-hybridconn--tablecached)

   * 5.4 [Engine Trait II: `WatchOnlyDerivationEngine` (Deterministic Route & Pool Leasing)](#54-engine-trait-ii-watchonlyderivationengine-deterministic-route--pool-leasing)

   * 5.5 [Engine Trait III: `ConsensusIndexingEngine` (DAG Continuity, Bloom & O(1) Match)](#55-engine-trait-iii-consensusindexingengine-dag-continuity-bloom--o1-match)

   * 5.6 [Engine Trait IV: `StateBalanceReaderEngine` (Proof-Verified Balance & Resource Quotas)](#56-engine-trait-iv-statebalancereaderengine-proof-verified-balance--resource-quotas)

   * 5.7 [Engine Trait V: `UnsignedSweepAssemblerEngine` (Dry-Run, RBF/CPFP & Gas Sponsor)](#57-engine-trait-v-unsignedsweepassemblerengine-dry-run-rbfcpfp--gas-sponsor)

   * 5.8 [Engine Trait VI: `UnifiedNetworkAdapterEngine` (Master Polymorphic Blockchain Engine)](#58-engine-trait-vi-unifiednetworkadapterengine-master-polymorphic-blockchain-engine)

   * 5.9 [Engine Trait VII: `InvoiceVerificationEngine` (5-Gate Typestate + `oberer-orm` CAS)](#59-engine-trait-vii-invoiceverificationengine-5-gate-typestate--oberer-orm-cas)

   * 5.10 [Engine Trait VIII: `PreSignPolicyGuardEngine` (Zone C Stateless AST/Bytecode Verifier)](#510-engine-trait-viii-presignpolicyguardengine-zone-c-stateless-astbytecode-verifier)

   * 5.11 [Engine Trait IX: `EnclaveKeySignerEngine` (Zeroized RAM & M-of-N Quorum Signer)](#511-engine-trait-ix-enclavekeysignerengine-zeroized-ram--m-of-n-quorum-signer)

   * 5.12 [Engine Trait X: `TreasuryPortfolioEngine` (Unified Valuation & FIFO Lot Calculus)](#512-engine-trait-x-treasuryportfolioengine-unified-valuation--fifo-lot-calculus)

   * 5.13 [Engine Trait XI: `CrossNetworkLiquidityRouterEngine` (Optimal Transport & Gas Refill)](#513-engine-trait-xi-crossnetworkliquidityrouterengine-optimal-transport--gas-refill)

   * 5.14 [Master Orchestrator: `SovereignGatewayRuntimeEngine` (Composition Root)](#514-master-orchestrator-sovereigngatewayruntimeengine-composition-root)

6. [Zero-Raw-SQL Type-Enforced `Schema<T>` Chainable Migration, Index, View & Trigger Engine](#6-zero-raw-sql-type-enforced-schemat-chainable-migration-index-view--trigger-engine)

   * 6.1 [Mathematical Theory of Phantom-Typed Relational Algebra & Zero-Raw-SQL DDL](#61-mathematical-theory-of-phantom-typed-relational-algebra--zero-raw-sql-ddl)

   * 6.2 [The Fluent Chainable `Schema<T>`, `IndexSchema<T>`, `ViewSchema<V>` & `TriggerSchema<T>` DSL](#62-the-fluent-chainable-schemat-indexschemat-viewschemav--triggerschemat-dsl)

   * 6.3 [Domain Enums (`#[derive(OrmEnum)]`) & Pure Structural Models (`#[derive(OrmTable)]` — Zero Raw SQL)](#63-domain-enums-deriveormenum--pure-structural-models-deriveormtable--zero-raw-sql)

   * 6.4 [Type-Safe View Structs (`TableView<V>`) & Chainable `ViewSchema<V>` Projections](#64-type-safe-view-structs-tableviewv--chainable-viewschemav-projections)

   * 6.5 [Reactive Audit Triggers (`TriggerSchema<T>`) & Atomic Release Execution (`MigrationPlanBuilder`)](#65-reactive-audit-triggers-triggerschemat--atomic-release-execution-migrationplanbuilder)

7. [Using as a Rust Library Crate (`lib.rs`) & Dependency Injection (`DI`) Architecture](#7-using-as-a-rust-library-crate-librs--dependency-injection-di-architecture)

   * 7.1 [Mathematical & Architectural Theory of Inversion of Control (`IoC`) & Zone-Isolated DI](#71-mathematical--architectural-theory-of-inversion-of-control-ioc--zone-isolated-di)

   * 7.2 [Consuming as a Library Crate: `Cargo.toml`, Feature Flags & `prelude` Facade (`src/lib.rs`)](#72-consuming-as-a-library-crate-cargotoml-feature-flags--prelude-facade-srclibrs)

   * 7.3 [Concrete Engine Implementations for DI Instantiation (`ObererHybridStorageEngineInstance`, etc.)](#73-concrete-engine-implementations-for-di-instantiation-obererhybridstorageengineinstance-etc)

   * 7.4 [Type-Safe DI Container (`SovereignDiContainer`) & Fluent IoC Builder (`SovereignGatewayDiBuilder`)](#74-type-safe-di-container-sovereigndicontainer--fluent-ioc-builder-sovereigngatewaydibuilder)

   * 7.5 [Injecting Engine Instances into `Axum` Web Services (`FromRef` Sub-State Extraction)](#75-injecting-engine-instances-into-axum-web-services-fromref-sub-state-extraction)

   * 7.6 [Unit & Integration Testing with Mock / `:memory:` DI Engine Swapping](#76-unit--integration-testing-with-mock--memory-di-engine-swapping)

8. [Formal Verification Matrix, 1000-Thread CAS Proof & Institutional Security Audit Checklist](#8-formal-verification-matrix-1000-thread-cas-proof--institutional-security-audit-checklist)

## 1. Axiomatic Security Foundations & Zero-Trust Threat Proofs

### 1.1 Formal Three-Zone Execution Partitioning

Let the total distributed gateway and custody system $\mathcal{S}$ be partitioned into three physically and logically isolated execution domains:

$$
\mathcal{S} = \mathcal{Z}_A \sqcup \mathcal{Z}_B \sqcup \mathcal{Z}_C
$$

1. **Public Watch-Only & Persistence Plane (**$\mathcal{Z}_A$**):**
   Executes the checkout quote engine, the **In-Memory Priority-Redundant Cache Engine (`IMPRCE`)**, the **5-Gate Typestate Invoice Verification Engine**, the **Unified Treasury & FIFO Portfolio Engine**, and our company's pure-Rust embedded ORM **`oberer-orm` (`HybridConn` `:memory:` L1 + Disk WAL L2)**.

2. **Isolated Native Node & Indexing Plane (**$\mathcal{Z}_B$**):**
   Executes full verification nodes (`bitcoind`, `reth`/`geth`, `solana-validator`, `java-tron`, `ton-lite-server`, `sui-node`) inside an ingress-restricted private subnet.

3. **Air-Gapped / Hardware-Attested Signing Enclave Plane (**$\mathcal{Z}_C$**):**
   Executes the stateless `PreSignPolicyGuardEngine` and `EnclaveKeySignerEngine` inside an AWS Nitro Enclave / HSM / MPC Quorum communicating exclusively over mutually authenticated VSOCK channels.

### 1.2 Nine Non-Negotiable Mathematical Security Axioms

#### Axiom 1 (Strict Zero-Key Isolation in $\mathcal{Z}_A \cup \mathcal{Z}_B$)

Let $\mathcal{K}_{\text{priv}}$ denote the set of all private signing keys, HD master seeds, MPC secret shares, and KMS signing credentials. Let $\text{Mem}_t(\mathcal{Z})$ and $\text{Disk}_t(\mathcal{Z})$ denote the addressable RAM (`HybridConn::mem()`, `TableCached<T>`, `IMPRCE`) and persistent SQLite database pages (`HybridConn` disk tier) of zone $\mathcal{Z}$ at time $t$:

$$
\forall t \in \mathbb{T}, \quad \left(\text{Mem}_t(\mathcal{Z}_A \cup \mathcal{Z}_B) \cup \text{Disk}_t(\mathcal{Z}_A \cup \mathcal{Z}_B)\right) \cap \mathcal{K}_{\text{priv}} = \emptyset
$$

Only extended public keys (`BIP-84 zpub`), counterfactual contract creation parameters (`factory`, `init_code_hash`), and watch-only descriptors exist in $\mathcal{Z}_A \cup \mathcal{Z}_B$.

#### Axiom 2 (Cryptographic Immutability of Counterfactual Sweep Destinations)

For all smart-contract account networks $\mathcal{C}_{\text{SC}} \in \{\text{Ethereum}, \text{BNB}, \text{Plume}, \text{Tron}\}$, let $B_{\text{init}}(v_{\text{cold}})$ be the initialization bytecode of a customer deposit receiver parameterized by the Cold Treasury address $v_{\text{cold}} \in \mathcal{W}_{\text{cold}}$. Because $v_{\text{cold}}$ is embedded directly into the runtime bytecode instructions (`PUSH20 <v_cold>`) with zero state-mutating opcodes (`SSTORE` is unreachable for destination modification), for any arbitrary transaction $tx$ signed by any compromised relayer key $k_{\text{relayer}}$:

$$
\forall tx \in \text{Tx}(\mathcal{C}_{\text{SC}}), \quad \text{Exec}(B_{\text{runtime}}, tx) \implies \text{Recipient}(\Delta \text{Balance}) \equiv v_{\text{cold}}
$$

#### Axiom 3 (Stateless Pre-Sign AST & Bytecode Policy Gate in $\mathcal{Z}_C$)

Let $\mathcal{P}_{\text{unsigned}} \in \{0,1\}^*$ be an arbitrary raw unsigned transaction byte sequence arriving at $\mathcal{Z}_C$. Let $\text{Parse}_{\mathcal{C}} : \{0,1\}^* \to \mathcal{P}(\text{Addr} \times \mathbb{Z}_{\ge 0}) \cup \{\bot\}$ be a total, deterministic parser extracting all non-change output pairs $(a_i, v_i)$ and the total network fee $f_{\text{tx}}$ directly from the raw PSBT/RLP/SVM/TVM/BOC/BCS bytes. Let $\mathcal{W}_{\text{pinned}} \subset \text{Addr}$ be the immutable hardware-pinned destination whitelist baked into the enclave image measurement $\text{PCR}_0$. The signing operation is gated by $\Pi_{\text{guard}}$:

$$
\Pi_{\text{guard}}(\mathcal{P}_{\text{unsigned}}, \Sigma_{\text{M-of-N}}) = 1 \iff \begin{cases}
\text{Parse}_{\mathcal{C}}(\mathcal{P}_{\text{unsigned}}) \neq \bot \\
\forall (a_i, v_i) \in \text{Outputs}(\mathcal{P}_{\text{unsigned}}), \; a_i \in \mathcal{W}_{\text{pinned}} \\
f_{\text{tx}} \le \text{MaxFee}_{\mathcal{C}} \;\land\; \text{VerifyQuorum}(\Sigma_{\text{M-of-N}}, \tau_{\text{tier}}) = 1
\end{cases}
$$

$$
\Pi_{\text{guard}}(\mathcal{P}_{\text{unsigned}}, \Sigma_{\text{M-of-N}}) = 0 \implies \text{Access}(\mathcal{K}_{\text{priv}}) = \text{FORBIDDEN}
$$

#### Axiom 4 (Ephemeral RAM Zeroization Lifecycle)

Every secret buffer $m \in \text{Mem}_t(\mathcal{Z}_C)$ and every cached sensitive cell $c \in \text{Mem}_t(\mathcal{Z}_A)$ implements a volatile write barrier (`ZeroizeOnDrop`) that overwrites all bytes $m[0..L]$ with `0x00` prior to heap deallocation:

$$
\forall j \in \{0, \dots, L-1\}, \quad \text{VolatileWrite}(m[j], \text{0x00}) \quad \text{such that} \quad \forall t > t_{\text{drop}}, \; m[j] = 0
$$

#### Axiom 5 (Closed Semiring Integer Arithmetic)

All fiat valuations use `oberer_orm::MicroCredits` / `MoneyMicro` ($1.00\text{ USD} = 1,000,000\text{ microcredits}$, backed by exact `i64` with `#[orm(micro_credits, non_negative)]`), and all raw on-chain token quantities operate strictly over the bounded unsigned 256-bit integer ring $\mathbb{Z}_{2^{256}} = [0, 2^{256}-1]$ with checked partial operators $(\oplus, \ominus, \otimes, \oslash)$. IEEE-754 floating-point representations (`f32`, `f64`) are forbidden across all workspace crates (`#![deny(clippy::float_arithmetic)]`).

#### Axiom 6 (Linear Typestate Monotonicity & Compile-Time Column Safety)

1. An invoice entity $I$ is parameterized at the Rust type level by its lifecycle state $S \in \{\text{StatePending}, \text{StateDetected}, \text{StatePartial}, \text{StateVerifiedComplete}, \text{StateExpired}\}$.

2. All `oberer-orm` database queries, migrations, indexes, `CHECK` constraints, views, and triggers use phantom-typed column accessors `Column<T, V>` emitted by `#[derive(OrmTable)]`, making type-mismatched SQL predicates or unfiltered batch mutations (`OrmError::NoFilterOnMutation`) impossible.

#### Axiom 7 (Priority-Redundant Cache Survivability)

In-memory state in `IMPRCE` is partitioned into five strict priority tiers $\mathcal{P} = \{P_0, P_1, P_2, P_3, P_4\}$ with hard per-tier byte quota fences $\mathcal{Q}(p)$ and $k(p) \in \{1, 2, 3\}$ disjoint keyed mirror arenas. Any adversarial flood on low-priority tiers ($P_3, P_4$) can never evict consensus headers ($P_0$), active payment watchlists ($P_1$), or rotating pool address leases ($P_2$).

#### Axiom 8 (Cryptographic Row MACs, Reactive Audit Triggers & Hash-Chained Ledgers)

Every critical struct persisted via `oberer-orm` (`#[derive(OrmTable)]`) includes:

1. `#[orm(version)] pub version: i64` for lock-free Optimistic Concurrency Control (`table.update_with_version(pk, version, &row)`).

2. `pub row_mac: Vec<u8>` holding a 32-byte keyed Blake3/HMAC tag binding the primary key, `version`, and all financial columns.

3. Reactive SQLite triggers built via `TriggerSchema::<T>::new(conn)` that automatically record state transitions into an immutable audit table alongside `oberer_orm::runtime::AuditRecordBuilder`.

#### Axiom 9 (Zero-Raw-SQL Compile-Time DDL & View Construction)

Let $\mathcal{L}_{\text{DDL}}$ be the set of all schema migration statements (`CREATE TABLE`, `CREATE INDEX`, `CREATE UNIQUE INDEX ... WHERE`, `CREATE VIEW`, `CREATE TRIGGER`). No element of $\mathcal{L}_{\text{DDL}}$ may be constructed from an untyped string literal. Every DDL artifact is a pure function of typed column projections $\pi_{\text{col}} : \text{Struct } T \to \text{Column}\langle T, V \rangle$ and typed predicate trees $\mathcal{P}\langle T \rangle$:

$$
\forall s \in \mathcal{L}_{\text{DDL}}, \quad s = \text{Compile}_{\text{AST}}\left(\text{Schema}\langle T \rangle, \; \{ \text{Column}\langle T, V_i \rangle \}, \; \text{Predicate}\langle T \rangle\right)
$$

## 2. Mathematical Foundations of `oberer-orm` (Oberer-lite) & Lexicographical $\mathbb{Z}_{2^{256}}$ Algebra

### 2.1 Direct SQLite C FFI & Thread-Queue Actor Serialization Math (`OrmConn`)

Unlike generic Rust ORMs that layer over `rusqlite` or async connection pools subject to `SQLITE_BUSY` lock-convoy collapses, `oberer-orm` binds directly to the native C API (`sqlite3_open_v2`, `sqlite3_prepare_v2`, `sqlite3_step`, `sqlite3_finalize`) inside an isolated **Thread-Queue Actor (`OrmConn`)**:

* Each `OrmConn` owns a dedicated background worker thread holding the exclusive `*mut sqlite3` pointer.

* Application threads (`Clone + Send + Sync`) submit command envelopes over a lock-free `mpsc` channel and await results on a one-shot reply channel.

**Queueing & Contention Elimination Theorem:** Let $N_{\text{threads}}$ application threads concurrently issue database mutations at aggregate rate $\lambda$. In traditional multi-connection SQLite pools, concurrent writers contend on the OS file lock, incurring exponential backoff and `SQLITE_BUSY` failures when $\lambda \to \mu_{\text{lock}}$. In `oberer-orm`'s Actor model, all mutations are serialized in FIFO arrival order through the `mpsc` queue with zero lock contention ($P(\text{SQLITE\_BUSY}) \equiv 0$). For an in-memory working connection (`OrmConn::new(":memory:")` inside `HybridConn`), the service time per prepared statement is $\tau_{\text{mem}} \approx 1.4\text{ }\mu\text{s}$, yielding a theoretical single-actor throughput of:

$$
\mu_{\text{actor}} = \frac{1}{\tau_{\text{mem}}} \approx 714,285\text{ operations/sec}
$$

### 2.2 Dual-Tier `HybridConn` (`:memory:` L1 + Disk WAL L2) & `DirtyTracker` Delta Calculus

`oberer-orm` provides `HybridConn::new(":memory:", disk_path, SyncPolicy)`, which pairs an ultra-fast `:memory:` SQLite working database ($L_1$) with a persistent `WAL`-mode disk database ($L_2$).

Let $\mathcal{M}_t$ be the set of rows in the `:memory:` working database at time $t$, and let $\mathcal{D}_t = (\Delta_{\text{upsert}}, \Delta_{\text{delete}})$ be the state of `oberer-orm`'s internal `DirtyTracker`, where:

* $\Delta_{\text{upsert}} \subset \text{PK}$ is the set of primary keys inserted or updated since the last sync ($t_{\text{sync}}$).

* $\Delta_{\text{delete}} \subset \text{PK}$ is the set of primary keys deleted since $t_{\text{sync}}$, with $\Delta_{\text{upsert}} \cap \Delta_{\text{delete}} = \emptyset$.

**Write-Coalescing Compression Ratio (**$\eta_{\text{sync}}$**):** Suppose an active invoice or wallet row $k \in \text{PK}$ undergoes $m \ge 1$ state transitions in `:memory:` during a sync window $\Delta t$ (e.g., `Pending` $\to$ `Detected (1 conf)` $\to$ `Detected (2 conf)` $\to$ `Completed`). In $\mathcal{D}_t$, all $m$ intermediate mutations coalesce into a single dirty key entry $k \in \Delta_{\text{upsert}}$. When `hybrid.sync()` executes under `SyncPolicy::Manual`, `SyncPolicy::OnDirty(N)`, or `SyncPolicy::Periodic(secs)`:

$$
\text{DiskWrites}(k) = 1 \quad \text{instead of} \quad m \implies \eta_{\text{sync}} = \frac{\sum_{k \in \Delta_{\text{upsert}}} m(k)}{\left|\Delta_{\text{upsert}}\right| + \left|\Delta_{\text{delete}}\right|} \ge 1
$$

All dirty rows are flushed to $L_2$ inside a single atomic disk transaction (`INSERT OR REPLACE` for $k \in \Delta_{\text{upsert}}$ and `DELETE WHERE pk = ?` for $k \in \Delta_{\text{delete}}$), combining sub-microsecond RAM execution with ACID disk durability.

### 2.3 Bijective Order-Preserving Homomorphism for 256-Bit Token Amounts in `oberer-orm`

In `oberer-orm`, Rust struct fields map to SQLite column affinities as follows:

* `i64` $\to$ `INTEGER`

* `String` $\to$ `TEXT`

* `Vec<u8>` $\to$ `BLOB`

While fiat values (`USD`) fit cleanly in `oberer_orm::MicroCredits` (`i64`, where $\max(\text{i64}) \approx 9.22 \times 10^{18}\text{ microcredits} = \$9.22\text{ trillion USD}$), raw on-chain token balances on 18-decimal chains (`WEI` on Ethereum, BNB Chain, Plume) require full 256-bit unsigned integers ($\mathbb{Z}_{2^{256}}$). To enable `oberer-orm`'s compile-time typed query engine (`Column<T, Vec<u8>>::gt()`, `.gte()`, `.lte()`, `.between()`) and `Schema::<T>` typed `CHECK` constraints to compare 256-bit integers natively, we use two bijective order-preserving encodings:

1. **32-Byte Big-Endian `Vec<u8>` (`BLOB`) Encoding (**$\Psi_{\text{blob}}$**):**

   $$
   \Psi_{\text{blob}}(x) = \text{vec!}[b_0, b_1, \dots, b_{31}] \in \text{Vec}\langle\text{u8}\rangle, \qquad b_i = \left\lfloor \frac{x}{256^{31-i}} \right\rfloor \bmod 256
   $$

   Because SQLite evaluates `>` / `<` / `<=` / `BETWEEN` on equal-length 32-byte `BLOB` columns via unsigned byte-by-byte `memcmp`:

   $$
   \forall x, y \in \mathbb{Z}_{2^{256}}, \quad x \le y \iff \Psi_{\text{blob}}(x) \le_{\text{SQLite BLOB}} \Psi_{\text{blob}}(y)
   $$

   Therefore, a type-safe `Schema::<CostBasisLotRow>` constraint `.check(CostBasisLotRow::remaining_amount_blob().blob_len_eq(32).and(CostBasisLotRow::remaining_amount_blob().lte_col(CostBasisLotRow::initial_amount_blob())))` enforces exact 256-bit conservation at the SQLite kernel level without a single raw SQL string!

2. **78-Character Zero-Padded `String` (`TEXT`) Encoding (**$\Psi_{\text{dec78}}$**):**

   $$
   \Psi_{\text{dec78}}(x) = \text{format!}(\text{"{:0>78}"}, x) \in \text{String}
   $$

   Preserves ASCII lexicographical ordering (`Column<T, String>::gte()`) and human auditability in SQLite views (`TableView<T>`).

### 2.4 Dual-Scale Financial Algebra: `oberer_orm::MicroCredits` ($10^6$) $\times$ On-Chain $\mathbb{Z}_{2^{256}}$ ($10^d$)

`oberer-orm` provides `MicroCredits` and `MoneyMicro` (`#[orm(micro_credits, non_negative)]`), representing exact fixed-point fiat values scaled by $10^6$ ($1.00\text{ USD} = 1,000,000\text{ }\mu\text{credits}$) stored as non-negative `i64` integers (`CHECK(col >= 0)`).

Let $F_{\text{mc}} \in \text{MicroCredits}$ (with $0 \le F_{\text{mc}}.\text{get}() \le 9.22 \times 10^{18}$) be the invoice fiat amount or locked asset price in USD microcredits, and let $R \in \mathbb{Z}_{2^{256}}$ be the raw on-chain token amount with physical decimals $d \in [0, 36]$. The exact algebraic bridge between `oberer_orm::MicroCredits` and $\mathbb{Z}_{2^{256}}$ is:

1. **Fiat `MicroCredits` to Clean Locked Raw Token Units (**$\mathcal{Q}_{\text{quote}}$**):** Given invoice fiat amount $F_{\text{mc}}$ and locked price $P_{\text{mc}}$ (both in `MicroCredits`):

   $$
   R_{\text{exact}} = \left\lfloor \frac{\text{U256}(F_{\text{mc}}.\text{get}()) \otimes 10^d}{\text{U256}(P_{\text{mc}}.\text{get}())} \right\rfloor
   $$

2. **Raw Token Units to Fiat `MicroCredits` Valuation (**$\mathcal{V}_{\text{mc}}$**):**

   $$
   \mathcal{V}_{\text{mc}}(R, d, P_{\text{mc}}) = \text{MicroCredits::new}\left( \text{i64::try\_from}\left( \left\lfloor \frac{R \otimes \text{U256}(P_{\text{mc}}.\text{get}())}{10^d} \right\rfloor \right) \right)
   $$

### 2.5 1000-Thread CAS Optimistic Concurrency Proof (`#[orm(version)]`) & Cryptographic Row MACs

In `oberer-orm`, annotating a struct field with `#[orm(version)] pub version: i64` emits `version INTEGER NOT NULL DEFAULT 1 CHECK(version >= 1)` and enables atomic Compare-And-Swap updates via:

```
table.update_with_version(pk, expected_version, &updated_row)

```

which compiles to the atomic SQLite statement:

```
UPDATE <table> SET ..., version = version + 1 WHERE <pk> = ?1 AND version = ?2;

```

If `rows_affected == 0`, `oberer-orm` immediately returns `Err(OrmError::OptimisticLockConflict)`.

**Binding Row-Level Cryptographic MAC (**$\tau_{\text{row}}$**) to `#[orm(version)]`:** When transitioning state from version $\nu$ to $\nu + 1$, the cryptographic row MAC stored in `pub row_mac: Vec<u8>` is computed over the **target post-update version** $\nu + 1$:

$$
\tau_{\text{row}}(E, \nu + 1) = \text{Blake3}_{\mathcal{K}_{\text{db}}}\left( \text{"OBERER\_ORM\_V9"} \parallel \text{TableName} \parallel \text{PK}(E) \parallel \text{Ser}_{64}(\nu + 1) \parallel \text{CanonicalFields}(E) \right)
$$

Because `update_with_version` increments `version` to $\nu + 1$ in SQLite while writing `row_mac` = $\tau_{\text{row}}(E, \nu + 1)$, any subsequent `table.find(pk)` or `table.query_typed().fetch()` verifies that the stored `version` and `row_mac` match in constant time (`subtle::ConstantTimeEq`).

## 3. Mathematical Foundations of Multi-Chain Cryptography, Quote Locking & 5-Gate Verification

### 3.1 Multi-Curve Watch-Only Derivation Algebra (BIP-84, `CREATE2`, SVM PDA, TON Feistel Memo)

1. **Bitcoin Native SegWit (`BIP-84` Non-Hardened Child Derivation):** From account-level extended public key $(K_{\text{ext}}, c_{\text{ext}})$ at `m/84'/0'/0'/0` and invoice index $i \in [0, 2^{31}-1]$:

   $$
   I = \text{HMAC-SHA512}\left(c_{\text{ext}}, \; \text{Ser}_{33}(K_{\text{ext}}) \parallel \text{Ser}_{32}(i)\right), \qquad K_i = \text{parse}_{256}(I[0..32]) \cdot G + K_{\text{ext}}
   $$

   $$
   \text{Addr}_{\text{BTC}}(i) = \text{Bech32Encode}\left(\text{"bc"}, \; 0, \; \text{RIPEMD160}(\text{SHA256}(\text{Ser}_{33}(K_i)))\right)
   $$

2. **EVM (`Ethereum`, `BNB`, `Plume`) & TVM (`Tron`) Counterfactual `CREATE2` Proxy Algebra:** With prefix $\pi(\mathcal{C}) = \text{0xff}$ for EVM and $\pi(\text{Tron}) = \text{0x41}$, factory $A_{\text{factory}}$, salt $s \in \{0,1\}^{256}$, and immutable-treasury init code hash $H_{\text{init}} = \text{Keccak256}(B_{\text{init}}(v_{\text{cold}}))$:

   $$
   A_{\text{raw}}(s) = \text{Keccak256}\left(\pi(\mathcal{C}) \parallel A_{\text{factory}} \parallel s \parallel H_{\text{init}}\right)[12..32]
   $$

   On **Tron (`TVM`)**, sweeping `TRC-20` tokens enforces the **1-Sun Dust Keeper Invariant** ($V_{\text{sweep}} = B_{\text{current}} \ominus 1$), keeping the `TRC-20` storage slot warm and saving $\approx 65,000\text{ Energy}$ on every future deposit.

3. **Solana (`SVM`) Dual-Mode Routing & Rent Reclamation:** Direct wallet checkouts attach a deterministic read-only `AccountMeta` reference public key $R_i \in \mathbb{E}_{\text{Ed25519}}$ to transfers targeting $A_{\text{ATA}}^{\text{cold}}$ ($0$ sweep cost). CEX withdrawals route to an off-curve PDA $A_{\text{PDA}}(s, b^*) \notin \mathbb{E}_{\text{Ed25519}}$, swept via `sweep_and_close_ata` to reclaim $2,039,280\text{ lamports}$ rent.

4. **TON Sharded Actor Jetton Verification & 40-Bit Feistel Memo Bijection:** Incoming `op::transfer_notification` (`0x7362d09c`) messages to Cold Vault $V_{\text{cold}}$ are authentic iff $\text{Src}(m) \equiv A_{\text{JettonWallet}}(V_{\text{cold}})$. Each invoice $i \in [0, 2^{40}-1]$ maps bijectively via a 40-bit Feistel permutation $\mathcal{F}_{K_{\text{memo}}}$ to an 8-character Crockford Base32 memo tag $\text{MemoTag}(i)$.

### 3.2 Dual-Oracle Circuit Breaker & Clean 18-Decimal Truncation

1. **Dual-Oracle Basis-Point Divergence Circuit Breaker (**$\delta_{\text{bps}} \le 150$**):** Given CEX WebSocket price $P_{\text{cex}} \in \text{MicroCredits}$ and on-chain Chainlink/Pyth price $P_{\text{chain}} \in \text{MicroCredits}$:

   $$
   \delta_{\text{bps}}\left(P_{\text{cex}}, P_{\text{chain}}\right) = \left\lfloor \frac{\left| P_{\text{cex}}.\text{get}() - P_{\text{chain}}.\text{get}() \right| \times 10,000}{\min\left(P_{\text{cex}}.\text{get}(), P_{\text{chain}}.\text{get}()\right)} \right\rfloor \le 150\text{ bps}
   $$

2. **18-Decimal Dust Truncation Operator (**$\mathcal{T}_{\text{clean}}$**):** Zeroes out the bottom 12 digits ($10^{12}$) on 18-decimal tokens so customers see a clean 6-decimal figure at checkout:

   $$
   R_{\text{expected}} = \mathcal{T}_{\text{clean}}(R_{\text{exact}}, d) = \begin{cases} \left\lfloor \dfrac{R_{\text{exact}}}{10^{12}} \right\rfloor \otimes 10^{12} & \text{if } d = 18 \\ R_{\text{exact}} & \text{if } d \neq 18 \end{cases}
   $$

### 3.3 DAG Parent-Hash Continuity, LCA Reorg Rollback & 5-Gate Typestate Algebra

Given active invoice $I$ and candidate transfer event $e$, settlement requires all five orthogonal predicates to hold simultaneously:

$$
G_{\text{total}}(I, e, \mathcal{B}_{\mathcal{C}}) \iff G_1(e) \;\land\; G_2(I, e) \;\land\; G_3(I, e) \;\land\; G_4(I, e) \;\land\; G_5(e, \mathcal{B}_{\mathcal{C}})
$$

where:

* **Gate 1 (**$G_1$**):** $e.\sigma_{\text{exec}} \equiv \text{true}$ (On-chain execution succeeded without revert).

* **Gate 2 (**$G_2$**):** $(e.c_{\text{id}} \equiv I.a.\text{chain\_id}) \land (\text{Lower}(e.C_{\text{token}}) \equiv \text{Lower}(I.a.\text{token\_contract})) \land ((e.c_{\text{id}}, e.C_{\text{token}}) \in \mathcal{R}_{\text{signed}})$.

* **Gate 3 (**$G_3$**):** $R_{\text{rec}} \oplus e.\Delta B_{\text{net}} \ge \left\lfloor \frac{R_{\text{exp}} \otimes (10,000 \ominus \beta_{\text{bps}})}{10,000} \right\rfloor$.

* **Gate 4 (**$G_4$**):** $e.T_{\text{block}} \le I.T_{\text{exp}}$.

* **Gate 5 (**$G_5$**):** $\left(P_{e.h} \equiv H_{e.h - 1}\right) \land \left(e.N_{\text{conf}} \ge e.N_{\text{target}}\right)$.

## 4. Object-Oriented `Engine` Trait Hierarchy & `oberer-orm` Integration Architecture

```
╔══════════════════════════════════════════════════════════════════════════════════════════════╗
║                         ROOT OOP TRAIT: `SovereignEngine`                                    ║
║   + engine_id(&self) -> &'static str                                                         ║
║   + security_zone(&self) -> SecurityZone                                                     ║
║   + initialize_engine(&self) -> Result<(), SovereignError>                                   ║
║   + audit_cryptographic_integrity(&self) -> Result<EngineIntegrityReport, SovereignError>    ║
║   + health_snapshot(&self) -> Result<EngineHealthSnapshot, SovereignError>                   ║
║   + shutdown_gracefully(&self) -> Result<(), SovereignError>                                 ║
╚══════════════════════════════════════════════╤═══════════════════════════════════════════════╝
                                               │
       ┌───────────────────────┬───────────────┴───────────────┬────────────────────────┐
       ▼                       ▼                               ▼                        ▼
┌─────────────────────┐ ┌─────────────────────────┐ ┌─────────────────────────┐ ┌──────────────────────┐
│ `ObererOrmStorage   │ │ `PriorityRedundantCache │ │ `UnifiedNetworkAdapter  │ │ `EnclaveKeySigner    │
│  Engine`            │ │  Engine` (IMPRCE)       │ │  Engine` (Subtraits:)   │ │  Engine` (Zone C)    │
│ • Direct SQLite FFI │ │ • k-Way Keyed Mirrors   │ │ ├─WatchOnlyDerivation   │ │ • PreSignPolicyGuard │
│ • OrmConn Actor     │ │ • CRC32C + HMAC Tags    │ │ ├─ConsensusIndexing     │ │ • ZeroizeOnDrop RAM  │
│ • HybridConn L1/L2  │ │ • 256-Bit Bloom Filter  │ │ ├─StateBalanceReader    │ │ • M-of-N Quorum      │
│ • Schema<T> Typed   │ │ • Hydrated via Hybrid   │ │ └─UnsignedSweepAssembler│ │ • PCR0 Attestation   │
│   Views/Triggers/Idx│ │   warm_table::<T>()     │ │                         │ │                      │
└──────────┬──────────┘ └────────────┬────────────┘ └────────────┬────────────┘ └──────────▲───────────┘
           │                         │                           │                         │
           └─────────────────────────┼───────────────────────────┘                         │
                                     ▼                                                     │
           ┌───────────────────────────────────────────────────────────┐                   │
           │  DOMAIN ORCHESTRATION ENGINES & DI CONTAINER              │                   │
           │  • `InvoiceVerificationEngine` (5-Gate + CAS + Idempot.)  │                   │
           │  • `TreasuryPortfolioEngine` (MicroCredits + FIFO Lots)   │───────────────────┘
           │  • `CrossNetworkLiquidityRouterEngine` (Cold/Market Split)│  VSOCK Unsigned
           │  • `SovereignDiContainer` + `SovereignGatewayDiBuilder`   │  Envelope Stream
           └───────────────────────────────────────────────────────────┘

```

## 5. Complete Enterprise Trait & `oberer-orm` Specification Catalog

### 5.1 Core Sovereign Primitives, `MicroCredits` Bridge & `U256` Codec (`gw-primitives`)

```
// crates/gw-primitives/src/lib.rs
#![forbid(unsafe_code)]
#![deny(
    clippy::float_arithmetic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::unwrap_used,
    clippy::indexing_slicing
)]

use alloy_primitives::U256;
use oberer_orm::{MicroCredits, OrmError, OrmFailure};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Security execution zones enforcing Axiom 1 and Axiom 3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SecurityZone {
    /// Zone A: Public DMZ, Quote Engine, IMPRCE, oberer-orm HybridConn (Zero Private Keys).
    ZoneAPublicWatchOnly,
    /// Zone B: Isolated Native Blockchain Node & Indexing Plane (Zero Private Keys).
    ZoneBPrivateNodePlane,
    /// Zone C: Air-Gapped / Hardware-Attested Signing Enclave (Stateless Policy Guard + Zeroized Keys).
    ZoneCZeroTrustEnclave,
}

/// Exhaustive algebraic, `oberer-orm`, cache, consensus, DI, and enclave security error taxonomy.
#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize, Deserialize)]
pub enum SovereignError {
    #[error("Checked 256-bit or 64-bit integer overflow in context: {context}")]
    Overflow { context: &'static str },
    #[error("Checked 256-bit or 64-bit integer underflow in context: {context}")]
    Underflow { context: &'static str },
    #[error("Division or modulo by zero in fixed-point calculation")]
    DivisionByZero,
    #[error("Physical decimal mismatch: expected={expected}, actual={actual}")]
    DecimalMismatch { expected: u8, actual: u8 },
    #[error("oberer-orm execution error: {detail}")]
    ObererOrmError { detail: String },
    #[error("oberer-orm CAS optimistic lock conflict (`#[orm(version)]`) on table={table}, pk={pk}")]
    ObererOptimisticLockConflict { table: &'static str, pk: String },
    #[error("oberer-orm safety violation: unfiltered UPDATE/DELETE blocked (`NoFilterOnMutation`)")]
    ObererUnfilteredMutationBlocked,
    #[error("CRITICAL SECURITY ALERT: SQLite row MAC or hash-chain tampering detected in table={table}, pk={primary_key}")]
    SqliteRowTamperingDetected {
        table: &'static str,
        primary_key: String,
    },
    #[error("IMPRCE cache integrity violation: CRC32C/MAC mismatch on key={key}, self_healed={self_healed}")]
    CacheIntegrityCorrupted { key: String, self_healed: bool },
    #[error("IMPRCE priority quota ceiling exhausted for non-evictable tier {tier:?}")]
    CacheQuotaExhausted { tier: CachePriorityTier },
    #[error("Replay attack blocked by oberer-orm IdempotencyService / IMPRCE: digest '{digest}'")]
    ReplayAttackBlocked { digest: String },
    #[error("Dual-oracle divergence {divergence_bps} bps exceeds circuit breaker {max_bps} bps")]
    OracleDivergenceCircuitBreaker { divergence_bps: u32, max_bps: u32 },
    #[error("Blockchain reorganization detected at height {height}: rolled back to LCA {lca_height}")]
    ConsensusReorgDetected { height: u64, lca_height: u64 },
    #[error("5-Gate verification failed at Gate {gate}: {reason}")]
    GateVerificationFailed { gate: u8, reason: String },
    #[error("Zone C Enclave policy violation: {reason}")]
    EnclavePolicyViolation { reason: String },
    #[error("Insufficient M-of-N hardware quorum: required={required}, verified={verified}")]
    QuorumInsufficient { required: usize, verified: usize },
    #[error("Native node transport failure on {chain:?}: {detail}")]
    NodeTransportFailure { chain: ChainId, detail: String },
    #[error("Rotating deposit address pool exhausted on {chain:?}")]
    AddressPoolExhausted { chain: ChainId },
    #[error("Dependency Injection (DI) configuration error: missing or invalid dependency '{dependency}'")]
    DependencyInjectionError { dependency: &'static str },
    #[error("Security Zone isolation violation: engine '{engine_id}' in {actual:?} cannot be injected into {expected:?}")]
    SecurityZoneMismatch {
        engine_id: &'static str,
        expected: SecurityZone,
        actual: SecurityZone,
    },
}

impl From<OrmError> for SovereignError {
    fn from(err: OrmError) -> Self {
        match err {
            OrmError::OptimisticLockConflict => Self::ObererOptimisticLockConflict {
                table: "unknown",
                pk: "cas_conflict".to_string(),
            },
            OrmError::NoFilterOnMutation => Self::ObererUnfilteredMutationBlocked,
            other => Self::ObererOrmError {
                detail: other.to_string(),
            },
        }
    }
}

impl From<OrmFailure> for SovereignError {
    fn from(failure: OrmFailure) -> Self {
        Self::ObererOrmError {
            detail: format!("{failure:?}"),
        }
    }
}

/// Canonical logical assets across all supported networks for unified portfolio aggregation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LogicalAsset {
    Btc,
    Eth,
    Bnb,
    Plume,
    Sol,
    Trx,
    Ton,
    Sui,
    Usdt,
    Usdc,
}

impl LogicalAsset {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Btc => "BTC",
            Self::Eth => "ETH",
            Self::Bnb => "BNB",
            Self::Plume => "PLUME",
            Self::Sol => "SOL",
            Self::Trx => "TRX",
            Self::Ton => "TON",
            Self::Sui => "SUI",
            Self::Usdt => "USDT",
            Self::Usdc => "USDC",
        }
    }
}

/// Blockchain execution model taxonomy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ChainArchitecture {
    BitcoinUtxo,
    EvmAccount,
    SvmAccount,
    TvmAccount,
    TonActor,
    MoveObject,
}

/// Canonical chain identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ChainId {
    BitcoinMainnet = 0,
    EthereumMainnet = 1,
    BnbChain = 56,
    PlumeMainnet = 98866,
    SolanaMainnet = 900,
    TronMainnet = 728126428,
    TonMainnet = 1100,
    SuiMainnet = 1200,
}

impl ChainId {
    pub const fn architecture(self) -> ChainArchitecture {
        match self {
            Self::BitcoinMainnet => ChainArchitecture::BitcoinUtxo,
            Self::EthereumMainnet | Self::BnbChain | Self::PlumeMainnet => {
                ChainArchitecture::EvmAccount
            }
            Self::SolanaMainnet => ChainArchitecture::SvmAccount,
            Self::TronMainnet => ChainArchitecture::TvmAccount,
            Self::TonMainnet => ChainArchitecture::TonActor,
            Self::SuiMainnet => ChainArchitecture::MoveObject,
        }
    }

    pub const fn as_i64(self) -> i64 {
        self as i64
    }
}

/// Priority tiers for the In-Memory Priority-Redundant Cache Engine (IMPRCE).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum CachePriorityTier {
    /// P0: Non-evictable consensus headers & signed token contract registry (k = 3 replicas).
    P0CriticalConsensus = 0,
    /// P1: Non-evictable active payment watchlists during quote + grace window (k = 3 replicas).
    P1ActivePaymentWatch = 1,
    /// P2: Rotating deposit address leases & 4-hour post-expiry cooldowns (k = 2 replicas).
    P2PoolLeaseCooldown = 2,
    /// P3: 15-minute locked checkout quotes & 10-second dual-oracle snapshots (k = 2 replicas).
    P3OracleQuoteLock = 3,
    /// P4: Webhook idempotency keys & ephemeral RPC telemetry cache (k = 1 replica, first evicted).
    P4TelemetryIdempotency = 4,
}

impl CachePriorityTier {
    pub const fn replication_factor(self) -> usize {
        match self {
            Self::P0CriticalConsensus | Self::P1ActivePaymentWatch => 3,
            Self::P2PoolLeaseCooldown | Self::P3OracleQuoteLock => 2,
            Self::P4TelemetryIdempotency => 1,
        }
    }
}

/// Identifies a specific physical asset contract/mint on a specific chain.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PhysicalAssetId {
    pub chain_id: ChainId,
    pub logical_asset: LogicalAsset,
    pub token_contract: String,
    pub decimals: u8,
    pub keep_one_unit_dust: bool,
    pub target_confirmations: u32,
}

/// Custody security tier taxonomy for treasury wallets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum WalletTier {
    Tier0CustomerDeposit,
    Tier1GasRelayer,
    Tier2OperationalHot,
    Tier3ColdVault,
    Tier4MarketExchange,
}

/// Bijective Order-Preserving 256-Bit Integer Codec for `oberer-orm` `Vec<u8>` (`BLOB`) and `String` (`TEXT`).
/// Guarantees SQLite `memcmp(a, b)` in B-Trees is isomorphic to `a < b` in \mathbb{Z}_{2^{256}}.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObererU256Codec;

impl ObererU256Codec {
    /// Encodes U256 into a 32-byte Big-Endian `Vec<u8>` for `oberer-orm` `BLOB` fields.
    #[inline]
    pub fn to_be_blob_vec(value: U256) -> Vec<u8> {
        value.to_be_bytes::<32>().to_vec()
    }

    /// Decodes a 32-byte Big-Endian `oberer-orm` `BLOB` slice back into U256.
    #[inline]
    pub fn from_be_blob_slice(bytes: &[u8]) -> Result<U256, SovereignError> {
        let arr: [u8; 32] = bytes.try_into().map_err(|_| SovereignError::ObererOrmError {
            detail: format!("Invalid U256 BLOB(32) byte length: {}", bytes.len()),
        })?;
        Ok(U256::from_be_bytes(arr))
    }

    /// Encodes U256 into a fixed 78-character zero-padded base-10 `String` for `oberer-orm` `TEXT` fields.
    #[inline]
    pub fn to_padded_dec78(value: U256) -> String {
        format!("{value:0>78}")
    }

    /// Parses a 78-character zero-padded decimal string back into U256.
    #[inline]
    pub fn from_padded_dec78(s: &str) -> Result<U256, SovereignError> {
        let trimmed = s.trim_start_matches('0');
        if trimmed.is_empty() {
            return Ok(U256::ZERO);
        }
        U256::from_str_radix(trimmed, 10).map_err(|e| SovereignError::ObererOrmError {
            detail: format!("Invalid U256 TEXT(78) decimal string '{s}': {e}"),
        })
    }
}

/// Checked fixed-scale raw on-chain integer amount.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RawAssetAmount {
    raw: U256,
    decimals: u8,
}

impl RawAssetAmount {
    pub const fn new(raw: U256, decimals: u8) -> Self {
        Self { raw, decimals }
    }

    pub const fn zero(decimals: u8) -> Self {
        Self {
            raw: U256::ZERO,
            decimals,
        }
    }

    pub const fn raw(&self) -> U256 {
        self.raw
    }

    pub const fn decimals(&self) -> u8 {
        self.decimals
    }

    pub fn is_zero(&self) -> bool {
        self.raw.is_zero()
    }

    pub fn to_oberer_blob_vec(self) -> Vec<u8> {
        ObererU256Codec::to_be_blob_vec(self.raw)
    }

    pub fn to_oberer_dec78(self) -> String {
        ObererU256Codec::to_padded_dec78(self.raw)
    }

    pub fn checked_add(self, rhs: Self) -> Result<Self, SovereignError> {
        if self.decimals != rhs.decimals {
            return Err(SovereignError::DecimalMismatch {
                expected: self.decimals,
                actual: rhs.decimals,
            });
        }
        let sum = self
            .raw
            .checked_add(rhs.raw)
            .ok_or(SovereignError::Overflow {
                context: "RawAssetAmount::checked_add",
            })?;
        Ok(Self::new(sum, self.decimals))
    }

    pub fn checked_sub(self, rhs: Self) -> Result<Self, SovereignError> {
        if self.decimals != rhs.decimals {
            return Err(SovereignError::DecimalMismatch {
                expected: self.decimals,
                actual: rhs.decimals,
            });
        }
        let diff = self
            .raw
            .checked_sub(rhs.raw)
            .ok_or(SovereignError::Underflow {
                context: "RawAssetAmount::checked_sub",
            })?;
        Ok(Self::new(diff, self.decimals))
    }

    /// Maps any physical decimal scale d \in [0, 36] into canonical 18-decimal units \mathcal{N}_{18}(r, d).
    pub fn to_canonical_18(self) -> Result<U256, SovereignError> {
        if self.decimals == 18 {
            return Ok(self.raw);
        }
        if self.decimals < 18 {
            let exp = 18_u8
                .checked_sub(self.decimals)
                .ok_or(SovereignError::Underflow {
                    context: "to_canonical_18::exp_lt",
                })?;
            let factor = U256::from(10_u64)
                .checked_pow(U256::from(exp))
                .ok_or(SovereignError::Overflow {
                    context: "to_canonical_18::pow_lt",
                })?;
            self.raw
                .checked_mul(factor)
                .ok_or(SovereignError::Overflow {
                    context: "to_canonical_18::mul",
                })
        } else {
            let exp = self
                .decimals
                .checked_sub(18_u8)
                .ok_or(SovereignError::Underflow {
                    context: "to_canonical_18::exp_gt",
                })?;
            let divisor = U256::from(10_u64)
                .checked_pow(U256::from(exp))
                .ok_or(SovereignError::Overflow {
                    context: "to_canonical_18::pow_gt",
                })?;
            self.raw
                .checked_div(divisor)
                .ok_or(SovereignError::DivisionByZero)
        }
    }
}

/// Bridge helper connecting `oberer_orm::MicroCredits` ($1.00 = 1_000_000 microcredits)
/// with 256-bit RawAssetAmount calculations.
pub struct MicroCreditsMathBridge;

impl MicroCreditsMathBridge {
    /// Evaluates \delta_{bps}(P_cex, P_chain) using `oberer_orm::MicroCredits` and enforces the circuit breaker.
    pub fn verify_dual_oracle_microcredits(
        cex_mid_mc: MicroCredits,
        onchain_oracle_mc: MicroCredits,
        max_divergence_bps: u32,
    ) -> Result<MicroCredits, SovereignError> {
        let cex_val = cex_mid_mc.get();
        let chain_val = onchain_oracle_mc.get();
        if cex_val <= 0 || chain_val <= 0 {
            return Err(SovereignError::DivisionByZero);
        }
        let min_p = cex_val.min(chain_val) as u128;
        let abs_diff = (cex_val as i128).abs_diff(chain_val as i128);
        let divergence_u128 = abs_diff
            .checked_mul(10_000)
            .ok_or(SovereignError::Overflow {
                context: "verify_dual_oracle_microcredits::mul",
            })?
            .checked_div(min_p)
            .ok_or(SovereignError::DivisionByZero)?;

        let divergence_bps = u32::try_from(divergence_u128).unwrap_or(u32::MAX);
        if divergence_bps > max_divergence_bps {
            return Err(SovereignError::OracleDivergenceCircuitBreaker {
                divergence_bps,
                max_bps: max_divergence_bps,
            });
        }
        Ok(MicroCredits::new(cex_val.min(chain_val)))
    }

    /// Converts an invoice fiat `MicroCredits` amount into clean locked raw token units \mathcal{T}_{clean}.
    pub fn to_clean_locked_quote(
        fiat_mc: MicroCredits,
        locked_price_mc: MicroCredits,
        decimals: u8,
    ) -> Result<RawAssetAmount, SovereignError> {
        if fiat_mc.get() < 0 || locked_price_mc.get() <= 0 {
            return Err(SovereignError::DivisionByZero);
        }
        let scale = U256::from(10_u64)
            .checked_pow(U256::from(decimals))
            .ok_or(SovereignError::Overflow {
                context: "to_clean_locked_quote::pow",
            })?;
        let mut raw = U256::from(fiat_mc.get() as u64)
            .checked_mul(scale)
            .ok_or(SovereignError::Overflow {
                context: "to_clean_locked_quote::mul",
            })?
            .checked_div(U256::from(locked_price_mc.get() as u64))
            .ok_or(SovereignError::DivisionByZero)?;

        if decimals == 18 {
            let mask = U256::from(1_000_000_000_000_u64);
            raw = raw
                .checked_div(mask)
                .ok_or(SovereignError::DivisionByZero)?
                .checked_mul(mask)
                .ok_or(SovereignError::Overflow {
                    context: "to_clean_locked_quote::mask",
                })?;
        }
        Ok(RawAssetAmount::new(raw, decimals))
    }

    /// Evaluates the USD `MicroCredits` valuation of a `RawAssetAmount` at `price_mc`.
    pub fn to_microcredits_valuation(
        amount: RawAssetAmount,
        price_mc: MicroCredits,
    ) -> Result<MicroCredits, SovereignError> {
        if price_mc.get() < 0 {
            return Err(SovereignError::Underflow {
                context: "to_microcredits_valuation::negative_price",
            });
        }
        let scale = U256::from(10_u64)
            .checked_pow(U256::from(amount.decimals()))
            .ok_or(SovereignError::Overflow {
                context: "to_microcredits_valuation::pow",
            })?;
        let val_u256 = amount
            .raw()
            .checked_mul(U256::from(price_mc.get() as u64))
            .ok_or(SovereignError::Overflow {
                context: "to_microcredits_valuation::mul",
            })?
            .checked_div(scale)
            .ok_or(SovereignError::DivisionByZero)?;

        let val_i64 = i64::try_from(val_u256).map_err(|_| SovereignError::Overflow {
            context: "to_microcredits_valuation::i64_cast",
        })?;
        Ok(MicroCredits::new(val_i64))
    }
}

```

### 5.2 Root OOP Lifecycle Trait (`SovereignEngine`) & `ObererOrmStorageEngine`

This module defines the root **`SovereignEngine`** trait inherited by every engine in the platform, followed by **`ObererOrmStorageEngine`**, which wraps `oberer_orm::conn::HybridConn`, `OrmConn`, `Table<T>`, `TableCached<T>`, `TableView<T>`, `TableTrigger<T>`, `IdempotencyService`, and `AuditRecordBuilder` into our formal OOP Engine hierarchy.

```
// crates/gw-enterprise-traits/src/oberer_storage_engine.rs
#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic, clippy::unwrap_used)]

use async_trait::async_trait;
use gw_primitives::{SecurityZone, SovereignError};
use oberer_orm::conn::{HybridConn, OrmConn};
use oberer_orm::prelude::*;
use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Cryptographic self-audit report returned by any `SovereignEngine`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EngineIntegrityReport {
    pub engine_id: String,
    pub security_zone: SecurityZone,
    pub verified_records_or_cells: u64,
    pub detected_corruptions_or_tampers: u64,
    pub autonomous_self_heals: u64,
    pub hash_chain_head_hex: String,
    pub audit_passed: bool,
}

/// Standardized real-time operational health snapshot for any `SovereignEngine`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EngineHealthSnapshot {
    pub engine_id: String,
    pub security_zone: SecurityZone,
    pub is_healthy: bool,
    pub active_workers_or_shards: usize,
    pub total_operations_processed: u64,
    pub p99_operation_latency_us: u64,
    pub memory_allocated_bytes: usize,
}

/// Root Object-Oriented Lifecycle & Security Trait implemented by every subsystem engine.
#[async_trait]
pub trait SovereignEngine: Send + Sync {
    /// Unique static identifier of the engine (e.g., `"OBERER_ORM_HYBRID_STORAGE_ENGINE"`).
    fn engine_id(&self) -> &'static str;

    /// Physical/logical security zone in which this engine is permitted to execute.
    fn security_zone(&self) -> SecurityZone;

    /// Initializes internal state, runs `MigrationPlanBuilder` / `Schema::<T>::new(conn)`, and warms caches.
    async fn initialize_engine(&self) -> Result<(), SovereignError>;

    /// Executes a deep cryptographic integrity audit (verifying CRC32C, Row MACs, or Hash Chains).
    async fn audit_cryptographic_integrity(&self) -> Result<EngineIntegrityReport, SovereignError>;

    /// Returns a standardized operational telemetry and health snapshot.
    async fn health_snapshot(&self) -> Result<EngineHealthSnapshot, SovereignError>;

    /// Flushes `TableCached<T>`, invokes `HybridConn::sync()`, and zeroes ephemeral keys.
    async fn shutdown_gracefully(&self) -> Result<(), SovereignError>;
}

/// Per-boot / HSM-unsealed 256-bit Row-Authentication Key for `oberer-orm` entities.
/// Protects SQLite `.db` and `.db-wal` files against offline tampering (Axiom 8).
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct ObererRowMacSigner {
    row_mac_key: [u8; 32],
}

impl ObererRowMacSigner {
    pub const fn new(row_mac_key: [u8; 32]) -> Self {
        Self { row_mac_key }
    }

    /// Computes a 32-byte keyed Blake3 MAC binding `(table || pk || version || canonical_bytes)`.
    pub fn compute_row_mac(
        &self,
        table_name: &'static str,
        primary_key: &str,
        version: i64,
        canonical_payload: &[u8],
    ) -> Vec<u8> {
        let mut hasher = blake3::Hasher::new_keyed(&self.row_mac_key);
        hasher.update(b"OBERER_ORM_V9_ROW_MAC");
        hasher.update(table_name.as_bytes());
        hasher.update(&(primary_key.len() as u64).to_le_bytes());
        hasher.update(primary_key.as_bytes());
        hasher.update(&version.to_le_bytes());
        hasher.update(&(canonical_payload.len() as u64).to_le_bytes());
        hasher.update(canonical_payload);
        hasher.finalize().as_bytes().to_vec()
    }

    /// Verifies the stored `row_mac: Vec<u8>` column in constant time (`subtle::ConstantTimeEq`).
    pub fn verify_row_mac(
        &self,
        table_name: &'static str,
        primary_key: &str,
        version: i64,
        canonical_payload: &[u8],
        stored_mac: &[u8],
    ) -> Result<(), SovereignError> {
        let expected =
            self.compute_row_mac(table_name, primary_key, version, canonical_payload);
        if expected.len() == stored_mac.len() && bool::from(expected.ct_eq(stored_mac)) {
            Ok(())
        } else {
            Err(SovereignError::SqliteRowTamperingDetected {
                table: table_name,
                primary_key: primary_key.to_string(),
            })
        }
    }
}

/// Trait implemented by security-critical `#[derive(OrmTable)]` models to bind `#[orm(version)]`
/// and `row_mac: Vec<u8>` verification into `ObererOrmStorageEngine`.
pub trait TamperEvidentOrmModel: OrmTable + Clone + Send + Sync + 'static {
    const TABLE_LABEL: &'static str;
    fn pk_string(&self) -> String;
    fn current_version(&self) -> i64;
    fn canonical_security_bytes(&self) -> Vec<u8>;
    fn row_mac_slice(&self) -> &[u8];
    fn assign_row_mac(&mut self, mac: Vec<u8>);
}

/// Master `oberer-orm` Dual-Tier Storage Engine Trait (`HybridConn` + V001/V002 Platform Services).
/// Note: Object-safe methods allow `Arc<dyn ObererOrmStorageEngine>` injection across all services!
#[async_trait]
pub trait ObererOrmStorageEngine: SovereignEngine {
    /// Returns a reference to the dual-tier `HybridConn` (`:memory:` working DB + disk WAL DB).
    fn hybrid_conn(&self) -> &HybridConn;

    /// Returns a cloned handle to the fast `:memory:` `OrmConn` thread-queue actor.
    fn memory_conn(&self) -> OrmConn {
        self.hybrid_conn().mem().clone()
    }

    /// Returns a cloned handle to the persistent disk WAL `OrmConn` thread-queue actor.
    fn disk_conn(&self) -> OrmConn {
        self.hybrid_conn().disk().clone()
    }

    /// Returns the cryptographic Row-MAC signer used to seal and verify `#[derive(OrmTable)]` rows.
    fn row_mac_signer(&self) -> &ObererRowMacSigner;

    /// Verifies and records an atomic idempotency barrier via `oberer_orm::runtime::IdempotencyService`
    /// (`RequestIdempotencyKey` V001 model).
    async fn acquire_idempotency_barrier(
        &self,
        scope: &str,
        request_key: &str,
        payload_sha256_hex: &str,
    ) -> Result<(), SovereignError>;

    /// Appends an immutable audit trail entry via `oberer_orm::runtime::AuditRecordBuilder` (`AuditLog`).
    async fn record_platform_audit_log(
        &self,
        action: &str,
        actor_id: &str,
        detail_json: &str,
    ) -> Result<(), SovereignError>;

    /// Triggers an explicit row-level delta flush (`DirtyTracker`) from `:memory:` L1 to Disk WAL L2.
    async fn sync_hybrid_to_disk(&self) -> Result<(), SovereignError>;
}

/// Extension trait providing generic closure-based ACID transactions on any `ObererOrmStorageEngine`
/// while preserving full `dyn ObererOrmStorageEngine` object-safety for Dependency Injection!
pub trait ObererOrmStorageEngineExt: ObererOrmStorageEngine {
    /// Executes an atomic transaction on the `:memory:` working tier via `conn.transaction(|tx| ...)`
    /// and immediately synchronizes dirty deltas to the disk WAL database via `hybrid.sync()`.
    fn execute_atomic_synced_tx<F, R>(&self, f: F) -> Result<R, SovereignError>
    where
        F: FnOnce(&OrmConn) -> Result<R, OrmError>,
    {
        let res = self.hybrid_conn().mem().transaction(f)?;
        self.hybrid_conn().sync()?;
        Ok(res)
    }
}

impl<T: ?Sized + ObererOrmStorageEngine> ObererOrmStorageEngineExt for T {}

```

### 5.3 Engine Trait I: `PriorityRedundantCacheEngine` (`IMPRCE` $\leftrightarrow$ `HybridConn` / `TableCached`)

```
// crates/gw-enterprise-traits/src/cache_engine_trait.rs
#![forbid(unsafe_code)]

use crate::oberer_storage_engine::{ObererOrmStorageEngine, SovereignEngine};
use async_trait::async_trait;
use gw_primitives::{CachePriorityTier, ChainId, SovereignError};
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashSet;

/// Tamper-evident raw in-memory cell returned by a quorum-verified read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedRawCacheCell {
    pub namespace: String,
    pub key: String,
    pub payload_bytes: Vec<u8>,
    pub priority_tier: CachePriorityTier,
    pub version: u64,
    pub created_offset_ms: u64,
    pub expires_offset_ms: Option<u64>,
    pub crc32c_tag: u32,
    pub mac_tag_128: [u8; 16],
    pub surviving_replicas: u8,
}

/// Typed wrapper decoded from a `VerifiedRawCacheCell`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedCacheCell<T> {
    pub namespace: String,
    pub key: String,
    pub value: T,
    pub priority_tier: CachePriorityTier,
    pub version: u64,
    pub created_offset_ms: u64,
    pub expires_offset_ms: Option<u64>,
    pub crc32c_tag: u32,
    pub mac_tag_128: [u8; 16],
    pub surviving_replicas: u8,
}

/// Engine Trait I: Object-Safe Sovereign In-Memory Priority-Redundant Cache Engine (`IMPRCE`)
/// paired with `oberer-orm` `HybridConn` and `TableCached<T>`.
#[async_trait]
pub trait PriorityRedundantCacheEngine: SovereignEngine {
    /// Hydrates P0 (Signed Asset Registry & 256-Block Ring Buffer), P1 (Active Watchlists via
    /// `TableView<ActiveWatchlistView>`), and P2 (Rotating Pool Leases) in under 10ms by invoking
    /// `hybrid.warm_table::<T>()` and verifying every row's `row_mac`.
    async fn hydrate_from_oberer_hybrid(
        &self,
        storage_engine: &dyn ObererOrmStorageEngine,
    ) -> Result<usize, SovereignError>;

    /// Writes a raw byte slice across `k(priority_tier)` disjoint SipHash/Blake3-sharded memory
    /// mirror arenas with dual `CRC32C` + `128-bit MAC` tags.
    async fn put_redundant_raw(
        &self,
        namespace: &'static str,
        key: &str,
        payload_bytes: &[u8],
        priority_tier: CachePriorityTier,
        ttl_ms: Option<u64>,
    ) -> Result<u64, SovereignError>;

    /// Performs an O(1) read from Primary Arena 0, validating both `CRC32C` and `128-bit MAC`
    /// in constant time. On bit-rot or lock contention, immediately serves from Mirror Arena 1 or 2
    /// and spawns an atomic self-healing repair write on Arena 0.
    async fn get_quorum_verified_raw(
        &self,
        namespace: &'static str,
        key: &str,
    ) -> Result<Option<VerifiedRawCacheCell>, SovereignError>;

    /// Atomic Compare-And-Swap (CAS) across all k mirror arenas.
    async fn compare_and_swap_redundant_raw(
        &self,
        namespace: &'static str,
        key: &str,
        expected_version: u64,
        new_payload_bytes: &[u8],
        priority_tier: CachePriorityTier,
        new_ttl_ms: Option<u64>,
    ) -> Result<bool, SovereignError>;

    /// Atomically registers an active checkout lookup key into the P1 triple-redundant watchlist
    /// and sets its bits in the L1 cache-resident 256-bit Blocked Bloom Filter.
    async fn register_active_watchlist_key(
        &self,
        chain_id: ChainId,
        lookup_key: &str,
        invoice_id: i64,
        ttl_ms: u64,
    ) -> Result<(), SovereignError>;

    /// Wait-free O(1) batch filter for block indexers: rejects 99.9% of non-watched blockchain
    /// outputs via the L1 Blocked Bloom Filter in ~4.2 ns, then verifies exact hits in P1 Arena.
    async fn filter_watched_keys_batch(
        &self,
        chain_id: ChainId,
        candidate_keys: &[String],
    ) -> Result<HashSet<String>, SovereignError>;

    /// Atomically removes a settled or expired key and overwrites its heap bytes with `0x00` (`Zeroize`).
    async fn remove_and_zeroize(
        &self,
        namespace: &'static str,
        key: &str,
    ) -> Result<bool, SovereignError>;

    /// Enforces hard per-tier byte quota ceilings by evicting entries in ascending order of
    /// Retention Priority Score \Phi(e, t) strictly from P4 and expired P3/P2.
    async fn enforce_quota_fence_eviction(&self) -> Result<usize, SovereignError>;
}

/// Ergonomic typed extension trait for `PriorityRedundantCacheEngine` (preserves `dyn` object safety).
#[async_trait]
pub trait PriorityRedundantCacheEngineExt: PriorityRedundantCacheEngine {
    async fn put_redundant<T>(
        &self,
        namespace: &'static str,
        key: &str,
        value: &T,
        priority_tier: CachePriorityTier,
        ttl_ms: Option<u64>,
    ) -> Result<u64, SovereignError>
    where
        T: Serialize + Send + Sync,
    {
        let bytes = serde_json::to_vec(value).map_err(|e| SovereignError::ObererOrmError {
            detail: format!("Cache serialization error: {e}"),
        })?;
        self.put_redundant_raw(namespace, key, &bytes, priority_tier, ttl_ms)
            .await
    }

    async fn get_quorum_verified<T>(
        &self,
        namespace: &'static str,
        key: &str,
    ) -> Result<Option<VerifiedCacheCell<T>>, SovereignError>
    where
        T: DeserializeOwned + Send + Sync,
    {
        let Some(raw) = self.get_quorum_verified_raw(namespace, key).await? else {
            return Ok(None);
        };
        let value: T =
            serde_json::from_slice(&raw.payload_bytes).map_err(|e| SovereignError::ObererOrmError {
                detail: format!("Cache deserialization error: {e}"),
            })?;
        Ok(Some(VerifiedCacheCell {
            namespace: raw.namespace,
            key: raw.key,
            value,
            priority_tier: raw.priority_tier,
            version: raw.version,
            created_offset_ms: raw.created_offset_ms,
            expires_offset_ms: raw.expires_offset_ms,
            crc32c_tag: raw.crc32c_tag,
            mac_tag_128: raw.mac_tag_128,
            surviving_replicas: raw.surviving_replicas,
        }))
    }
}

impl<E: ?Sized + PriorityRedundantCacheEngine> PriorityRedundantCacheEngineExt for E {}

```

### 5.4 Engine Trait II: `WatchOnlyDerivationEngine` (Deterministic Route & Pool Leasing)

```
// crates/gw-enterprise-traits/src/derivation_engine_trait.rs
#![forbid(unsafe_code)]

use crate::oberer_storage_engine::SovereignEngine;
use async_trait::async_trait;
use gw_primitives::{ChainId, PhysicalAssetId, SovereignError};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Complete deterministic deposit routing descriptor returned to the checkout engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckoutDepositRoute {
    pub chain_id: ChainId,
    /// Primary receiving address (bc1q... on BTC, CREATE2 0x... on EVM, T... on Tron, Vault on TON/Solana).
    pub deposit_address: String,
    /// Optional secondary routing identifier (Solana Reference Pubkey or TON 8-Byte Crockford MemoTag).
    pub routing_reference: Option<String>,
    /// Exact deterministic derivation path (m/84'/0'/0'/0/i) or 32-byte CREATE2/PDA salt hex.
    pub derivation_locator: String,
    /// Cryptographic commitment attesting that the address is bound to Cold Storage.
    pub counterfactual_commitment_hash: String,
}

impl CheckoutDepositRoute {
    pub fn canonical_watchlist_key(&self) -> &str {
        self.routing_reference
            .as_deref()
            .unwrap_or(&self.deposit_address)
    }
}

/// Engine Trait II: Deterministic Watch-Only Address & Counterfactual Route Derivation Engine.
#[async_trait]
pub trait WatchOnlyDerivationEngine: SovereignEngine {
    fn chain_id(&self) -> ChainId;

    /// Derives a single deterministic checkout route in pure watch-only mode (Zero Private Keys).
    /// For guest checkouts on EVM/Tron/Solana PDA, leases an `Available` route atomically via
    /// `Table::<DepositAddressPoolRow>::update_with_version` in `oberer-orm`.
    async fn derive_checkout_route(
        &self,
        invoice_id: i64,
        customer_id: Option<Uuid>,
        asset: &PhysicalAssetId,
        prefer_zero_sweep_reference: bool,
    ) -> Result<CheckoutDepositRoute, SovereignError>;

    /// Pre-generates a contiguous batch of deterministic routes (`Erlang-B` bound M^*) and persists
    /// them into `Table::<DepositAddressPoolRow>` in `oberer-orm`.
    async fn warm_address_pool_batch(
        &self,
        start_pool_index: u64,
        count: usize,
        asset: &PhysicalAssetId,
    ) -> Result<Vec<CheckoutDepositRoute>, SovereignError>;

    /// Cryptographically verifies that a candidate deposit_address and routing_reference match
    /// the deterministic derivation locator and immutable Cold Treasury bytecode commitment.
    fn verify_route_determinism(
        &self,
        route: &CheckoutDepositRoute,
        expected_cold_vault_address: &str,
    ) -> Result<bool, SovereignError>;

    /// Validates external user-supplied or exchange-whitelisted destination address syntax & checksum.
    fn validate_address_syntax_and_checksum(
        &self,
        raw_address: &str,
    ) -> Result<String, SovereignError>;
}

```

### 5.5 Engine Trait III: `ConsensusIndexingEngine` (DAG Continuity, Bloom & O(1) Match)

```
// crates/gw-enterprise-traits/src/indexing_engine_trait.rs
#![forbid(unsafe_code)]

use crate::oberer_storage_engine::SovereignEngine;
use async_trait::async_trait;
use gw_primitives::{ChainArchitecture, ChainId, RawAssetAmount, SovereignError};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Canonical block header stored in P0 IMPRCE and `Table::<ConsensusBlockHeaderRow>` in `oberer-orm`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsensusBlockHeader {
    pub chain_id: ChainId,
    pub block_number: i64,
    pub block_hash: String,
    pub parent_block_hash: String,
    pub state_root_or_merkle_root: String,
    pub timestamp_unix: i64,
    pub is_finalized: bool,
}

/// Normalized incoming value transfer extracted from any blockchain's native block/slot/checkpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedTransferEvent {
    pub chain_id: ChainId,
    pub architecture: ChainArchitecture,
    pub tx_hash: String,
    pub event_index: u32,
    pub block_header: ConsensusBlockHeader,
    pub watchlist_lookup_key: String,
    pub sender_address: String,
    pub recipient_address: String,
    pub token_contract: String,
    /// Strict balance differential: post_balance.checked_sub(pre_balance) on recipient vault.
    pub net_credited_amount: RawAssetAmount,
    /// Gate 1 indicator: true iff receipt.status == 1 / meta.err == None / !aborted.
    pub execution_succeeded: bool,
    pub confirmations: u32,
}

/// Result of verifying parent-hash continuity \Lambda_{cont}(b_h, \mathcal{B}_C).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContinuityCheckOutcome {
    CanonicalAppend,
    ReorgDetected {
        orphaned_from_height: i64,
        orphaned_to_height: i64,
        lca_height: i64,
        lca_block_hash: String,
    },
}

/// Engine Trait III: Native Node Block Ingestion, DAG Continuity & O(1) Watchlist Matching Engine.
#[async_trait]
pub trait ConsensusIndexingEngine: SovereignEngine {
    fn chain_id(&self) -> ChainId;

    /// Queries the latest head height and finalized height from the local native node quorum.
    async fn query_consensus_heights(&self) -> Result<(i64, i64), SovereignError>;

    /// Fetches the canonical block header at `block_height` and verifies `parent_block_hash`
    /// continuity against the P0 ring buffer in `IMPRCE` and `oberer-orm`.
    async fn fetch_and_verify_block_continuity(
        &self,
        block_height: i64,
        previous_cached_header: Option<&ConsensusBlockHeader>,
    ) -> Result<(ConsensusBlockHeader, ContinuityCheckOutcome), SovereignError>;

    /// Walks backward to find the Lowest Common Ancestor (LCA) height h_{LCA} upon fork detection.
    async fn find_reorg_lowest_common_ancestor(
        &self,
        divergent_height: i64,
        cached_window: &[ConsensusBlockHeader],
    ) -> Result<i64, SovereignError>;

    /// Scans a single block/slot/checkpoint and extracts all transfer events matching
    /// `active_watchlist_keys` in O(1) time per transaction.
    async fn scan_block_for_watched_transfers(
        &self,
        block_header: &ConsensusBlockHeader,
        current_tip_height: i64,
        active_watchlist_keys: &HashSet<String>,
    ) -> Result<Vec<NormalizedTransferEvent>, SovereignError>;

    /// High-throughput range catch-up scanner using temporary staging table `#[orm(temporary)]`
    /// in `oberer-orm` for zero-disk-footprint batch filtering.
    async fn scan_block_range_batch(
        &self,
        start_height: i64,
        end_height: i64,
        active_watchlist_keys: &HashSet<String>,
    ) -> Result<Vec<NormalizedTransferEvent>, SovereignError>;
}

```

### 5.6 Engine Trait IV: `StateBalanceReaderEngine` (Proof-Verified Balance & Resource Quotas)

```
// crates/gw-enterprise-traits/src/balance_engine_trait.rs
#![forbid(unsafe_code)]

use crate::oberer_storage_engine::SovereignEngine;
use async_trait::async_trait;
use gw_primitives::{ChainId, PhysicalAssetId, RawAssetAmount, SovereignError};
use serde::{Deserialize, Serialize};

/// Chain-specific execution resource and gas relayer health telemetry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelayerResourceHealth {
    pub chain_id: ChainId,
    pub relayer_address: String,
    pub native_gas_balance: RawAssetAmount,
    pub min_required_gas_balance: RawAssetAmount,
    /// For Tron (TVM): Available delegated Stake 2.0 Energy & Bandwidth units.
    pub available_tvm_energy: u64,
    pub available_tvm_bandwidth: u64,
    /// For Solana (SVM): Count of open temporary PDA Token Accounts holding reclaimable rent.
    pub reclaimable_svm_rent_lamports: u64,
    pub next_pending_nonce: u64,
    pub needs_immediate_refill: bool,
}

/// Unspent Transaction Output (UTXO) or discrete Move Coin<T> object descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpendableDiscreteOutput {
    pub output_id: String,
    pub owner_address: String,
    pub derivation_locator: String,
    pub amount: RawAssetAmount,
    pub confirmations: u32,
}

/// Engine Trait IV: On-Chain State Balance, UTXO/Object Set & Relayer Resource Reader Engine.
#[async_trait]
pub trait StateBalanceReaderEngine: SovereignEngine {
    fn chain_id(&self) -> ChainId;

    /// Queries the exact raw balance of `(address, asset)` at the latest finalized block.
    async fn get_finalized_balance(
        &self,
        address: &str,
        asset: &PhysicalAssetId,
    ) -> Result<RawAssetAmount, SovereignError>;

    /// Multicall / pipelined batch balance query across N wallet addresses and assets.
    async fn get_balances_batch(
        &self,
        queries: &[(String, PhysicalAssetId)],
    ) -> Result<Vec<RawAssetAmount>, SovereignError>;

    /// Lists all spendable UTXOs (Bitcoin) or discrete Coin<T> objects (Sui) for batch consolidation.
    async fn list_spendable_discrete_outputs(
        &self,
        addresses_with_locators: &[(String, String)],
        asset: &PhysicalAssetId,
        min_confirmations: u32,
    ) -> Result<Vec<SpendableDiscreteOutput>, SovereignError>;

    /// Inspects the Tier 1 Gas Station Relayer's native gas, nonce continuity, Tron Stake 2.0
    /// Energy quota, and Solana reclaimable PDA rent.
    async fn inspect_gas_relayer_health(
        &self,
        relayer_address: &str,
        min_required_gas: RawAssetAmount,
    ) -> Result<RelayerResourceHealth, SovereignError>;

    /// Estimates the current optimal network fee rate (sat/vB, EIP-1559 wei, micro-lamports, Energy).
    async fn estimate_optimal_fee_rate(&self, target_blocks: u8) -> Result<u128, SovereignError>;
}

```

### 5.7 Engine Trait V: `UnsignedSweepAssemblerEngine` (Dry-Run, RBF/CPFP & Gas Sponsor)

```
// crates/gw-enterprise-traits/src/sweep_engine_trait.rs
#![forbid(unsafe_code)]

use crate::oberer_storage_engine::SovereignEngine;
use async_trait::async_trait;
use gw_primitives::{
    ChainArchitecture, ChainId, PhysicalAssetId, RawAssetAmount, SovereignError, WalletTier,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Deterministic unsigned transaction envelope persisted in `oberer-orm` and sent to Zone C.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnsignedTransactionEnvelope {
    pub envelope_id: Uuid,
    pub chain_id: ChainId,
    pub architecture: ChainArchitecture,
    pub source_tier: WalletTier,
    pub destination_tier: WalletTier,
    pub source_addresses: Vec<String>,
    pub expected_destination_address: String,
    pub physical_asset: PhysicalAssetId,
    pub net_transfer_amount: RawAssetAmount,
    pub max_network_fee_native: RawAssetAmount,
    pub nonce_or_sequence: u64,
    pub raw_unsigned_payload: Vec<u8>,
    pub payload_sha256_hex: String,
    pub signer_key_locators: Vec<String>,
}

/// Pre-broadcast local node dry-run simulation report (`eth_call`, `simulateTransaction`, `testmempoolaccept`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DryRunSimulationReport {
    pub simulation_succeeded: bool,
    pub exact_gas_or_energy_used: u64,
    pub verified_recipient_delta: RawAssetAmount,
    pub reclaimed_rent_or_dust_saved: u64,
    pub revert_reason: Option<String>,
}

/// Engine Trait V: Unsigned Sweep, Batch Consolidation, Fee-Bump & Broadcast Engine.
#[async_trait]
pub trait UnsignedSweepAssemblerEngine: SovereignEngine {
    fn chain_id(&self) -> ChainId;

    /// Assembles an unsigned Tier 0 -> Tier 2/3 customer deposit sweep envelope:
    /// - EVM/Tron: `GatewaySweeperFactory::deployAndSweep(salt, token, keepOneUnitDust)`
    /// - Bitcoin: Batch UTXO consolidation PSBT
    /// - Solana: `sweep_pda_and_close_ata` reclaiming 0.00203928 SOL rent
    /// - Sui: Sponsored PTB (`MergeCoins` + `TransferObjects`)
    async fn assemble_tier0_deposit_sweep(
        &self,
        deposit_locators: &[String],
        destination_vault_address: &str,
        destination_tier: WalletTier,
        asset: &PhysicalAssetId,
        relayer_address: &str,
    ) -> Result<UnsignedTransactionEnvelope, SovereignError>;

    /// Assembles an unsigned institutional treasury transfer (Tier 2 Hot -> Tier 3 Cold Vault
    /// or Tier 2 Hot -> Tier 4 Market Exchange or Tier 2 Hot -> Tier 1 Gas Relayer).
    async fn assemble_treasury_transfer(
        &self,
        source_wallet_address: &str,
        source_key_locator: &str,
        destination_address: &str,
        source_tier: WalletTier,
        destination_tier: WalletTier,
        asset: &PhysicalAssetId,
        amount: RawAssetAmount,
    ) -> Result<UnsignedTransactionEnvelope, SovereignError>;

    /// Executes a deterministic dry-run simulation against the local native node BEFORE
    /// sending the envelope to Zone C for signing.
    async fn simulate_unsigned_envelope(
        &self,
        envelope: &UnsignedTransactionEnvelope,
    ) -> Result<DryRunSimulationReport, SovereignError>;

    /// Constructs an RBF / EIP-1559 same-nonce gas bump or CPFP replacement envelope.
    async fn assemble_fee_bump_replacement(
        &self,
        stuck_envelope: &UnsignedTransactionEnvelope,
        fee_multiplier_bps: u32,
    ) -> Result<UnsignedTransactionEnvelope, SovereignError>;

    /// Broadcasts a Zone-C cryptographically signed transaction payload to the local native node.
    async fn broadcast_signed_transaction(
        &self,
        envelope_id: Uuid,
        signed_raw_bytes: &[u8],
    ) -> Result<String, SovereignError>;
}

```

### 5.8 Engine Trait VI: `UnifiedNetworkAdapterEngine` (Master Polymorphic Blockchain Engine)

```
// crates/gw-enterprise-traits/src/unified_adapter_engine_trait.rs
#![forbid(unsafe_code)]

use crate::balance_engine_trait::StateBalanceReaderEngine;
use crate::derivation_engine_trait::WatchOnlyDerivationEngine;
use crate::indexing_engine_trait::ConsensusIndexingEngine;
use crate::sweep_engine_trait::UnsignedSweepAssemblerEngine;
use async_trait::async_trait;
use gw_primitives::{ChainArchitecture, ChainId, SovereignError};
use serde::{Deserialize, Serialize};

/// Comprehensive diagnostic status of a blockchain adapter engine and its local native node endpoints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdapterDiagnosticSnapshot {
    pub chain_id: ChainId,
    pub architecture: ChainArchitecture,
    pub active_rpc_endpoint: String,
    pub standby_rpc_endpoints_ready: usize,
    pub local_node_synced: bool,
    pub canonical_tip_height: i64,
    pub finalized_height: i64,
    pub p99_rpc_latency_ms: u32,
}

/// Engine Trait VI: Master Polymorphic Network Adapter Supertrait.
/// Unifies Watch-Only Derivation, Consensus Indexing, Balance Reading, and Unsigned Sweep Assembly
/// under the `SovereignEngine` OOP hierarchy.
#[async_trait]
pub trait UnifiedNetworkAdapterEngine:
    WatchOnlyDerivationEngine
    + ConsensusIndexingEngine
    + StateBalanceReaderEngine
    + UnsignedSweepAssemblerEngine
    + Send
    + Sync
{
    fn architecture(&self) -> ChainArchitecture {
        WatchOnlyDerivationEngine::chain_id(self).architecture()
    }

    /// Performs a deep health check of the underlying native node and triggers automatic failover
    /// to a standby local node endpoint if block lag or latency exceeds thresholds.
    async fn check_node_health_and_failover(
        &self,
    ) -> Result<AdapterDiagnosticSnapshot, SovereignError>;
}

```

### 5.9 Engine Trait VII: `InvoiceVerificationEngine` (5-Gate Typestate + `oberer-orm` CAS)

```
// crates/gw-enterprise-traits/src/verification_engine_trait.rs
#![forbid(unsafe_code)]

use crate::derivation_engine_trait::CheckoutDepositRoute;
use crate::indexing_engine_trait::NormalizedTransferEvent;
use crate::oberer_storage_engine::SovereignEngine;
use async_trait::async_trait;
use gw_primitives::{PhysicalAssetId, RawAssetAmount, SovereignError};
use oberer_orm::MicroCredits;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Compile-Time Typestate Lifecycle Markers
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatePending;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateDetected {
    pub current_confirmations: u32,
    pub target_confirmations: u32,
    pub candidate_tx_hash: String,
    pub detected_at_block: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatePartial {
    pub shortfall_raw: RawAssetAmount,
    pub received_tx_hashes: Vec<String>,
    pub extended_grace_expires_at_unix: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateVerifiedComplete {
    pub settlement_tx_hashes: Vec<String>,
    pub settled_at_unix: i64,
    pub is_overpaid: bool,
    pub surplus_raw: RawAssetAmount,
    pub fulfillment_idempotency_token: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateExpired {
    pub expired_at_unix: i64,
    pub late_received_raw: RawAssetAmount,
}

/// Generic Typestate Invoice Container enforcing Axiom 6 at compile time.
/// Uses `oberer_orm::MicroCredits` for exact integer fiat/oracle pricing and `i64` version for CAS.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypestateInvoice<S> {
    pub invoice_id: i64,
    pub customer_id: Option<Uuid>,
    pub asset: PhysicalAssetId,
    pub route: CheckoutDepositRoute,
    pub fiat_amount_mc: MicroCredits,
    pub locked_price_mc: MicroCredits,
    pub expected_amount_raw: RawAssetAmount,
    pub cumulative_received_raw: RawAssetAmount,
    pub tolerance_bps: i64,
    pub expires_at_unix: i64,
    pub version: i64,
    pub row_mac: Vec<u8>,
    pub state: S,
}

/// Algebraic union of valid typestate transitions produced by the 5-Gate Engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerifiedTransitionOutcome {
    RemainsDetected(TypestateInvoice<StateDetected>),
    MovedToPartial(TypestateInvoice<StatePartial>),
    MovedToVerifiedComplete(TypestateInvoice<StateVerifiedComplete>),
    MovedToExpired(TypestateInvoice<StateExpired>),
}

/// Engine Trait VII: 5-Gate Typestate Invoice Verification Engine backed by `IMPRCE` & `oberer-orm`.
#[async_trait]
pub trait InvoiceVerificationEngine: SovereignEngine {
    /// Creates a new locked quote `TypestateInvoice<StatePending>`:
    /// 1. Verifies dual-oracle deviation (\delta_{bps} <= 150) via `MicroCreditsMathBridge`.
    /// 2. Applies 18-decimal clean truncation \mathcal{T}_{clean}.
    /// 3. Inserts the sealed `InvoiceRow` into `oberer-orm` `HybridConn` via `Table::<InvoiceRow>::insert(&row).unenvelope()?`.
    /// 4. Registers the lookup key in the P1 triple-redundant `IMPRCE` watchlist and L1 Bloom filter.
    async fn create_and_persist_locked_invoice(
        &self,
        customer_id: Option<Uuid>,
        asset: PhysicalAssetId,
        fiat_amount_mc: MicroCredits,
        cex_mid_price_mc: MicroCredits,
        onchain_oracle_price_mc: MicroCredits,
        tolerance_bps: i64,
        quote_ttl_seconds: i64,
    ) -> Result<TypestateInvoice<StatePending>, SovereignError>;

    /// Evaluates a `StatePending` invoice against an incoming `NormalizedTransferEvent` across all 5 Gates
    /// and atomically commits the state transition in `oberer-orm` via `table.update_with_version(id, version, &row).unenvelope()?`
    /// while recording the idempotency barrier in `oberer_orm::runtime::IdempotencyService`.
    async fn evaluate_pending_invoice(
        &self,
        invoice: TypestateInvoice<StatePending>,
        event: &NormalizedTransferEvent,
        parent_hash_continuity_ok: bool,
    ) -> Result<VerifiedTransitionOutcome, SovereignError>;

    /// Advances a `StateDetected` invoice as new blocks confirm the transaction toward finality.
    async fn advance_detected_invoice(
        &self,
        invoice: TypestateInvoice<StateDetected>,
        updated_confirmations: u32,
        parent_hash_continuity_ok: bool,
    ) -> Result<VerifiedTransitionOutcome, SovereignError>;

    /// Evaluates a top-up payment arriving for a `StatePartial` invoice within the extended grace window.
    async fn evaluate_partial_topup(
        &self,
        invoice: TypestateInvoice<StatePartial>,
        topup_event: &NormalizedTransferEvent,
        parent_hash_continuity_ok: bool,
    ) -> Result<VerifiedTransitionOutcome, SovereignError>;

    /// Atomically rolls back all `StateDetected` invoices with `detected_at_block > lca_height`
    /// back to `StatePending` using `oberer-orm`'s typed query filter:
    /// `InvoiceRow::status().eq(InvoiceLifecycleStatus::Detected).and(InvoiceRow::detected_at_block().gt(lca_height))`
    async fn rollback_reorged_invoices_above_lca(
        &self,
        lca_height: i64,
    ) -> Result<Vec<TypestateInvoice<StatePending>>, SovereignError>;
}

```

### 5.10 Engine Trait VIII: `PreSignPolicyGuardEngine` (Zone C Stateless AST/Bytecode Verifier)

```
// crates/gw-enterprise-traits/src/policy_guard_engine_trait.rs
#![forbid(unsafe_code)]

use crate::oberer_storage_engine::SovereignEngine;
use crate::sweep_engine_trait::UnsignedTransactionEnvelope;
use async_trait::async_trait;
use gw_primitives::{ChainId, RawAssetAmount, SovereignError, WalletTier};
use serde::{Deserialize, Serialize};

/// Cryptographic M-of-N hardware approval signature submitted by an institutional custodian.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HardwareQuorumSignature {
    pub signer_id: String,
    pub signer_public_key: Vec<u8>,
    pub signature_bytes: Vec<u8>,
    pub signed_at_unix: i64,
}

/// Cryptographic proof emitted by the Stateless Pre-Sign Policy Guard Engine after parsing raw bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyVerificationReceipt {
    pub verified_payload_sha256: [u8; 32],
    pub parsed_destination_addresses: Vec<String>,
    pub parsed_net_output_amount: RawAssetAmount,
    pub parsed_network_fee: RawAssetAmount,
    pub verified_quorum_signers: usize,
}

/// Engine Trait VIII: Zone C Stateless Pre-Sign Bytecode, PSBT & AST Policy Guard Engine.
/// Enforces Axiom 3: No key material can be accessed without a valid `PolicyVerificationReceipt`.
#[async_trait]
pub trait PreSignPolicyGuardEngine: SovereignEngine {
    /// Deserializes `envelope.raw_unsigned_payload` directly from raw bytes (BIP-174 PSBT, EIP-1559 RLP,
    /// Solana Message, Tron protobuf, TON BOC, Sui BCS) and verifies:
    /// 1. 100% of non-change output addresses belong to the PCR0 hardware-pinned Cold/Market whitelist.
    /// 2. Parsed destination and amount match the envelope header.
    /// 3. Envelope SHA-256 digest has never been signed before (Anti-Replay check).
    /// 4. M-of-N hardware quorum signatures are valid for Tier 2 outflows.
    async fn inspect_and_authorize_raw_payload(
        &self,
        envelope: &UnsignedTransactionEnvelope,
        quorum_signatures: &[HardwareQuorumSignature],
    ) -> Result<PolicyVerificationReceipt, SovereignError>;

    /// Checks whether `(chain_id, destination_tier, address)` is present in the PCR0 pinned whitelist.
    fn is_pinned_whitelisted_destination(
        &self,
        chain_id: ChainId,
        destination_tier: WalletTier,
        address: &str,
    ) -> bool;
}

```

### 5.11 Engine Trait IX: `EnclaveKeySignerEngine` (Zeroized RAM & M-of-N Quorum Signer)

```
// crates/gw-enterprise-traits/src/enclave_signer_engine_trait.rs
#![forbid(unsafe_code)]

use crate::oberer_storage_engine::SovereignEngine;
use crate::policy_guard_engine_trait::{
    HardwareQuorumSignature, PolicyVerificationReceipt, PreSignPolicyGuardEngine,
};
use crate::sweep_engine_trait::UnsignedTransactionEnvelope;
use async_trait::async_trait;
use gw_primitives::{ChainId, SovereignError};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Signed transaction artifact emitted by the Zone C Enclave after zeroizing temporary RAM buffers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedTransactionArtifact {
    pub envelope_id: Uuid,
    pub chain_id: ChainId,
    pub verified_payload_sha256: [u8; 32],
    pub broadcastable_signed_bytes: Vec<u8>,
    pub expected_tx_hash: String,
    pub ephemeral_ram_zeroized: bool,
}

/// Engine Trait IX: Unified Zero-Trust Enclave Key Signer Engine (Secp256k1, Ed25519, Schnorr, MPC).
#[async_trait]
pub trait EnclaveKeySignerEngine: SovereignEngine {
    /// Executes the two-step Enclave Signing Ceremony:
    /// Step 1: Invokes `policy_guard.inspect_and_authorize_raw_payload(envelope, quorum_signatures)`.
    /// Step 2: Unseals/derives the child signing key inside a `Zeroizing<[u8; 64]>` buffer,
    ///         produces the raw signature, zeroes the buffer via volatile fence, and returns the artifact.
    async fn authorize_and_sign_envelope(
        &self,
        policy_guard: &dyn PreSignPolicyGuardEngine,
        envelope: &UnsignedTransactionEnvelope,
        quorum_signatures: &[HardwareQuorumSignature],
    ) -> Result<(PolicyVerificationReceipt, SignedTransactionArtifact), SovereignError>;

    /// Exports only the extended public key (`zpub`) or view/relayer public key for Zone A bootstrap.
    async fn export_watch_only_public_descriptor(
        &self,
        chain_id: ChainId,
        key_locator: &str,
    ) -> Result<String, SovereignError>;

    /// Generates a cryptographic Nitro Enclave / TPM attestation document binding PCR0 and the whitelist hash.
    async fn generate_enclave_attestation_report(
        &self,
        nonce: [u8; 32],
    ) -> Result<Vec<u8>, SovereignError>;
}

```

### 5.12 Engine Trait X: `TreasuryPortfolioEngine` (Unified Valuation & FIFO Lot Calculus)

```
// crates/gw-enterprise-traits/src/treasury_engine_trait.rs
#![forbid(unsafe_code)]

use crate::oberer_storage_engine::SovereignEngine;
use crate::verification_engine_trait::{StateVerifiedComplete, TypestateInvoice};
use alloy_primitives::U256;
use async_trait::async_trait;
use gw_primitives::{
    ChainId, LogicalAsset, PhysicalAssetId, RawAssetAmount, SovereignError, WalletTier,
};
use oberer_orm::MicroCredits;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

/// Immutable FIFO Tax Lot domain model backed by `CostBasisLotRow` in `oberer-orm`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FifoCostBasisLot {
    pub lot_id: i64,
    pub lot_uuid: String,
    pub source_invoice_id: i64,
    pub asset: PhysicalAssetId,
    pub initial_amount: RawAssetAmount,
    pub remaining_amount: RawAssetAmount,
    pub acquisition_price_mc: MicroCredits,
    pub acquired_at_unix: i64,
    pub version: i64,
    pub row_mac: Vec<u8>,
}

/// Unified cross-network portfolio rollup for a single LogicalAsset (e.g., Total USDT across all 6 chains).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnifiedAssetPortfolioSummary {
    pub logical_asset: LogicalAsset,
    pub total_canonical_18_units: U256,
    pub total_market_valuation_mc: MicroCredits,
    pub total_fifo_cost_basis_mc: MicroCredits,
    pub unrealized_pnl_microcredits: i64,
    pub valuation_by_tier_mc: BTreeMap<WalletTier, MicroCredits>,
    pub valuation_by_chain_mc: BTreeMap<ChainId, MicroCredits>,
}

/// Hash-chained audit record persisted in `oberer-orm` `RealizedPnlLedgerRow` upon FIFO disposal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RealizedPnlSettlementReport {
    pub settlement_id: i64,
    pub settlement_uuid: Uuid,
    pub sequence_no: i64,
    pub logical_asset: LogicalAsset,
    pub chain_id: ChainId,
    pub disposed_amount: RawAssetAmount,
    pub consumed_lot_slices: Vec<(i64, RawAssetAmount, MicroCredits)>,
    pub total_cost_basis_mc: MicroCredits,
    pub gross_proceeds_mc: MicroCredits,
    pub network_fee_burned_mc: MicroCredits,
    pub net_realized_pnl_microcredits: i64,
    pub prev_chain_hash_hex: String,
    pub entry_chain_hash_hex: String,
}

/// Engine Trait X: Unified Cross-Network Asset Aggregation & FIFO Cost-Basis Calculus Engine.
#[async_trait]
pub trait TreasuryPortfolioEngine: SovereignEngine {
    /// Creates and seals an immutable `CostBasisLotRow` inside `oberer-orm` `HybridConn`
    /// upon `TypestateInvoice<StateVerifiedComplete>`.
    async fn record_verified_invoice_fifo_lot(
        &self,
        verified_invoice: &TypestateInvoice<StateVerifiedComplete>,
    ) -> Result<FifoCostBasisLot, SovereignError>;

    /// Aggregates all wallet holdings across Bitcoin, Ethereum, BNB, Plume, Solana, Tron, TON, and Sui
    /// from `oberer-orm` `Table::<AssetBalanceRow>` and `Table::<CostBasisLotRow>` into a unified
    /// per-LogicalAsset balance sheet with exact Unrealized P&L in `MicroCredits`.
    async fn compute_unified_global_portfolio(
        &self,
        spot_prices_mc: &BTreeMap<LogicalAsset, MicroCredits>,
    ) -> Result<BTreeMap<LogicalAsset, UnifiedAssetPortfolioSummary>, SovereignError>;

    /// Consumes open FIFO tax lots in strict chronological order (`CostBasisLotRow::acquired_at_unix()`)
    /// inside an atomic `oberer-orm` `conn.transaction(|tx| ...)` block, updating each lot via
    /// `update_with_version` and appending a hash-chained `RealizedPnlLedgerRow`.
    async fn execute_fifo_lot_consumption(
        &self,
        asset: &PhysicalAssetId,
        disposed_amount: RawAssetAmount,
        execution_price_mc: MicroCredits,
        network_fee_mc: MicroCredits,
    ) -> Result<RealizedPnlSettlementReport, SovereignError>;
}

```

### 5.13 Engine Trait XI: `CrossNetworkLiquidityRouterEngine` (Optimal Transport & Gas Refill)

```
// crates/gw-enterprise-traits/src/liquidity_router_engine_trait.rs
#![forbid(unsafe_code)]

use crate::oberer_storage_engine::SovereignEngine;
use crate::sweep_engine_trait::UnsignedTransactionEnvelope;
use async_trait::async_trait;
use gw_primitives::{ChainId, LogicalAsset, PhysicalAssetId, RawAssetAmount, SovereignError};
use oberer_orm::MicroCredits;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Per-network liquidity threshold and split allocation policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkLiquidityPolicy {
    pub asset: PhysicalAssetId,
    pub tier1_relayer_address: String,
    pub tier2_hot_wallet_address: String,
    pub tier2_key_locator: String,
    pub tier3_cold_vault_address: String,
    pub tier4_market_exchange_address: String,
    pub hot_wallet_ceiling_mc: MicroCredits,
    pub cold_vault_split_bps: u16,
    pub market_exchange_split_bps: u16,
    pub min_relayer_gas_raw: RawAssetAmount,
    pub target_relayer_gas_raw: RawAssetAmount,
}

/// Global routing execution mode across all chains.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GlobalRoutingDirective {
    EmergencyOrScheduledFlushToCold,
    FlushAllToMarketExchange,
    PolicyDrivenProportionalSplit,
}

/// Engine Trait XI: Cross-Network Liquidity Routing, Relayer Auto-Refill & Consolidation Engine.
#[async_trait]
pub trait CrossNetworkLiquidityRouterEngine: SovereignEngine {
    /// Evaluates all Tier 1 Gas Relayers across all networks and generates unsigned top-up
    /// envelopes from Tier 2 Operational Hot Wallets whenever `balance < min_relayer_gas_raw`.
    async fn plan_tier1_relayer_gas_refills(
        &self,
        policies: &[NetworkLiquidityPolicy],
    ) -> Result<Vec<UnsignedTransactionEnvelope>, SovereignError>;

    /// Plans optimal cross-network transfers of a specific LogicalAsset or the entire portfolio
    /// to Cold Storage and/or Market Exchanges, persisting `TreasuryTransferRow` in `oberer-orm`.
    async fn plan_global_liquidity_transfers(
        &self,
        filter_asset: Option<LogicalAsset>,
        directive: GlobalRoutingDirective,
        policies: &[NetworkLiquidityPolicy],
        spot_prices_mc: &BTreeMap<LogicalAsset, MicroCredits>,
    ) -> Result<Vec<UnsignedTransactionEnvelope>, SovereignError>;

    /// Computes the minimum economical sweep threshold for Tier 0 customer deposit addresses.
    async fn calculate_economical_sweep_threshold(
        &self,
        chain_id: ChainId,
        asset: &PhysicalAssetId,
        max_fee_to_principal_bps: u16,
    ) -> Result<RawAssetAmount, SovereignError>;
}

```

### 5.14 Master Orchestrator: `SovereignGatewayRuntimeEngine` (Composition Root)

```
// crates/gw-enterprise-traits/src/runtime_engine.rs
#![forbid(unsafe_code)]

use crate::cache_engine_trait::PriorityRedundantCacheEngine;
use crate::enclave_signer_engine_trait::EnclaveKeySignerEngine;
use crate::liquidity_router_engine_trait::CrossNetworkLiquidityRouterEngine;
use crate::oberer_storage_engine::{
    EngineHealthSnapshot, EngineIntegrityReport, ObererOrmStorageEngine, SovereignEngine,
};
use crate::policy_guard_engine_trait::PreSignPolicyGuardEngine;
use crate::treasury_engine_trait::TreasuryPortfolioEngine;
use crate::unified_adapter_engine_trait::UnifiedNetworkAdapterEngine;
use crate::verification_engine_trait::InvoiceVerificationEngine;
use async_trait::async_trait;
use gw_primitives::{ChainId, SecurityZone, SovereignError};
use std::collections::BTreeMap;
use std::sync::Arc;

/// Master OOP Composition Root & Runtime Engine.
/// Orchestrates `oberer-orm` HybridConn storage, `IMPRCE` k-way priority cache, all 8 blockchain
/// adapter engines, the 5-Gate invoice verification engine, the FIFO treasury engine, and the
/// Zone C enclave policy/signing bridge.
#[derive(Clone)]
pub struct SovereignGatewayRuntimeEngine {
    pub storage_engine: Arc<dyn ObererOrmStorageEngine>,
    pub cache_engine: Arc<dyn PriorityRedundantCacheEngine>,
    pub network_adapters: BTreeMap<ChainId, Arc<dyn UnifiedNetworkAdapterEngine>>,
    pub verification_engine: Arc<dyn InvoiceVerificationEngine>,
    pub treasury_engine: Arc<dyn TreasuryPortfolioEngine>,
    pub liquidity_router_engine: Arc<dyn CrossNetworkLiquidityRouterEngine>,
    pub enclave_policy_guard: Option<Arc<dyn PreSignPolicyGuardEngine>>,
    pub enclave_signer: Option<Arc<dyn EnclaveKeySignerEngine>>,
}

#[async_trait]
impl SovereignEngine for SovereignGatewayRuntimeEngine {
    fn engine_id(&self) -> &'static str {
        "SOVEREIGN_GATEWAY_MASTER_RUNTIME_ENGINE_V9"
    }

    fn security_zone(&self) -> SecurityZone {
        SecurityZone::ZoneAPublicWatchOnly
    }

    async fn initialize_engine(&self) -> Result<(), SovereignError> {
        // 1. Initialize `oberer-orm` HybridConn, V001/V002 Schema<T> migrations, Views & Triggers
        self.storage_engine.initialize_engine().await?;

        // 2. Initialize IMPRCE and warm-hydrate P0/P1/P2 from `oberer-orm` HybridConn in <10ms
        self.cache_engine.initialize_engine().await?;
        self.cache_engine
            .hydrate_from_oberer_hybrid(self.storage_engine.as_ref())
            .await?;

        // 3. Initialize all registered blockchain adapter engines
        for adapter in self.network_adapters.values() {
            adapter.initialize_engine().await?;
        }

        // 4. Initialize verification, treasury, and liquidity routing engines
        self.verification_engine.initialize_engine().await?;
        self.treasury_engine.initialize_engine().await?;
        self.liquidity_router_engine.initialize_engine().await?;

        Ok(())
    }

    async fn audit_cryptographic_integrity(&self) -> Result<EngineIntegrityReport, SovereignError> {
        let storage_audit = self.storage_engine.audit_cryptographic_integrity().await?;
        let cache_audit = self.cache_engine.audit_cryptographic_integrity().await?;

        Ok(EngineIntegrityReport {
            engine_id: self.engine_id().to_string(),
            security_zone: self.security_zone(),
            verified_records_or_cells: storage_audit
                .verified_records_or_cells
                .saturating_add(cache_audit.verified_records_or_cells),
            detected_corruptions_or_tampers: storage_audit
                .detected_corruptions_or_tampers
                .saturating_add(cache_audit.detected_corruptions_or_tampers),
            autonomous_self_heals: storage_audit
                .autonomous_self_heals
                .saturating_add(cache_audit.autonomous_self_heals),
            hash_chain_head_hex: storage_audit.hash_chain_head_hex,
            audit_passed: storage_audit.audit_passed && cache_audit.audit_passed,
        })
    }

    async fn health_snapshot(&self) -> Result<EngineHealthSnapshot, SovereignError> {
        let storage_health = self.storage_engine.health_snapshot().await?;
        let cache_health = self.cache_engine.health_snapshot().await?;

        Ok(EngineHealthSnapshot {
            engine_id: self.engine_id().to_string(),
            security_zone: self.security_zone(),
            is_healthy: storage_health.is_healthy && cache_health.is_healthy,
            active_workers_or_shards: storage_health
                .active_workers_or_shards
                .saturating_add(cache_health.active_workers_or_shards),
            total_operations_processed: storage_health
                .total_operations_processed
                .saturating_add(cache_health.total_operations_processed),
            p99_operation_latency_us: storage_health
                .p99_operation_latency_us
                .max(cache_health.p99_operation_latency_us),
            memory_allocated_bytes: storage_health
                .memory_allocated_bytes
                .saturating_add(cache_health.memory_allocated_bytes),
        })
    }

    async fn shutdown_gracefully(&self) -> Result<(), SovereignError> {
        self.liquidity_router_engine.shutdown_gracefully().await?;
        self.treasury_engine.shutdown_gracefully().await?;
        self.verification_engine.shutdown_gracefully().await?;
        for adapter in self.network_adapters.values() {
            adapter.shutdown_gracefully().await?;
        }
        self.cache_engine.shutdown_gracefully().await?;
        self.storage_engine.shutdown_gracefully().await?;
        Ok(())
    }
}

```

## 6. Zero-Raw-SQL Type-Enforced `Schema<T>` Chainable Migration, Index, View & Trigger Engine

### 6.1 Mathematical Theory of Phantom-Typed Relational Algebra & Zero-Raw-SQL DDL

In traditional ORMs (and earlier schema configurations), developers embed raw SQL strings inside macro attributes such as `#[orm(check = "remaining_amount <= initial_amount")]`, `#[orm(index = "col1,col2|unique|where=status = 'PENDING'")]`, or `#[orm(view_sql = "SELECT ...")]`.

Raw string literals in schema definitions violate **Axiom 6** and **Axiom 9** for three reasons:

1. **Silent Identifier Drift:** Renaming a struct field or changing an enum variant compiles cleanly in Rust while silently breaking the raw SQL string inside `check = "..."` or `index = "...|where=..."`.

2. **Type-System Bypass:** String attributes allow comparing incompatible types (e.g., comparing a `BLOB` column to an integer or comparing two columns of different types) without a compile-time error.

3. **Unverified View Projections:** A raw `view_sql` string does not verify at compile time that the `SELECT` projection column types $V_1, \dots, V_k$ match the target `TableView<V>` struct fields $V_1', \dots, V_k'$.

#### The Phantom-Typed `Schema<T>` & `ViewSchema<V>` Algebra

Let every `#[derive(OrmTable)]` struct $T$ emit phantom-typed column descriptors $\text{Column}\langle T, V \rangle$ where $T$ binds the owning table type and $V$ binds the Rust scalar/enum/blob type. We enforce that **all** DDL statements are constructed via chainable combinators:

* **Column-to-Column & Scalar Check Invariants (**$\mathcal{C}\langle T \rangle$**):** Given $c_1 : \text{Column}\langle T, V \rangle$ and $c_2 : \text{Column}\langle T, V \rangle$, the method `c1.lte_col(c2)` is well-typed **if and only if** both columns belong to table $T$ and share the exact Rust type $V$.

* **Typed Composite & Partial Indexes (**$\mathcal{I}\langle T \rangle$**):** An index builder `IndexSchema::<T>::new("idx_name")` accepts only columns $\text{Column}\langle T, V_i \rangle$ belonging to $T$, and its partial index filter `.where_clause(pred)` accepts only a typed `Predicate<T>` constructed from `Column<T, V>` operators (`.eq()`, `.in_values()`, `.is_null()`, `.is_not_null()`).

* **Type-Enforced View Projections (**$\Pi_{S \to V}$**):** A view builder `ViewSchema::<V>::from::<S>(conn)` requires every projected column mapping `.map_col(source_col, view_col)` to satisfy the compile-time type equality constraint:

  $$
  \text{map\_col} : \text{Column}\langle S, \tau \rangle \times \text{Column}\langle V, \tau \rangle \to \text{ViewSchema}\langle V, S \rangle
  $$

  If `source_col` in source table $S$ has type `MicroCredits` and `view_col` in view struct $V$ has type `i64` (or `String`), **compilation fails immediately**!

### 6.2 The Fluent Chainable `Schema<T>`, `IndexSchema<T>`, `ViewSchema<V>` & `TriggerSchema<T>` DSL

```
// crates/gw-oberer-models/src/typed_schema_dsl.rs
#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic, clippy::unwrap_used)]

use gw_primitives::SovereignError;
use oberer_orm::conn::OrmConn;
use oberer_orm::prelude::*;
use oberer_orm::query::Predicate;
use oberer_orm::schema::trigger_dsl::{New, Old, TriggerExpr, TriggerInsert};
use std::marker::PhantomData;

/// Extension trait on `oberer_orm::Column<T, V>` providing 100% compile-time type-checked
/// column-to-column comparisons and SQLite BLOB/TEXT length invariants without raw SQL strings.
pub trait TypedColumnCheckExt<T: OrmTable, V> {
    /// Enforces `col_a <= col_b` where both columns MUST belong to table `T` and share type `V`.
    fn lte_col(&self, rhs: Column<T, V>) -> Predicate<T>;

    /// Enforces `col_a < col_b` where both columns MUST belong to table `T` and share type `V`.
    fn lt_col(&self, rhs: Column<T, V>) -> Predicate<T>;

    /// Enforces `col_a >= col_b` where both columns MUST belong to table `T` and share type `V`.
    fn gte_col(&self, rhs: Column<T, V>) -> Predicate<T>;

    /// Enforces `col_a == col_b` where both columns MUST belong to table `T` and share type `V`.
    fn eq_col(&self, rhs: Column<T, V>) -> Predicate<T>;
}

/// Extension trait specifically for `Column<T, Vec<u8>>` (`BLOB`) and `Column<T, String>` (`TEXT`)
/// to enforce exact cryptographic byte/character lengths (e.g., 32-byte U256 BLOBs or 78-char TEXT)
/// via type-safe AST predicates rather than raw SQL `length(...) = 32` strings.
pub trait TypedByteLengthCheckExt<T: OrmTable> {
    /// Emits `length(col) = expected_len` in the typed AST.
    fn len_eq(&self, expected_len: usize) -> Predicate<T>;
}

/// Type-Enforced Chainable Index Builder (`IndexSchema<T>`).
/// Prevents raw index strings (`#[orm(index = "a,b|unique|where=...")]`).
/// Every column and partial index `WHERE` predicate is verified against `T` at compile time!
pub struct IndexSchema<T: OrmTable> {
    index_name: &'static str,
    columns: Vec<&'static str>,
    is_unique: bool,
    partial_where: Option<Predicate<T>>,
    _marker: PhantomData<T>,
}

impl<T: OrmTable> IndexSchema<T> {
    pub const fn named(index_name: &'static str) -> Self {
        Self {
            index_name,
            columns: Vec::new(),
            is_unique: false,
            partial_where: None,
            _marker: PhantomData,
        }
    }

    /// Appends a typed column `Column<T, V>` to the composite index key list.
    /// Attempting to pass a column from a different table `U != T` is a compile-time error!
    pub fn col<V>(mut self, column: Column<T, V>) -> Self {
        self.columns.push(column.name());
        self
    }

    /// Marks the index as `UNIQUE`.
    pub const fn unique(mut self) -> Self {
        self.is_unique = true;
        self
    }

    /// Attaches a compile-time type-checked partial index predicate (`WHERE <Predicate<T>>`).
    pub fn where_clause(mut self, predicate: Predicate<T>) -> Self {
        self.partial_where = Some(predicate);
        self
    }

    pub fn index_name(&self) -> &'static str {
        self.index_name
    }

    pub fn columns(&self) -> &[&'static str] {
        &self.columns
    }

    pub const fn is_unique(&self) -> bool {
        self.is_unique
    }

    pub fn partial_where(&self) -> Option<&Predicate<T>> {
        self.partial_where.as_ref()
    }
}

/// Fluent, Chainable, Zero-Raw-SQL Schema Migration Builder (`Schema<T>`).
/// Wraps `oberer-orm`'s `Table::<T>::new(conn).schema_query()` and enforces that all
/// table constraints, indexes, and foreign keys are constructed via `Column<T, V>` markers.
pub struct Schema<T: OrmTable> {
    conn: OrmConn,
    checks: Vec<Predicate<T>>,
    indexes: Vec<IndexSchema<T>>,
    _marker: PhantomData<T>,
}

impl<T: OrmTable> Schema<T> {
    /// Creates a new type-enforced chainable `Schema<T>` builder for struct `T`.
    pub fn new(conn: OrmConn) -> Self {
        Self {
            conn,
            checks: Vec::new(),
            indexes: Vec::new(),
            _marker: PhantomData,
        }
    }

    /// Adds a compile-time type-safe table `CHECK(...)` constraint constructed from `Column<T, V>` predicates.
    pub fn check(mut self, predicate: Predicate<T>) -> Self {
        self.checks.push(predicate);
        self
    }

    /// Adds a compile-time type-safe single, composite, unique, or partial index via a closure over `IndexSchema<T>`.
    pub fn index<F>(mut self, build_fn: F) -> Self
    where
        F: FnOnce(IndexSchema<T>) -> IndexSchema<T>,
    {
        let idx = build_fn(IndexSchema::<T>::named("idx_auto"));
        self.indexes.push(idx);
        self
    }

    /// Adds a named compile-time type-safe index via `IndexSchema<T>`.
    pub fn index_named<F>(mut self, name: &'static str, build_fn: F) -> Self
    where
        F: FnOnce(IndexSchema<T>) -> IndexSchema<T>,
    {
        let idx = build_fn(IndexSchema::<T>::named(name));
        self.indexes.push(idx);
        self
    }

    /// Registers a compile-time type-checked Foreign Key (`Column<T, V> -> Column<Parent, V>`)
    /// with `ON DELETE CASCADE` and `belongs_to` relation

```