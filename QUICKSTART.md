# Sovereign Crypto Gateway & Unified Treasury Engine v9.0
# Quick Start & Operational Integration Guide

Welcome to the **Sovereign Crypto Gateway & Unified Treasury Engine v9.0** (`crypto-engine-v9`). This guide provides a step-by-step, production-ready walkthrough for configuring wallets, managing cryptographic keys across multi-zone security boundaries, issuing payment invoices, executing 5-gate deterministic settlements, and performing automated cold storage sweeps.

---

## 1. Architectural Mental Model & Security Zones

The Gateway enforces **Axiom 1 (Zero-Trust Key Segregation)** across three physical and logical security zones:

```
+-----------------------------------------------------------------------------------+
| ZONE A: UNTRUSTED INGESTION (Edge & Blockchain Network Layer)                     |
| - Chain Adapters (BTC, ETH, SOL, TON, SUI, TRON)                                  |
| - WebSocket / RPC Blockchain Scanners                                             |
| - IMPRCE In-Memory Price Rate Engine (Binance, Kraken, Coinbase, OKX)             |
| * STRICT ZERO PRIVATE KEYS ALLOWED *                                              |
+-----------------------------------------------------------------------------------+
                                        | (Read-Only Observation & Rate Quotes)
                                        v
+-----------------------------------------------------------------------------------+
| ZONE B: SOVEREIGN CORE LOGIC & STORAGE (Database & Business Orchestration)        |
| - Storage Engine: `gw-storage-oberer` (SQLite WAL + Blake3 Row MACs)              |
| - Settlement Engine: `gw-engine` (5-Gate Pipeline & DAG Continuity)                |
| - Treasury Engine: `gw-treasury` (Deterministic FIFO Cost-Basis Lots)             |
| - Liquidity Router: `gw-router` (Cross-Network Sweep Profitability)               |
| - DI Container & Axum REST API: `sovereign-gateway-engine`                        |
| * ONLY PUBLIC KEYS & DERIVATION XPUBs RESIDE HERE *                               |
+-----------------------------------------------------------------------------------+
                                        | (Unsigned Raw Transaction Payloads)
                                        v
+-----------------------------------------------------------------------------------+
| ZONE C: ISOLATED HARDWARE ENCLAVE & KMS (Signing & Key Storage)                   |
| - Enclave Signer: `gw-kms-enclave` (Ed25519, Secp256k1, BIP-84/44 Key Storage)  |
| - Policy Guard Engine: Pinned Cold Vault Whitelisting & Velocity Limiters        |
| - Dual-Hardware Security Token Multi-Sig Rotation                                 |
| * THE ONLY ZONE PERMITTED TO TOUCH PRIVATE KEYS OR SEEDS *                        |
+-----------------------------------------------------------------------------------+
```

---

## 2. Prerequisites & Workspace Setup

### 2.1. Toolchain Requirements

Ensure the host environment satisfies the sovereign compilation standards:
- **Rust Toolchain:** Version `1.85.0+` (Edition `2024` required).
- **SQLite Engine:** Version `3.45.0+` compiled with WAL and memory pragma support.
- **Hardware Architecture:** `x86_64` or `aarch64` with hardware AES / Blake3 instruction extensions.

### 2.2. Clone and Initialize Repository

```bash
git clone --recurse-submodules https://github.com/Kurisuma/crypto-engine-v9.git
cd crypto-engine-v9

# If cloned without submodules, initialize oberer-lite:
git submodule update --init --recursive
```

### 2.3. Workspace Verification

Run the comprehensive mathematical and invariant audit suite:

```bash
cargo test --workspace
```

Verify that all 48 mathematical audit tests pass with 0 failures:
- `gw-storage-oberer`: 4 tests (Blake3 HMAC, migration triggers, view aggregations)
- `gw-imprce`: 4 tests (VWAP freshness, sub-millisecond locks, zero-allocation ring buffer)
- `gw-chain-adapters`: 8 tests (BIP-84, CREATE2, Solana, TON, Sui derivation)
- `gw-engine`: 8 tests (5-Gate deterministic pipeline, DAG continuity)
- `gw-kms-enclave`: 4 tests (Policy engine, AST parsing, zero-leakage drop)
- `gw-treasury`: 4 tests (FIFO cost-basis, lot disposal, balance conservation)
- `gw-router`: 4 tests (Fee profitability evaluation, multi-tier routing)
- `gw-runtime`: 4 tests (Continuous 60s audit, daemon recovery, reorg rollback)

---

## 3. Configuration Setup (`config.toml` & Environment)

Create `config.toml` in your working directory:

