#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic, clippy::unwrap_used)]

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

    /// Performs a deep health check of the underlying native node and triggers automatic failover.
    async fn check_node_health_and_failover(
        &self,
    ) -> Result<AdapterDiagnosticSnapshot, SovereignError>;
}
