#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic, clippy::unwrap_used)]

use serde::{Deserialize, Serialize};

/// 5-Tier Priority Classification for In-Memory State.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PriorityTier {
    /// P0: Consensus headers, block ring buffer, active network config (k=3, non-evictable)
    P0CriticalConsensus = 0,
    /// P1: Active deposit watchlists, pending invoices (k=3, protected until grace expiry)
    P1ActivePaymentWatch = 1,
    /// P2: Rotating deposit address pool leases, cooldown locks (k=2, protected until cooldown)
    P2PoolLeaseCooldown = 2,
    /// P3: Oracle exchange rates, locked quotes (k=2, evictable on quote expiry)
    P3OracleQuoteLock = 3,
    /// P4: Telemetry, webhook idempotency, ephemeral cache (k=1, strictly sacrificial)
    P4TelemetryIdempotency = 4,
}

impl PriorityTier {
    /// Replication factor k(p) defining how many mirror arenas hold this entry.
    pub const fn replication_factor(&self) -> usize {
        match self {
            Self::P0CriticalConsensus => 3,
            Self::P1ActivePaymentWatch => 3,
            Self::P2PoolLeaseCooldown => 2,
            Self::P3OracleQuoteLock => 2,
            Self::P4TelemetryIdempotency => 1,
        }
    }

    /// Reserved memory floor in basis points (10,000 bps = 100%).
    pub const fn reserved_floor_bps(&self) -> u64 {
        match self {
            Self::P0CriticalConsensus => 1000,  // 10%
            Self::P1ActivePaymentWatch => 3500, // 35%
            Self::P2PoolLeaseCooldown => 2000,  // 20%
            Self::P3OracleQuoteLock => 1500,    // 15%
            Self::P4TelemetryIdempotency => 0,  // 0%
        }
    }

    /// Hard memory ceiling in basis points.
    pub const fn hard_ceiling_bps(&self) -> u64 {
        match self {
            Self::P0CriticalConsensus => 1500,  // 15%
            Self::P1ActivePaymentWatch => 5000, // 50%
            Self::P2PoolLeaseCooldown => 3000,  // 30%
            Self::P3OracleQuoteLock => 2500,    // 25%
            Self::P4TelemetryIdempotency => 1500, // 15%
        }
    }

    /// Weight ω(p) used in Retention Priority Score Φ calculation.
    pub const fn retention_weight(&self) -> u128 {
        match self {
            Self::P0CriticalConsensus => 1_000_000_000_000,
            Self::P1ActivePaymentWatch => 1_000_000_000,
            Self::P2PoolLeaseCooldown => 10_000_000,
            Self::P3OracleQuoteLock => 10_000,
            Self::P4TelemetryIdempotency => 1,
        }
    }

    /// Check if this entry can be evicted given current monotonic time in milliseconds.
    pub fn is_evictable(&self, now_ms: u64, expires_at_ms: u64) -> bool {
        match self {
            Self::P0CriticalConsensus => false, // Immortal
            Self::P1ActivePaymentWatch => now_ms > expires_at_ms.saturating_add(60_000), // 60s grace
            Self::P2PoolLeaseCooldown => now_ms > expires_at_ms,
            Self::P3OracleQuoteLock => now_ms > expires_at_ms,
            Self::P4TelemetryIdempotency => true, // Always eligible for sacrificial eviction
        }
    }
}

/// Computes the integer Retention Priority Score Φ(e, t_now) in Z_{2^128}.
/// Low scores are evicted first. Expired entries strictly evaluate to 0.
pub fn compute_retention_score(
    tier: PriorityTier,
    now_ms: u64,
    expires_at_ms: u64,
    hit_count: u64,
    byte_footprint: usize,
) -> u128 {
    if now_ms >= expires_at_ms {
        return 0; // Strictly prioritize expired items for eviction
    }

    let delta_rem_ms = (expires_at_ms.saturating_sub(now_ms)).max(1) as u128;
    let s = (byte_footprint as u128).max(64);
    let hits = (hit_count as u128).saturating_add(1);

    let weight_part = tier.retention_weight().saturating_mul(1_000_000_000);
    let recency_part = (hits.saturating_mul(delta_rem_ms)) / s;

    weight_part.saturating_add(recency_part)
}