```toml
[server]
listen_addr = "127.0.0.1:8080"
metrics_port = 9090
log_level = "info"

[database]
path = "sovereign_gateway.db"
busy_timeout_ms = 5000
cache_size_kb = 64000
wal_autocheckpoint_pages = 1000

[enclave]
mode = "simulated_sgx" # "simulated_sgx", "aws_nitro", or "hardware_pkcs11"
enclave_ipc_socket = "/var/run/sovereign-enclave.sock"
mac_secret_key_hex = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"

[networks.bitcoin]
enabled = true
network_type = "mainnet" # "mainnet", "testnet4", or "regtest"
rpc_endpoint = "http://127.0.0.1:8332"
rpc_user = "sovereign_rpc"
rpc_password = "super_secret_rpc_password_99"
min_confirmations = 3
dust_threshold_satoshis = 546

[networks.ethereum]
enabled = true
chain_id = 1
rpc_endpoint = "https://eth-mainnet.alchemyapi.io/v2/YOUR_API_KEY"
create2_factory_address = "0x0000000000ffe8b47b3e2130213b802212439497"
init_code_hash_hex = "9bd1c140c83a743aaec1982b5faaeec0b4e341595180f688849b2efebdbdb7b3"
min_confirmations = 12

[networks.solana]
enabled = true
cluster = "mainnet-beta"
rpc_endpoint = "https://api.mainnet-beta.solana.com"
commitment = "finalized"

[networks.tron]
enabled = true
rpc_endpoint = "https://api.trongrid.io"
usdt_contract_address = "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t"
min_confirmations = 19

[treasury]
tier0_max_hot_btc = 25000000      # 0.25 BTC max hot reserve
tier1_sweep_batch_btc = 100000000  # 1.0 BTC triggers automated cold sweep
cold_vault_btc = "bc1q9v0z6uk42wvyu2qjmn759f23v0a24qndpyl8k6"
cold_vault_eth = "0x89205A3A3b2A69De6Dbf7f01ED13B2108B2c43e7"
cold_vault_sol = "ColdVault11111111111111111111111111111111111"

[imprce]
quote_ttl_seconds = 600
stale_threshold_seconds = 30
active_sources = ["Binance", "Kraken", "Coinbase", "Okx"]
```

Set environment secrets:

```bash
export SOVEREIGN_ENCLAVE_SECRET_HEX="0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
export SOVEREIGN_DB_PATH="sovereign_gateway.db"
```

---

## 4. Key Management: Generation, Derivation & Injection

> [!CAUTION]
> **Strict Rule of Separation:** Raw seed phrases (mnemonic), `xprv`, and private key hex strings must NEVER enter Zone A or Zone B code, logs, or databases. Zone B only receives public keys, `xpub`, and address salts.

```
+-----------------------------------------------------------------------------------+
| STEP 1: Cold Offline Generation                                                   |
| Entropy (256-bit) -> BIP-39 (24 words) -> Master Seed (512-bit)                   |
+-----------------------------------------------------------------------------------+
                                  |
                                  v
+-----------------------------------------------------------------------------------+
| STEP 2: Enclave Key Provisioning (Zone C)                                         |
| Ingest Master Seed -> Store in Secure RAM -> Zeroize Temporary Memory Buffers     |
+-----------------------------------------------------------------------------------+
                                  |
                                  v
+-----------------------------------------------------------------------------------+
| STEP 3: Gateway Public Parameter Export (Zone B)                                  |
| Export BIP-84 `xpub` (BTC) | Export Factory Address & Bytecode Hash (EVM)         |
| Export Ed25519 Root Pubkey (SOL, TON, SUI)                                        |
+-----------------------------------------------------------------------------------+
```

### 4.1. Bitcoin Native SegWit (BIP-84) Keys

Bitcoin payments use BIP-84 Native SegWit addresses starting with `bc1q...`.

#### Offline Seed & Key Derivation (CLI / Enclave Tool)
To generate the BIP-84 master key pair:
1. Root Derivation Path: `m/84'/0'/0'` (Account Level)
2. Private Extended Key (`xprv` / `zprv`): Injected directly into the Hardware Enclave (Zone C).
3. Public Extended Key (`xpub` / `zpub`): Exported to Zone B Gateway storage for invoice address derivation.

```bash
# Example derivation representation
Account Extended Private Key (Zone C ONLY):
zprvAdG4iTXWHiqnvq2CjH...

Account Extended Public Key (Exported to Zone B):
zpub6rFR7y4Q2AijBEqTUquhZz23456789abcdef...
```

#### How the Gateway Uses `xpub` in Zone B:
Zone B derives a unique receiving address for every customer invoice by deriving unhardened index paths `m/0/{invoice_index}` without ever needing the private key:

```rust
use gw_chain_adapters::bip84::Bip84Derivator;

// Initialize derivator in Zone B using ONLY the public key
let xpub = "zpub6rFR7y4Q2AijBEqTUquhZz23456789abcdef...";
let derivator = Bip84Derivator::from_xpub(xpub)?;

// Derive payment address for invoice index #42
let invoice_address = derivator.derive_address_at_index(42)?;
println!("Customer Payment Address: {}", invoice_address);
// Output: bc1q...
```

### 4.2. EVM Deterministic Counterfactual Addresses (CREATE2)

To accept ETH, USDT (ERC-20), or USDC without pre-funding gas for thousands of addresses, the Gateway uses counterfactual **CREATE2** factory derivation:

