#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic, clippy::unwrap_used)]

use crate::cell::SecureCacheCell;
use crate::priority::PriorityTier;
use parking_lot::RwLock;
use std::collections::HashMap;

pub const NUM_SHARDS: usize = 64;

#[derive(Default)]
pub struct ShardInner {
    pub entries: HashMap<String, SecureCacheCell>,
    pub byte_usage: usize,
}

pub struct Shard {
    pub inner: RwLock<ShardInner>,
}

impl Shard {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(ShardInner::default()),
        }
    }
}

impl Default for Shard {
    fn default() -> Self {
        Self::new()
    }
}

/// A single disjoint mirror arena sharded into 64 fine-grained concurrent buckets.
pub struct MirrorArena {
    pub arena_index: usize,
    pub shards: Vec<Shard>,
}

impl MirrorArena {
    pub fn new(arena_index: usize) -> Self {
        let mut shards = Vec::with_capacity(NUM_SHARDS);
        for _ in 0..NUM_SHARDS {
            shards.push(Shard::new());
        }
        Self {
            arena_index,
            shards,
        }
    }

    /// Insert or update a cell in the given shard.
    pub fn insert(&self, shard_idx: usize, cell: SecureCacheCell) -> Option<SecureCacheCell> {
        let shard = self.shards.get(shard_idx)?;
        let mut lock = shard.inner.write();

        let composite_key = format!("{}:{}", cell.ns, cell.key);
        let cell_bytes = cell.byte_footprint();

        if let Some(old) = lock.entries.insert(composite_key, cell) {
            lock.byte_usage = lock.byte_usage.saturating_sub(old.byte_footprint());
            lock.byte_usage = lock.byte_usage.saturating_add(cell_bytes);
            Some(old)
        } else {
            lock.byte_usage = lock.byte_usage.saturating_add(cell_bytes);
            None
        }
    }

    /// Lookup a cell clone from the given shard without verification.
    pub fn get(&self, shard_idx: usize, ns: &str, key: &str) -> Option<SecureCacheCell> {
        let shard = self.shards.get(shard_idx)?;
        let mut lock = shard.inner.write();

        let composite_key = format!("{ns}:{key}");
        if let Some(cell) = lock.entries.get_mut(&composite_key) {
            cell.hit_count = cell.hit_count.saturating_add(1);
            Some(cell.clone())
        } else {
            None
        }
    }

    /// Remove a cell from the shard and return it (caller will zeroize).
    pub fn remove(&self, shard_idx: usize, ns: &str, key: &str) -> Option<SecureCacheCell> {
        let shard = self.shards.get(shard_idx)?;
        let mut lock = shard.inner.write();

        let composite_key = format!("{ns}:{key}");
        if let Some(removed) = lock.entries.remove(&composite_key) {
            lock.byte_usage = lock.byte_usage.saturating_sub(removed.byte_footprint());
            Some(removed)
        } else {
            None
        }
    }

    /// Total bytes currently stored across all shards in this arena.
    pub fn total_byte_usage(&self) -> usize {
        let mut total: usize = 0;
        for shard in &self.shards {
            let lock = shard.inner.read();
            total = total.saturating_add(lock.byte_usage);
        }
        total
    }

    /// Total number of entries currently stored across all shards.
    pub fn count_entries(&self) -> usize {
        let mut count: usize = 0;
        for shard in &self.shards {
            let lock = shard.inner.read();
            count = count.saturating_add(lock.entries.len());
        }
        count
    }

    /// Evict the entry with lowest retention priority score Φ in a target shard,
    /// constrained to priority tiers at or below `max_evictable_tier`.
    pub fn evict_lowest_phi(
        &self,
        shard_idx: usize,
        now_ms: u64,
        max_evictable_tier: PriorityTier,
    ) -> Option<SecureCacheCell> {
        let shard = self.shards.get(shard_idx)?;
        let mut lock = shard.inner.write();

        let mut candidate_key: Option<String> = None;
        let mut min_score = u128::MAX;

        for (k, cell) in &lock.entries {
            if cell.priority >= max_evictable_tier && cell.priority.is_evictable(now_ms, cell.expires_at_ms) {
                let score = cell.retention_score(now_ms);
                if score < min_score {
                    min_score = score;
                    candidate_key = Some(k.clone());
                }
            }
        }

        if let Some(key) = candidate_key {
            if let Some(evicted) = lock.entries.remove(&key) {
                lock.byte_usage = lock.byte_usage.saturating_sub(evicted.byte_footprint());
                return Some(evicted);
            }
        }

        None
    }

    /// Purge all expired entries across all shards.
    pub fn purge_expired(&self, now_ms: u64) -> usize {
        let mut purged: usize = 0;
        for shard in &self.shards {
            let mut lock = shard.inner.write();
            let mut to_remove = Vec::new();

            for (k, cell) in &lock.entries {
                if cell.is_expired(now_ms) {
                    to_remove.push(k.clone());
                }
            }

            for k in to_remove {
                if let Some(cell) = lock.entries.remove(&k) {
                    lock.byte_usage = lock.byte_usage.saturating_sub(cell.byte_footprint());
                    purged = purged.saturating_add(1);
                }
            }
        }
        purged
    }
}
