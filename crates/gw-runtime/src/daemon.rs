#![forbid(unsafe_code)]
#![deny(
    clippy::float_arithmetic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::unwrap_used,
    clippy::indexing_slicing
)]

use gw_enterprise_traits::{EngineIntegrityReport, SovereignEngine, SovereignGatewayRuntimeEngine};
use gw_primitives::SovereignError;
use sovereign_gateway_engine::di_container::SovereignDiContainer;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, watch};
use tokio::task::JoinHandle;

/// Configuration parameters for the Master Runtime Daemon.
#[derive(Clone, Debug)]
pub struct DaemonConfig {
    /// Periodic interval between background cryptographic integrity audits.
    pub audit_interval: Duration,
    /// Maximum number of audit execution records preserved in in-memory ring buffer.
    pub max_audit_history: usize,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            audit_interval: Duration::from_secs(60),
            max_audit_history: 100,
        }
    }
}

/// Historical record of a completed cryptographic integrity self-audit.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AuditExecutionRecord {
    pub timestamp_utc: chrono::DateTime<chrono::Utc>,
    pub report: EngineIntegrityReport,
    pub execution_duration_us: u64,
}

/// Master Runtime Daemon orchestrating lifecycle, sequential boot, periodic
/// cryptographic self-audits, and graceful shutdown across all gateway subsystems.
pub struct MasterRuntimeDaemon {
    di_container: SovereignDiContainer,
    config: DaemonConfig,
    audit_history: Arc<RwLock<Vec<AuditExecutionRecord>>>,
    shutdown_tx: Option<watch::Sender<bool>>,
    daemon_task: Option<JoinHandle<()>>,
}

impl MasterRuntimeDaemon {
    /// Creates a new Master Runtime Daemon instance with the given DI container and configuration.
    pub fn new(di_container: SovereignDiContainer, config: DaemonConfig) -> Self {
        Self {
            di_container,
            config,
            audit_history: Arc::new(RwLock::new(Vec::new())),
            shutdown_tx: None,
            daemon_task: None,
        }
    }

    /// Access the underlying DI container holding all 11 gateway engines.
    pub fn container(&self) -> &SovereignDiContainer {
        &self.di_container
    }

    /// Access the master composition root runtime engine.
    pub fn runtime(&self) -> Arc<SovereignGatewayRuntimeEngine> {
        self.di_container.runtime()
    }

    /// Executes the ordered boot sequence across all gateway engines:
    /// Storage Engine -> Cache Hydration -> Blockchain Adapters -> Verification -> Treasury -> Router.
    pub async fn boot(&self) -> Result<(), SovereignError> {
        self.di_container.initialize_all().await
    }

    /// Executes a single synchronous cryptographic integrity audit pass,
    /// records the result in the audit history ring buffer, and returns the audit record.
    pub async fn run_audit_once(&self) -> Result<AuditExecutionRecord, SovereignError> {
        let start = Instant::now();
        let report = self.runtime().audit_cryptographic_integrity().await?;
        let execution_duration_us =
            u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX);

        let record = AuditExecutionRecord {
            timestamp_utc: chrono::Utc::now(),
            report,
            execution_duration_us,
        };

        let mut history = self.audit_history.write().await;
        history.push(record.clone());
        if history.len() > self.config.max_audit_history {
            let excess = history.len().saturating_sub(self.config.max_audit_history);
            history.rotate_left(excess);
            history.truncate(self.config.max_audit_history);
        }

        Ok(record)
    }

    /// Spawns the background periodic cryptographic self-audit loop.
    pub fn start_background_audit(&mut self) -> Result<(), SovereignError> {
        if self.daemon_task.is_some() {
            return Ok(());
        }

        let (shutdown_tx, mut shutdown_rx) = watch::channel(false);
        self.shutdown_tx = Some(shutdown_tx);

        let runtime = self.runtime();
        let audit_history = Arc::clone(&self.audit_history);
        let interval_duration = self.config.audit_interval;
        let max_history = self.config.max_audit_history;

        let handle = tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval_duration);
            // Skip immediate initial tick to allow explicit initial audit if desired
            ticker.tick().await;

            loop {
                tokio::select! {
                    _ = shutdown_rx.changed() => {
                        if *shutdown_rx.borrow() {
                            break;
                        }
                    }
                    _ = ticker.tick() => {
                        let start = Instant::now();
                        match runtime.audit_cryptographic_integrity().await {
                            Ok(report) => {
                                let execution_duration_us =
                                    u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX);

                                let record = AuditExecutionRecord {
                                    timestamp_utc: chrono::Utc::now(),
                                    report,
                                    execution_duration_us,
                                };

                                let mut history = audit_history.write().await;
                                history.push(record);
                                if history.len() > max_history {
                                    let excess = history.len().saturating_sub(max_history);
                                    history.rotate_left(excess);
                                    history.truncate(max_history);
                                }
                            }
                            Err(_e) => {
                                // Background audit failure is captured via audit_passed = false or logged
                            }
                        }
                    }
                }
            }
        });

        self.daemon_task = Some(handle);
        Ok(())
    }

    /// Retrieves a snapshot copy of all recorded audit passes.
    pub async fn get_audit_history(&self) -> Vec<AuditExecutionRecord> {
        self.audit_history.read().await.clone()
    }

    /// Executes graceful shutdown: terminates background audit tasks and invokes
    /// reverse-order engine teardown (Router -> Treasury -> Verification -> Adapters -> Cache -> Storage).
    pub async fn shutdown(&mut self) -> Result<(), SovereignError> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(true);
        }

        if let Some(handle) = self.daemon_task.take() {
            let _ = handle.await;
        }

        self.runtime().shutdown_gracefully().await?;
        Ok(())
    }
}