$$\text{Address} = \text{keccak256}(0\text{xff} \mathbin{\Vert} \text{FactoryAddress} \mathbin{\Vert} \text{Salt} \mathbin{\Vert} \text{keccak256}(\text{InitCode}))[12..32]$$

#### Factory & Parameter Setup
- **Factory Contract Address:** Pinned deployer contract on Ethereum/Polygon/Arbitrum.
- **Bytecode Hash:** Fixed keccak256 hash of the deposit forwarder contract.
- **Salt:** Generated deterministically from `Blake3(invoice_id || customer_id)`.

```rust
use gw_chain_adapters::create2::Create2AddressDerivator;

let factory_address = "0x0000000000ffe8b47b3e2130213b802212439497";
let init_code_hash = hex::decode("9bd1c140c83a743aaec1982b5faaeec0b4e341595180f688849b2efebdbdb7b3")?;

let derivator = Create2AddressDerivator::new(factory_address, init_code_hash)?;

// Calculate receiving address for invoice "INV-2026-009"
let invoice_salt = blake3::hash(b"INV-2026-009").as_bytes();
let payment_address = derivator.compute_address(invoice_salt)?;
println!("Counterfactual EVM Address: {}", payment_address);
// Output: 0x71C...
```

### 4.3. Solana, Sui & TON (Ed25519) Derivation

For Ed25519 networks:
- **Solana:** Path `m/44'/501'/0'/0'`
- **Sui:** Path `m/44'/784'/0'/0'/0'`
- **TON:** Path `m/44'/607'/0'` (Deriving workchain 0 bounceable/non-bounceable addresses)

In Zone B, only child public keys are derived and cached for invoice matching.

---

## 5. Configuring & Adding Wallets in the Gateway

The Gateway organizes treasury assets into three distinct operational tiers:

| Tier Level | Name | Purpose | Private Key Location |
| :--- | :--- | :--- | :--- |
| **Tier 0** | Hot Reservoir | Immediate merchant payouts & gas reserves | Zone C Enclave (Restricted limit) |
| **Tier 1** | Warm Aggregator | Intermediate sweep consolidation | Zone C Enclave (Multi-sig policy) |
| **Tier 2** | Cold Storage Vault | Secure offline cold storage | Hardware Vault / Air-gapped (Out-of-band) |

### 5.1. Registering Wallets Programmatically

Using the type-safe `ObererStorageBackend` (Zero Raw SQL):

```rust
use gw_storage_oberer::prelude::*;
use gw_primitives::asset::AssetId;
use std::sync::Arc;

pub async fn configure_gateway_wallets(
    storage: &Arc<ObererStorageBackend>,
) -> Result<(), StorageError> {
    // 1. Register Tier 0 Hot BIP-84 Wallet for Bitcoin
    storage.insert_wallet(&WalletRecord {
        wallet_id: "BTC-HOT-01".to_string(),
        asset_id: AssetId::Bitcoin,
        tier: WalletTier::Tier0Hot,
        derivation_xpub: Some("zpub6rFR7y4Q2AijBEqTUquh...".to_string()),
        pinned_address: None,
        max_balance_atomic: 50_000_000, // 0.5 BTC max
        current_balance_atomic: 0,
        status: WalletStatus::Active,
    }).await?;

    // 2. Register Tier 2 Offline Cold Storage Pinned Address for Bitcoin
    storage.insert_wallet(&WalletRecord {
        wallet_id: "BTC-COLD-VAULT".to_string(),
        asset_id: AssetId::Bitcoin,
        tier: WalletTier::Tier2Cold,
        derivation_xpub: None,
        pinned_address: Some("bc1q9v0z6uk42wvyu2qjmn759f23v0a24qndpyl8k6".to_string()),
        max_balance_atomic: u64::MAX,
        current_balance_atomic: 0,
        status: WalletStatus::Active,
    }).await?;

    // 3. Register Tier 2 Cold Storage Pinned Address for Ethereum
    storage.insert_wallet(&WalletRecord {
        wallet_id: "ETH-COLD-VAULT".to_string(),
        asset_id: AssetId::Ethereum,
        tier: WalletTier::Tier2Cold,
        derivation_xpub: None,
        pinned_address: Some("0x89205A3A3b2A69De6Dbf7f01ED13B2108B2c43e7".to_string()),
        max_balance_atomic: u64::MAX,
        current_balance_atomic: 0,
        status: WalletStatus::Active,
    }).await?;

    tracing::info!("Gateway multi-tier treasury wallets initialized successfully.");
    Ok(())
}
```

---

## 6. Invoice Lifecycle: Creation, Pricing & Address Derivation

```
Customer Checkout ($150 USD)
         |
         v
[IMPRCE Rate Engine] -----> Fetches Multi-Exchange VWAP -> Locks Quote (e.g. 0.00300000 BTC)
         |
         v
[Address Derivator]  -----> Derives Fresh Address: bc1q... / 0x...
         |
         v
[Oberer SQLite WAL]  -----> Inserts Invoice with Blake3 HMAC Signature
         |
         v
[Invoice Response]   -----> Returns Payment URI + QR Code + 10-Minute Expiry Window
```

