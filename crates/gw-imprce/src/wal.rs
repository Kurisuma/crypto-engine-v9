#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic, clippy::unwrap_used)]

use crate::error::ImprceError;
use crate::priority::PriorityTier;
use sha2::{Digest, Sha256};
use parking_lot::RwLock;

/// Individual immutable frame in the tamper-evident hash chain.
#[derive(Debug, Clone)]
pub struct WalFrame {
    pub lsn: u64,
    pub prev_hash: [u8; 32],
    pub frame_hash: [u8; 32],
    pub ns: String,
    pub key: String,
    pub priority: PriorityTier,
    pub version: u64,
    pub expires_at_ms: u64,
    pub payload: Vec<u8>,
    pub crc32: u32,
}

struct WalState {
    latest_lsn: u64,
    latest_hash: [u8; 32],
    frames: Vec<WalFrame>,
}

/// Cryptographically chained append-only Write-Ahead Log for P0, P1, P2 mutations.
pub struct HashChainedWal {
    state: RwLock<WalState>,
}

fn to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(s, "{:02x}", b);
    }
    s
}

impl HashChainedWal {
    pub fn new() -> Self {
        Self {
            state: RwLock::new(WalState {
                latest_lsn: 0,
                latest_hash: [0u8; 32], // Genesis hash H_0 = 0x00...00
                frames: Vec::new(),
            }),
        }
    }

    /// Compute frame hash: H_n = SHA256(H_{n-1} || LSN_n || ns || key || p || v || exp || payload || crc32).
    #[allow(clippy::too_many_arguments)]
    fn compute_frame_hash(
        prev_hash: &[u8; 32],
        lsn: u64,
        ns: &str,
        key: &str,
        priority: PriorityTier,
        version: u64,
        expires_at_ms: u64,
        payload: &[u8],
        crc32: u32,
    ) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(prev_hash);
        hasher.update(lsn.to_be_bytes());
        hasher.update(ns.as_bytes());
        hasher.update(b":");
        hasher.update(key.as_bytes());
        hasher.update([priority as u8]);
        hasher.update(version.to_be_bytes());
        hasher.update(expires_at_ms.to_be_bytes());
        hasher.update(payload);
        hasher.update(crc32.to_be_bytes());

        let out = hasher.finalize();
        let mut frame_hash = [0u8; 32];
        frame_hash.copy_from_slice(&out);
        frame_hash
    }

    /// Append a mutation frame to the hash chain.
    #[allow(clippy::too_many_arguments)]
    pub fn append(
        &self,
        ns: &str,
        key: &str,
        priority: PriorityTier,
        version: u64,
        expires_at_ms: u64,
        payload: &[u8],
        crc32: u32,
    ) -> [u8; 32] {
        let mut state = self.state.write();
        let lsn = state.latest_lsn.saturating_add(1);
        let prev_hash = state.latest_hash;

        let frame_hash = Self::compute_frame_hash(
            &prev_hash,
            lsn,
            ns,
            key,
            priority,
            version,
            expires_at_ms,
            payload,
            crc32,
        );

        state.latest_lsn = lsn;
        state.latest_hash = frame_hash;
        state.frames.push(WalFrame {
            lsn,
            prev_hash,
            frame_hash,
            ns: ns.to_string(),
            key: key.to_string(),
            priority,
            version,
            expires_at_ms,
            payload: payload.to_vec(),
            crc32,
        });

        frame_hash
    }

    /// Verify unbroken cryptographic continuity of the WAL chain.
    pub fn verify_continuity(&self) -> Result<(), ImprceError> {
        let state = self.state.read();
        let mut expected_prev = [0u8; 32];

        for frame in &state.frames {
            if frame.prev_hash != expected_prev {
                return Err(ImprceError::WalHashChainBroken {
                    expected: to_hex(&expected_prev),
                    computed: to_hex(&frame.prev_hash),
                });
            }

            let expected_current = Self::compute_frame_hash(
                &expected_prev,
                frame.lsn,
                &frame.ns,
                &frame.key,
                frame.priority,
                frame.version,
                frame.expires_at_ms,
                &frame.payload,
                frame.crc32,
            );

            if frame.frame_hash != expected_current {
                return Err(ImprceError::WalHashChainBroken {
                    expected: to_hex(&expected_current),
                    computed: to_hex(&frame.frame_hash),
                });
            }

            expected_prev = frame.frame_hash;
        }

        Ok(())
    }

    /// Get total frame count.
    pub fn frame_count(&self) -> usize {
        self.state.read().frames.len()
    }
}

impl Default for HashChainedWal {
    fn default() -> Self {
        Self::new()
    }
}
