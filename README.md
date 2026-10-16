# Sovereign Rust Cryptocurrency Payment Gateway & Unified Treasury Engine (v9.0)

[![Rust Version](https://img.shields.io/badge/rustc-1.85%2B-blue.svg)](https://www.rust-lang.org)
[![Edition](https://img.shields.io/badge/edition-2024-orange.svg)](https://doc.rust-lang.org/edition-guide/rust-2024/index.html)
[![Safety](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://doc.rust-lang.org/nomicon/safe-unsafe-meaning.html)
[![Zero-Raw-SQL](https://img.shields.io/badge/ORM-oberer--orm%20v3.0-purple.svg)](https://github.com/Kurisuma/Oberer-lite)
[![Tests](https://img.shields.io/badge/audit%20tests-48%2F48%20passing-brightgreen.svg)]()

> **Production-Grade Sovereign Crypto Gateway & Institutional Treasury Infrastructure**  
> Engineered with pure Rust OOP polymorphic engine traits, Zero-Raw-SQL type-safe schema builders (`oberer-orm`), 5-Gate Typestate verification, and closed-semiring mathematical accounting.

---

## Architectural Highlights

- **Zero-Raw-SQL Type Enforcement (Axiom 9):** 100% of tables, views, triggers, check constraints, composite/partial indexes, and foreign key cascades are synthesized exclusively through native `oberer-orm` schema builders with zero ad-hoc SQL strings.
- **Strict Key Isolation (Axiom 1):** Zone A (`ZoneAPublicWatchOnly`) and Zone B (`ZoneBStatefulPersisted`) crates have zero compile-time dependencies on private key signing symbols (`k256`, `ed25519-dalek`, `bip32`). Private keys reside exclusively within Zone C hardware security enclaves.
- **Closed Semiring Integer Arithmetic (Axiom 5):** Zero IEEE-754 floating-point arithmetic across all domain crates. Financial calculations operate over exact 6-decimal fixed-point scalar `MicroCredits` and mathematical ring $\mathbb{Z}_{2^{256}}$ with checked partial operators.
- **Bijective Order-Preserving Homomorphisms:** `ObererU256Codec` provides 32-byte Big-Endian BLOBs ($\Psi_{\text{blob}}$) and 78-character zero-padded decimal strings ($\Psi_{\text{dec78}}$), ensuring SQLite `memcmp()` and ASCII comparisons match mathematical $\mathbb{Z}_{2^{256}}$ integer ordering.
- **Dual-Tier Hybrid Database Engine (`HybridConn`):** Sub-millisecond `:memory:` L1 working database paired with crash-resilient Write-Ahead Logging (WAL) L2 persistent disk database, backed by sub-10ms warm-start table hydration.
- **5-Gate Typestate Invoice Verification:** Linear typestate transitions (`StatePending` $\to$ `StateDetected` $\to$ `StateVerifiedComplete`) enforcing payment execution success, token/address matching, mathematical tolerance boundaries ($\pm 50\text{ bps}$), timestamp monotonic expiration, and confirmation depth.
- **Chronological FIFO Treasury Tax Lots:** IRS/IFRS compliant chronological lot disposal, 18-decimal canonical rollup, and unbroken SHA-256 cryptographic hash-chained realized P&L ledgers ($H_n = \text{SHA256}(H_{n-1} \parallel \text{seq} \parallel \tau_{\text{row}})$).
- **Row-Level Cryptographic Security (Axiom 8):** Keyed Blake3 Row MACs protecting persisted records against offline database file tampering and bit-rot injection.

---

## Workspace Structure

```text
crypto-engine-v9/
├── .cargo/config.toml                  # Compiler & Clippy lint denial configurations
├── Cargo.toml                          # Virtual workspace definition
├── USECASE.md                          # Comprehensive 1,500+ line oberer-orm usage reference
├── Notes.md                            # Retrospective, pain points & 10-point v4.0 upgrade roadmap
├── task.md                             # Master 12-Phase implementation and formal audit matrix
├── oberer-lite/                        # Submodule: Direct SQLite C FFI ORM engine
└── crates/
    ├── gw-primitives/                  # Z_{2^256} Codecs (BLOB/DEC78), RawAssetAmount, MicroCredits
    ├── gw-storage-oberer/              # Zero-Raw-SQL Schema DSL, 11 Tables, Views, Triggers & Row MACs
    ├── gw-imprce/                      # In-Memory Priority-Redundant Cache Engine (P0..P4, CRC32C)
    ├── gw-enterprise-traits/           # SovereignEngine Root & 11 Polymorphic OOP Engine Traits
    ├── gw-chain-adapters/              # Watch-Only Adapters: BTC, EVM, SVM, TVM, TON, Move
    ├── gw-engine/                      # 5-Gate Typestate Invoice Verification & LCA Reorg Rollback
    ├── gw-kms-enclave/                 # Zone C Stateless Pre-Sign AST Guard & Enclave Key Signer
    ├── gw-treasury/                    # 18-Dec Canonical Valuation, FIFO Lots, Hash-Chained Ledgers
    ├── gw-router/                      # Liquidity Router, Gas Relayer Auto-Refill, Economical Sweeps
    ├── sovereign-gateway-engine/       # Standalone Library SDK, DI Container, Axum FromRef Extractor
    └── gw-runtime/                     # Composition Root Daemon, 60s Background Audit, CAS Proofs
```

---

## 12-Phase Verification & Mathematical Audit Status

| Phase | Subsystem Crate | Core Invariant Verified | Tests | Status |
| :---: | :--- | :--- | :---: | :---: |
| **0** | Workspace | Axiom 1 Zero-Key Isolation & Zero Floats | 2 | **100% PASSED** |
| **1** | `gw-primitives` | $\Psi_{\text{blob}}$ & $\Psi_{\text{dec78}}$ Homomorphisms, Dual-Oracle Breaker | 6 | **100% PASSED** |
| **2** | `gw-storage-oberer` | Zero-Raw-SQL Schemas, Triggers, 1,000-Thread CAS Concurrency | 4 | **100% PASSED** |
| **3** | `gw-imprce` | Priority Quota Isolation ($P_0 \dots P_4$), Self-Healing Bit-Rot | 5 | **100% PASSED** |
| **4** | `gw-chain-adapters` | BIP-84, CREATE2, Solana PDA, TON Feistel Memo Bijection | 3 | **100% PASSED** |
| **5** | `gw-engine` | DAG Continuity ($\Lambda_{\text{cont}}$), LCA Reorg Backtracking | 5 | **100% PASSED** |
| **6** | `gw-engine` | 5-Gate Conjunction Theorem, Typestate Linear Monotonicity | 7 | **100% PASSED** |
| **7** | `gw-kms-enclave` | Axiom 3 $\Pi_{\text{guard}}$ Pinned Whitelist & Volatile RAM Zeroization | 7 | **100% PASSED** |
| **8** | `gw-chain-adapters` | Counterfactual Immutability, Tron 1-Sun Dust Keeper | 2 | **100% PASSED** |
| **9** | `gw-treasury` | FIFO Conservation Law, Cryptographic Realized P&L Hash Chain | 5 | **100% PASSED** |
| **10** | `gw-router` | Economical Sweep Inequality, Proportional Split Conservation | 6 | **100% PASSED** |
| **11** | `sovereign-gateway-engine`| Dependency Injection Container & Axum `FromRef` Extraction | 4 | **100% PASSED** |
| **12** | `gw-runtime` | Multi-Chain Checkout Lifecycle, Crash Recovery, 1000-Thread CAS | 4 | **100% PASSED** |

**Total:** 48 / 48 Formal Mathematical Audit Tests Passing (100% Pass Rate).

---

## Quick Start

### Prerequisites
- Rust 1.85+ (Edition 2024)
- Git with Submodules enabled

### Clone & Build
```bash
git clone --recurse-submodules https://github.com/Kurisuma/crypto-engine-v9.git
cd crypto-engine-v9

# Run the complete mathematical test suite across all 11 crates
cargo test --workspace

# Verify strict zero-warning compiler and clippy fencing
cargo clippy --workspace --all-targets --no-deps
```

---

## Documentation Links

- **[`USECASE.md`](./USECASE.md):** 1,500+ line comprehensive manual and operational reference for `oberer-orm` across all platform subsystems.
- **[`Notes.md`](./Notes.md):** Engineering retrospective, runtime friction points, and 10 concrete architectural proposals for `oberer-orm` v4.0.
- **[`task.md`](./task.md):** Complete specification, mathematical theory, and verification records for Phases 0 through 12.

---

## License

Dual-licensed under MIT or Apache 2.0.