### 6.1. Step-by-Step Code Recipe: Creating an Invoice

```rust
use sovereign_gateway_engine::prelude::*;
use rust_decimal_macros::dec;

pub async fn handle_create_invoice_request(
    di: &SovereignGatewayContainer,
    customer_id: &str,
    fiat_amount: rust_decimal::Decimal,
    fiat_currency: FiatCurrency,
    target_asset: AssetId,
) -> Result<InvoiceResponse, EngineError> {
    // 1. Obtain sub-millisecond VWAP rate from IMPRCE engine
    let quote = di.imprce_engine().calculate_crypto_quote(
        target_asset,
        fiat_currency,
        fiat_amount,
        600, // 10 minute price guarantee window
    ).await?;

    // 2. Allocate fresh sequential BIP-84 address index or CREATE2 salt
    let next_index = di.storage().get_next_derivation_index(target_asset).await?;
    let deposit_address = match target_asset {
        AssetId::Bitcoin => {
            di.bip84_engine().derive_address(next_index)?
        },
        AssetId::Ethereum | AssetId::UsdtErc20 => {
            let salt = blake3::hash(format!("{}:{}", customer_id, next_index).as_bytes());
            di.create2_engine().compute_address(salt.as_bytes())?
        },
        _ => return Err(EngineError::UnsupportedAsset(target_asset)),
    };

    // 3. Build invoice entity
    let invoice = InvoiceRecord {
        invoice_id: format!("INV-{}", uuid::Uuid::new_v4()),
        customer_id: customer_id.to_string(),
        asset_id: target_asset,
        fiat_currency,
        fiat_amount,
        crypto_amount_atomic: quote.crypto_amount_atomic,
        deposit_address: deposit_address.clone(),
        derivation_index: next_index,
        created_at: chrono::Utc::now().timestamp(),
        expires_at: chrono::Utc::now().timestamp() + 600,
        status: InvoiceStatus::Pending,
        tx_hash: None,
        confirmed_at: None,
    };

    // 4. Persist to SQLite with automatic Blake3 Row HMAC
    di.storage().insert_invoice(&invoice).await?;

    Ok(InvoiceResponse {
        invoice_id: invoice.invoice_id,
        deposit_address,
        crypto_amount_atomic: invoice.crypto_amount_atomic,
        formatted_crypto: format_atomic_units(target_asset, invoice.crypto_amount_atomic),
        expires_at: invoice.expires_at,
        qr_payload: generate_payment_uri(target_asset, &invoice.deposit_address, invoice.crypto_amount_atomic),
    })
}
```

---

## 7. Autonomous Ingestion & 5-Gate Deterministic Verification

When a transaction is broadcast to the network, the Gateway scanner intercepts it and passes it through the mathematical **5-Gate Verification Pipeline**:

```
Incoming Tx ──> [ Gate 1: Address & Payload Match ]
                     │ Passes
                     ▼
                [ Gate 2: Amount Exactness Check ]
                     │ Passes
                     ▼
                [ Gate 3: Block Reorg & DAG Continuity ]
                     │ Passes
                     ▼
                [ Gate 4: Finality Threshold Met ]
                     │ Passes
                     ▼
                [ Gate 5: Anti-Double-Spend & Time-Window Integrity ]
                     │
                     ▼ All 5 Gates Pass
             SETTLED & ALLOCATED TO TREASURY FIFO
```

### 7.1. Running the Ingestion Daemon

```rust
use gw_engine::prelude::*;
use gw_runtime::daemon::GatewayRuntimeDaemon;

pub async fn start_background_ingestion(di: &SovereignGatewayContainer) {
    let daemon = GatewayRuntimeDaemon::new(
        di.storage().clone(),
        di.verification_engine().clone(),
        di.chain_adapters().clone(),
        di.treasury_engine().clone(),
        di.router_engine().clone(),
        di.enclave_signer().clone(),
    );

    // Launch daemon in background tokio task
    tokio::spawn(async move {
        daemon.run_daemon_loop().await;
    });

    tracing::info!("Autonomous 5-Gate ingestion scanner running.");
}
```

### 7.2. Inspecting Gate Verification Results

```rust
let evaluation = di.verification_engine().evaluate_transaction(&tx_candidate).await?;

if evaluation.is_fully_confirmed() {
    println!("Transaction {} passed all 5 gates successfully!", tx_candidate.tx_hash);
    // Automatically triggers:
    // 1. Invoice status -> Settled
    // 2. Dispatch merchant webhook notification
    // 3. Register deposit lot in FIFO Treasury
} else {
    println!("Transaction pending: {:?}", evaluation.rejection_or_pending_reasons());
}
```

---

## 8. Outbound Transfers & Sweeping (How to Transfer Out)

When funds accumulate in hot receiving addresses or Tier 0 wallets, they must be swept into secure warm aggregators or Tier 2 cold vaults.

### 8.1. Sweep Economics & Viability Evaluation

The Liquidity Router enforces **Axiom 7 (Economically Rational Sweeping)**:

$$\text{Viability} = \text{Balance} > (\text{GasFee} \times \text{ThresholdMultiplier})$$

