#![forbid(unsafe_code)]

use oberer_orm_derive::OrmEnum;
use serde::{Deserialize, Serialize};

#[derive(OrmEnum, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum OrmChainArchitecture {
    BitcoinUtxo,
    EvmAccount,
    SvmAccount,
    TvmAccount,
    TonActor,
    MoveObject,
}

#[derive(OrmEnum, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum OrmLogicalAsset {
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

#[derive(OrmEnum, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PoolLeaseStatus {
    Available,
    Leased,
    Cooldown,
}

#[derive(OrmEnum, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum InvoiceLifecycleStatus {
    Pending,
    Detected,
    Partial,
    Completed,
    Overpaid,
    Expired,
}

#[derive(OrmEnum, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum OrmWalletTier {
    Tier0CustomerDeposit,
    Tier1GasRelayer,
    Tier2OperationalHot,
    Tier3ColdVault,
    Tier4MarketExchange,
}

#[derive(OrmEnum, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum TransferGovernanceStatus {
    PendingQuorum,
    SignedAuthorized,
    Broadcasted,
    ConfirmedFinal,
    RejectedPolicy,
}

#[derive(OrmEnum, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PnlDisposalEventKind {
    ColdVaultConsolidation,
    MarketExchangeDisposal,
    GasRelayerBurn,
}
