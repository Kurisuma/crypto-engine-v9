#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

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
