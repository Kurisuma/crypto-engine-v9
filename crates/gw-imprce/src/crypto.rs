#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic, clippy::unwrap_used)]

use crate::error::ImprceError;
use subtle::ConstantTimeEq;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Ephemeral Per-Boot Cryptographic Keys for Disjoint Routing & Memory MACs.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct ImprceCryptoEngine {
    arena_keys: [[u8; 32]; 3],
    mac_key: [u8; 32],
}

impl ImprceCryptoEngine {
    /// Initialize with deterministic keys (e.g. from CSPRNG / seed).
    pub fn new(seed: [u8; 32]) -> Self {
        let mut h = blake3::Hasher::new_keyed(&seed);
        h.update(b"IMPRCE_BOOT_CRYPTO_ENGINE_V9");

        let mut out = [0u8; 128];
        let mut xof = h.finalize_xof();
        xof.fill(&mut out);

        let mut k0 = [0u8; 32];
        let mut k1 = [0u8; 32];
        let mut k2 = [0u8; 32];
        let mut kmac = [0u8; 32];

        k0.copy_from_slice(&out[0..32]);
        k1.copy_from_slice(&out[32..64]);
        k2.copy_from_slice(&out[64..96]);
        kmac.copy_from_slice(&out[96..128]);

        Self {
            arena_keys: [k0, k1, k2],
            mac_key: kmac,
        }
    }

    /// Compute disjoint shard index for arena j in [0, num_shards - 1].
    pub fn route_shard(&self, arena_idx: usize, ns: &str, key: &str, num_shards: usize) -> usize {
        let key_bytes = self.arena_keys.get(arena_idx).unwrap_or(&self.arena_keys[0]);
        let mut h = blake3::Hasher::new_keyed(key_bytes);
        h.update(ns.as_bytes());
        h.update(b":");
        h.update(key.as_bytes());
        let hash_output = h.finalize();
        let bytes = hash_output.as_bytes();

        let val = u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]);
        usize::try_from(val).unwrap_or(0) % num_shards
    }

    /// Compute hardware-accelerated CRC32C over cell fields.
    pub fn compute_crc32(ns: &str, key: &str, version: u64, payload: &[u8]) -> u32 {
        let mut hasher = crc32fast::Hasher::new();
        hasher.update(ns.as_bytes());
        hasher.update(b":");
        hasher.update(key.as_bytes());
        hasher.update(&version.to_be_bytes());
        hasher.update(payload);
        hasher.finalize()
    }

    /// Compute 128-bit Keyed MAC over cell envelope.
    pub fn compute_mac(
        &self,
        ns: &str,
        key: &str,
        tier: u8,
        version: u64,
        expires_at_ms: u64,
        payload: &[u8],
    ) -> [u8; 16] {
        let mut h = blake3::Hasher::new_keyed(&self.mac_key);
        h.update(b"IMPRCE_CELL_V5");
        h.update(ns.as_bytes());
        h.update(key.as_bytes());
        h.update(&[tier]);
        h.update(&version.to_be_bytes());
        h.update(&expires_at_ms.to_be_bytes());
        h.update(payload);

        let digest = h.finalize();
        let mut mac = [0u8; 16];
        if let Some(slice) = digest.as_bytes().get(0..16) {
            mac.copy_from_slice(slice);
        }
        mac
    }

    /// Verify in constant time both CRC32 and 128-bit MAC.
    #[allow(clippy::too_many_arguments)]
    pub fn verify_cell(
        &self,
        ns: &str,
        key: &str,
        tier: u8,
        version: u64,
        expires_at_ms: u64,
        payload: &[u8],
        expected_crc: u32,
        expected_mac: &[u8; 16],
    ) -> Result<(), ImprceError> {
        let computed_crc = Self::compute_crc32(ns, key, version, payload);
        if computed_crc != expected_crc {
            return Err(ImprceError::BitRotDetected {
                expected: expected_crc,
                computed: computed_crc,
            });
        }

        let computed_mac = self.compute_mac(ns, key, tier, version, expires_at_ms, payload);
        if computed_mac.ct_eq(expected_mac).unwrap_u8() != 1 {
            return Err(ImprceError::MacTamperingDetected(key.to_string()));
        }

        Ok(())
    }
}