Never spend 50,000 satoshis in network fees to sweep a 60,000 satoshi balance!

```rust
use gw_router::prelude::*;

let balance = di.treasury_engine().get_tier0_balance(&AssetId::Bitcoin).await?;
let current_gas_fee = di.chain_adapters().bitcoin().estimate_current_fee().await?;

let is_profitable = di.router_engine().evaluate_sweep_viability(
    &AssetId::Bitcoin,
    balance,
    current_gas_fee,
)?;

if !is_profitable {
    tracing::info!("Sweep deferred: Network fee exceeds economic efficiency boundary.");
    return;
}
```

### 8.2. Formulating, Signing & Broadcasting an Outbound Sweep

The sweep process safely crosses the security boundary between Zone B and Zone C:

```
[Zone B: Router] ──> Constructs Unsigned Sweep Payload (Destination: Pinned Cold Vault)
                           │
                           ▼
[Zone C: Enclave] ─> Validates Destination Address against Pinned Whitelist
                  ─> Enforces Policy Guard Velocity Limits
                  ─> Hardware Signs Payload (ECDSA Secp256k1 / Ed25519)
                  ─> Returns Cryptographic Signature ONLY
                           │
                           ▼
[Zone A: Network] ─> Assembles Raw Tx & Broadcasts to Mempool
                           │
                           ▼
[Zone B: Treasury]─> Disposes FIFO Lots & Updates Ledger Balances
```

#### Executing the Sweep in Code:

```rust
pub async fn execute_cold_storage_sweep(
    di: &SovereignGatewayContainer,
    asset: AssetId,
) -> Result<SweepExecutionReport, EngineError> {
    // 1. Determine eligible funds in Tier 0 / Hot Deposit addresses
    let available_balance = di.treasury_engine().get_sweepable_balance(&asset).await?;
    if available_balance == 0 {
        return Err(EngineError::InsufficientFunds);
    }

    // 2. Retrieve strictly pinned Tier 2 Cold Vault Address
    let cold_vault_address = di.storage().get_pinned_cold_vault(&asset).await?
        .ok_or(EngineError::ColdVaultNotConfigured(asset))?;

    // 3. Build unsigned raw transaction in Zone B
    let unsigned_tx = di.router_engine().build_sweep_transaction(
        &asset,
        available_balance,
        &cold_vault_address,
    ).await?;

    // 4. Request signature from Zone C Enclave
    // Zone C validates destination against whitelist before signing!
    let signature = di.enclave_signer().sign_sweep_transaction(&unsigned_tx).await?;

    // 5. Broadcast signed transaction via Zone A network adapter
    let tx_hash = di.chain_adapters().get_adapter(asset)?
        .broadcast_raw_transaction(&unsigned_tx, &signature).await?;

    // 6. Dispose FIFO accounting lots in Treasury
    let disposed_lots = di.treasury_engine().dispose_fifo_lots(
        &asset,
        available_balance,
    ).await?;

    tracing::info!(
        tx_hash = %tx_hash,
        asset = ?asset,
        amount = available_balance,
        disposed_lot_count = disposed_lots.len(),
        "Cold storage sweep broadcasted successfully."
    );

    Ok(SweepExecutionReport {
        tx_hash,
        swept_amount: available_balance,
        destination: cold_vault_address,
        disposed_lots,
    })
}
```

---

## 9. Treasury Management & FIFO Cost-Basis Accounting

Every confirmed deposit creates an immutable **Treasury Lot** recorded in SQLite with its historical fiat acquisition exchange rate:

```
Deposit Lot #1: 100,000 satoshis @ $45,000 / BTC
Deposit Lot #2: 150,000 satoshis @ $48,000 / BTC
Deposit Lot #3:  50,000 satoshis @ $50,000 / BTC
Total Available: 300,000 satoshis
```

When a merchant withdrawal or sweep of **200,000 satoshis** occurs:
1. **100,000 satoshis** disposed from Lot #1 (Exhausted)
2. **100,000 satoshis** disposed from Lot #2 (50,000 satoshis remaining)
3. Lot #3 remains untouched.

### 9.1. Querying Portfolio Balances & Realized Gains

```rust
// Query aggregate treasury position across all tiers
let btc_portfolio = di.treasury_engine().get_portfolio_summary(AssetId::Bitcoin).await?;

println!("Bitcoin Treasury Status:");
println!("  Hot Reserve (Tier 0):  {} sats", btc_portfolio.tier0_hot_balance);
println!("  Warm Vault (Tier 1):   {} sats", btc_portfolio.tier1_warm_balance);
println!("  Cold Storage (Tier 2): {} sats", btc_portfolio.tier2_cold_balance);
println!("  Total Treasury Asset:  {} sats", btc_portfolio.total_balance());
println!("  Active Unspent Lots:   {}", btc_portfolio.active_lot_count);
```

---

## 10. Operations, Monitoring & Emergency Response

### 10.1. Running the Continuous 60-Second Cryptographic Audit

