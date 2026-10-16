#![forbid(unsafe_code)]
#![deny(
    clippy::float_arithmetic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::unwrap_used,
    clippy::indexing_slicing
)]

pub mod daemon;

pub use daemon::{AuditExecutionRecord, DaemonConfig, MasterRuntimeDaemon};

pub mod prelude {
    pub use crate::daemon::{AuditExecutionRecord, DaemonConfig, MasterRuntimeDaemon};
    pub use gw_enterprise_traits::{
        CrossNetworkLiquidityRouterEngine, EngineHealthSnapshot, EngineIntegrityReport,
        InvoiceVerificationEngine, ObererOrmStorageEngine, PriorityRedundantCacheEngine,
        SovereignEngine, SovereignGatewayRuntimeEngine, TreasuryPortfolioEngine,
        UnifiedNetworkAdapterEngine,
    };
    pub use gw_primitives::*;
    pub use sovereign_gateway_engine::prelude::*;
}