The system runs an autonomous background audit every 60 seconds verifying **Axiom 5 (Cryptographic Row MACs)** and **Axiom 6 (Mathematical Ledger Balancing)**:

```rust
let audit_report = di.runtime_engine().audit_cryptographic_integrity().await?;

if audit_report.detected_corruptions_or_tampers > 0 {
    eprintln!("CRITICAL ALERT: Database tampering detected! Mismatched HMAC rows found.");
    // Runtime daemon automatically engages emergency quarantine
} else {
    println!("Database integrity verified: 0 tampered rows, 0 balance leaks.");
}
```

### 10.2. Production SQLite Maintenance Commands

```bash
# Verify SQLite WAL integrity
sqlite3 sovereign_gateway.db "PRAGMA integrity_check;"

# Force checkpoint WAL log into primary database
sqlite3 sovereign_gateway.db "PRAGMA wal_checkpoint(TRUNCATE);"

# View active database tables
sqlite3 sovereign_gateway.db ".tables"
```

---

## 11. Axum REST API Integration

The Gateway provides pre-configured Axum HTTP routes for seamless integration into web services.

### 11.1. Starting the HTTP Server

```rust
use axum::{routing::{get, post}, Router};
use sovereign_gateway_engine::axum_ext::*;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Build the sovereign dependency injection container
    let container = SovereignGatewayBuilder::new()
        .with_sqlite_storage("sovereign_gateway.db").await?
        .with_imprce_price_feeds().await?
        .with_chain_adapters().await?
        .with_hardware_enclave().await?
        .build()?;

    // Register REST endpoints
    let app = Router::new()
        .route("/api/v1/invoices", post(create_invoice_handler))
        .route("/api/v1/invoices/:id", get(get_invoice_handler))
        .route("/api/v1/treasury/balances", get(get_treasury_balances_handler))
        .route("/api/v1/treasury/sweep", post(trigger_sweep_handler))
        .route("/api/v1/health", get(health_check_handler))
        .with_state(container);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    tracing::info!("Sovereign Gateway API listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
```

### 11.2. API Quick Reference

#### Create Invoice (`POST /api/v1/invoices`)
```bash
curl -X POST http://127.0.0.1:8080/api/v1/invoices \
  -H "Content-Type: application/json" \
  -d '{
    "customer_id": "cust_882910",
    "fiat_amount": "150.00",
    "fiat_currency": "USD",
    "asset_id": "Bitcoin"
  }'
```

Response:
```json
{
  "invoice_id": "INV-7a42f5eb-09bb-4bc3-9584-3c66b26cf5da",
  "deposit_address": "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh",
  "crypto_amount_atomic": 300000,
  "formatted_crypto": "0.00300000 BTC",
  "expires_at": 1776346200,
  "qr_payload": "bitcoin:bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh?amount=0.00300000"
}
```

#### Query Invoice Status (`GET /api/v1/invoices/:id`)
```bash
curl http://127.0.0.1:8080/api/v1/invoices/INV-7a42f5eb-09bb-4bc3-9584-3c66b26cf5da
```

Response:
```json
{
  "invoice_id": "INV-7a42f5eb-09bb-4bc3-9584-3c66b26cf5da",
  "status": "Settled",
  "expected_amount_atomic": 300000,
  "received_amount_atomic": 300000,
  "confirmations": 6,
  "tx_hash": "0x4b7891...b52a"
}
```

#### View Treasury Balances (`GET /api/v1/treasury/balances`)
```bash
curl http://127.0.0.1:8080/api/v1/treasury/balances
```

Response:
```json
{
  "bitcoin": {
    "tier0_hot_satoshis": 1500000,
    "tier1_warm_satoshis": 45000000,
    "tier2_cold_satoshis": 250000000,
    "total_satoshis": 296500000
  },
  "ethereum": {
    "tier0_hot_wei": "500000000000000000",
    "tier2_cold_wei": "15000000000000000000",
    "total_wei": "15500000000000000000"
  }
}
```

---

## 12. Complete Executable End-to-End Starter Script

Below is a complete, self-contained Rust program demonstrating the complete gateway workflow from initialization to cold vault sweep:

```rust
// File: src/bin/quickstart_demo.rs

use gw_storage_oberer::prelude::*;
use gw_imprce::prelude::*;
use gw_chain_adapters::prelude::*;
use gw_engine::prelude::*;
use gw_treasury::prelude::*;
use gw_router::prelude::*;
use gw_kms_enclave::prelude::*;
use gw_primitives::asset::AssetId;
use gw_primitives::fiat::FiatCurrency;
use rust_decimal_macros::dec;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Sovereign Crypto Gateway v9.0 Quickstart Demo ===");

    // Step 1: Initialize In-Memory SQLite Database with Oberer ORM
    let storage = Arc::new(ObererStorageBackend::new_in_memory().await?);
    storage.run_migrations().await?;
    println!("[1/6] Database initialized with SQLite WAL & Blake3 HMAC triggers.");

    // Step 2: Configure Wallets (BIP-84 Hot xpub + Cold Storage Pinned Address)
    storage.insert_wallet(&WalletRecord {
        wallet_id: "BTC-HOT".to_string(),
        asset_id: AssetId::Bitcoin,
        tier: WalletTier::Tier0Hot,
        derivation_xpub: Some("zpub6rFR7y4Q2AijBEqTUquh...".to_string()),
        pinned_address: None,
        max_balance_atomic: 100_000_000,
        current_balance_atomic: 0,
        status: WalletStatus::Active,
    }).await?;

    storage.insert_wallet(&WalletRecord {
        wallet_id: "BTC-COLD-VAULT".to_string(),
        asset_id: AssetId::Bitcoin,
        tier: WalletTier::Tier2Cold,
        derivation_xpub: None,
        pinned_address: Some("bc1q9v0z6uk42wvyu2qjmn759f23v0a24qndpyl8k6".to_string()),
        max_balance_atomic: u64::MAX,
        current_balance_atomic: 0,
        status: WalletStatus::Active,
    }).await?;
    println!("[2/6] Wallets configured: Tier 0 Hot + Tier 2 Cold Vault.");

    // Step 3: Initialize IMPRCE Rate Engine & Calculate Quote
    let imprce = Arc::new(MockImprceEngine::new(dec!(65_000.00))); // BTC = $65,000 USD
    let quote = imprce.calculate_crypto_quote(
        AssetId::Bitcoin,
        FiatCurrency::Usd,
        dec!(130.00), // $130.00 invoice
        600,
    ).await?;
    println!("[3/6] IMPRCE Quote: $130.00 USD = {} satoshis (0.002 BTC)", quote.crypto_amount_atomic);

    // Step 4: Issue Customer Invoice
    let deposit_address = "bc1qdemoaddress99887766554433221100223344";
    let invoice = InvoiceRecord {
        invoice_id: "INV-DEMO-001".to_string(),
        customer_id: "customer_42".to_string(),
        asset_id: AssetId::Bitcoin,
        fiat_currency: FiatCurrency::Usd,
        fiat_amount: dec!(130.00),
        crypto_amount_atomic: quote.crypto_amount_atomic,
        deposit_address: deposit_address.to_string(),
        derivation_index: 1,
        created_at: 1776345000,
        expires_at: 1776345600,
        status: InvoiceStatus::Pending,
        tx_hash: None,
        confirmed_at: None,
    };
    storage.insert_invoice(&invoice).await?;
    println!("[4/6] Customer invoice created with address: {}", deposit_address);

    // Step 5: Simulate Ingestion & 5-Gate Verification
    let incoming_tx = ChainTransaction {
        tx_hash: "0x112233445566778899aabbccddeeff00112233445566778899aabbccddeeff00".to_string(),
        network_id: 1,
        destination_address: deposit_address.to_string(),
        atomic_amount: quote.crypto_amount_atomic,
        block_height: 850_100,
        block_hash: "0x00000000000000000003a...".to_string(),
        parent_block_hash: "0x00000000000000000002b...".to_string(),
        confirmations: 6,
    };

    let verification_engine = FiveGateVerificationEngine::new(storage.clone());
    let evaluation = verification_engine.evaluate_transaction_gates(&incoming_tx, &invoice.invoice_id).await?;
    assert!(evaluation.passed_all_gates());
    storage.update_invoice_status(&invoice.invoice_id, InvoiceStatus::Settled).await?;
    println!("[5/6] Transaction passed all 5 gates! Invoice settled.");

    // Step 6: FIFO Treasury Ingestion & Cold Storage Sweep
    let treasury = FifoTreasuryEngine::new(storage.clone());
    treasury.record_deposit_lot(
        AssetId::Bitcoin,
        quote.crypto_amount_atomic,
        &incoming_tx.tx_hash,
        dec!(65_000.00),
    ).await?;

    let router = CrossNetworkRouter::new(treasury.clone());
    let enclave = MockIsolatedEnclave::new();

    let sweep_tx = router.build_sweep_transaction(
        &AssetId::Bitcoin,
        quote.crypto_amount_atomic,
        "bc1q9v0z6uk42wvyu2qjmn759f23v0a24qndpyl8k6",
    ).await?;

    let signature = enclave.sign_transaction(&sweep_tx.signing_payload).await?;
    assert!(!signature.is_empty());

    let disposed = treasury.dispose_fifo_lots(&AssetId::Bitcoin, quote.crypto_amount_atomic).await?;
    println!("[6/6] Automated Cold Storage Sweep Executed!");
    println!("      Swept: {} sats to Cold Vault", quote.crypto_amount_atomic);
    println!("      Disposed Lots: {}", disposed.lots.len());
    println!("=== Demo Completed Successfully ===");

    Ok(())
}
```

---

## 13. Production Checklist & Troubleshooting Runbook

### 13.1. Pre-Deployment Verification Matrix

- [ ] **Zero Key Infiltration:** Run `powershell ./scripts/verify_zero_key_isolation.ps1` to ensure no private key variables or crate imports exist in Zone A or Zone B.
- [ ] **Strict Compiler Denial:** Ensure `Cargo.toml` contains `#![forbid(unsafe_code)]` and strict clippy denial attributes (`clippy::unwrap_used`, `clippy::float_arithmetic`, `clippy::indexing_slicing`).
- [ ] **Enclave Whitelist Locked:** Confirm that all withdrawal destinations are strictly whitelisted to pinned cold vault addresses.
- [ ] **WAL Mode Active:** Verify that SQLite database reports `journal_mode = wal`.
- [ ] **Audit Tests Passing:** Verify `cargo test --workspace` executes 48 passing tests without failures.

### 13.2. Common Operational Issues & Remediation

| Issue | Root Cause | Remediation Step |
| :--- | :--- | :--- |
| **`InvoiceExpiredBeforePayment`** | Customer sent transaction after the 10-minute quote window. | Review transaction in mempool. If amount matches fiat value, execute manual override via administrative CLI or refund to customer. |
| **`Gate3ReorgDetected`** | Blockchain experienced a block reorg; parent hash continuity broke. | Gateway pauses network ingestion. Wait for canonical chain reorganization depth to stabilize, then re-evaluate. |
| **`MacTamperViolation`** | SQLite row was modified outside the Gateway (e.g. direct SQL edit). | Quarantine affected table. Restore from latest verified WAL snapshot. Identify unauthorized DB access. |
| **`EnclavePolicyRejection`** | Sweep transaction attempted to send funds to an unwhitelisted address. | Verify destination against pinned cold storage configuration. Enclave will strictly refuse to sign unknown addresses. |
| **`ImprceStaleFeed`** | Exchange WebSocket / REST disconnected for > 30 seconds. | Check network connectivity to Binance/Kraken. IMPRCE will reject rate calculations until fresh ticks are received. |

---

## 14. Merchant Webhook Notifications & Signature Verification

When a transaction successfully clears all 5 gates and settles an invoice, the Gateway dispatches an HTTP `POST` webhook to the merchant's configured endpoint.

### 14.1. Webhook Payload Schema

```json
{
  "event": "invoice.settled",
  "event_id": "evt_98234710293847",
  "timestamp": 1776345630,
  "data": {
    "invoice_id": "INV-DEMO-001",
    "customer_id": "customer_42",
    "asset": "BTC",
    "fiat_amount": "130.00",
    "fiat_currency": "USD",
    "crypto_amount_atomic": 200000,
    "crypto_amount_formatted": "0.00200000 BTC",
    "deposit_address": "bc1qdemoaddress99887766554433221100223344",
    "tx_hash": "0x112233445566778899aabbccddeeff00112233445566778899aabbccddeeff00",
    "confirmations": 6,
    "settled_at": 1776345630
  }
}
```

### 14.2. Cryptographic HMAC Signature Verification

All webhooks include an `X-Sovereign-Signature` header computed as:

$$\text{Signature} = \text{HMAC-SHA256}(\text{MerchantWebhookSecret}, \text{Timestamp} \mathbin{\Vert} \text{"."} \mathbin{\Vert} \text{RawBody})$$

#### Node.js / TypeScript Verification Example:

```typescript
import crypto from "crypto";

export function verifyWebhookSignature(
  rawBody: string,
  signatureHeader: string,
  timestampHeader: string,
  webhookSecret: string
): boolean {
  const payloadToSign = `${timestampHeader}.${rawBody}`;
  const expectedSignature = crypto
    .createHmac("sha256", webhookSecret)
    .update(payloadToSign)
    .digest("hex");

  return crypto.timingSafeEqual(
    Buffer.from(signatureHeader, "hex"),
    Buffer.from(expectedSignature, "hex")
  );
}
```

#### Python Verification Example:

```python
import hmac
import hashlib

def verify_sovereign_webhook(raw_body: bytes, signature: str, timestamp: str, secret: str) -> bool:
    message = f"{timestamp}.".encode("utf-8") + raw_body
    computed = hmac.new(secret.encode("utf-8"), message, hashlib.sha256).hexdigest()
    return hmac.compare_digest(signature, computed)
```

---

## 15. Gateway Administrative CLI Commands & Maintenance Reference

The Gateway ships with an administrative CLI binary for out-of-band operational tasks:

```bash
# Display general status and engine health snapshots
cargo run --bin gateway-admin -- health

# List all configured wallets and real-time ledger balances
cargo run --bin gateway-admin -- wallet list

# Register a new Tier 2 Cold Vault Address
cargo run --bin gateway-admin -- wallet add \
  --asset BTC \
  --tier cold \
  --address bc1q9v0z6uk42wvyu2qjmn759f23v0a24qndpyl8k6

# Trigger immediate cryptographic audit of all database tables
cargo run --bin gateway-admin -- audit run

# Force an immediate cold storage sweep for an asset
cargo run --bin gateway-admin -- sweep execute --asset BTC

# Check active FIFO treasury lots
cargo run --bin gateway-admin -- treasury lots --asset BTC
```

---

*Authored by the Sovereign Gateway & Treasury Engine Architecture Team.*  
*Document ID: `QUICKSTART-SOVEREIGN-CRYPTO-GATEWAY-V9`*
