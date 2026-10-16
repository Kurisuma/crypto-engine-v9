#![forbid(unsafe_code)]
#![deny(
    clippy::float_arithmetic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::unwrap_used,
    clippy::indexing_slicing
)]

use gw_primitives::{ChainArchitecture, ChainId, SovereignError};
use std::collections::BTreeMap;
use std::sync::RwLock;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Volatile zeroized RAM buffer for 32-byte asymmetric cryptographic private keys.
/// Enforces Axiom 4: Secret data is scrubbed with a volatile write barrier on drop.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct ZeroizedKeyBuffer {
    bytes: [u8; 32],
}

impl ZeroizedKeyBuffer {
    pub fn new(secret: [u8; 32]) -> Self {
        Self { bytes: secret }
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.bytes
    }
}

/// Key store mapping key locator strings to zeroized private keys inside Zone C.
pub struct EnclaveKeyStore {
    secp_keys: RwLock<BTreeMap<String, ZeroizedKeyBuffer>>,
    ed_keys: RwLock<BTreeMap<String, ZeroizedKeyBuffer>>,
}

impl Default for EnclaveKeyStore {
    fn default() -> Self {
        Self::new()
    }
}

impl EnclaveKeyStore {
    pub fn new() -> Self {
        Self {
            secp_keys: RwLock::new(BTreeMap::new()),
            ed_keys: RwLock::new(BTreeMap::new()),
        }
    }

    /// Registers a 32-byte secp256k1 private key for a key locator URI.
    pub fn insert_secp256k1_key(
        &self,
        locator: &str,
        secret: [u8; 32],
    ) -> Result<(), SovereignError> {
        let mut guard = self.secp_keys.write().map_err(|_| {
            SovereignError::EnclavePolicyViolation {
                reason: "Enclave secp_keys lock poisoned".to_string(),
            }
        })?;
        guard.insert(locator.to_string(), ZeroizedKeyBuffer::new(secret));
        Ok(())
    }

    /// Registers a 32-byte Ed25519 private key seed for a key locator URI.
    pub fn insert_ed25519_key(
        &self,
        locator: &str,
        secret: [u8; 32],
    ) -> Result<(), SovereignError> {
        let mut guard = self.ed_keys.write().map_err(|_| {
            SovereignError::EnclavePolicyViolation {
                reason: "Enclave ed_keys lock poisoned".to_string(),
            }
        })?;
        guard.insert(locator.to_string(), ZeroizedKeyBuffer::new(secret));
        Ok(())
    }

    /// Derives and exports the watch-only public descriptor (compressed public key hex).
    pub fn export_public_descriptor(
        &self,
        chain_id: ChainId,
        locator: &str,
    ) -> Result<String, SovereignError> {
        let arch = chain_id.architecture();
        match arch {
            ChainArchitecture::EvmAccount
            | ChainArchitecture::BitcoinUtxo
            | ChainArchitecture::TvmAccount => {
                let guard = self.secp_keys.read().map_err(|_| {
                    SovereignError::EnclavePolicyViolation {
                        reason: "Enclave secp_keys lock poisoned".to_string(),
                    }
                })?;
                let key_buf = guard.get(locator).ok_or_else(|| {
                    SovereignError::EnclavePolicyViolation {
                        reason: format!("Key locator '{locator}' not found in secp256k1 store"),
                    }
                })?;

                let signing_key = k256::ecdsa::SigningKey::from_bytes(key_buf.as_bytes().into())
                    .map_err(|e| SovereignError::EnclavePolicyViolation {
                        reason: format!("Invalid secp256k1 scalar in enclave: {e}"),
                    })?;
                let pub_point = signing_key.verifying_key().to_encoded_point(true);
                Ok(hex_encode(pub_point.as_bytes()))
            }
            ChainArchitecture::SvmAccount
            | ChainArchitecture::TonActor
            | ChainArchitecture::MoveObject => {
                let guard = self.ed_keys.read().map_err(|_| {
                    SovereignError::EnclavePolicyViolation {
                        reason: "Enclave ed_keys lock poisoned".to_string(),
                    }
                })?;
                let key_buf = guard.get(locator).ok_or_else(|| {
                    SovereignError::EnclavePolicyViolation {
                        reason: format!("Key locator '{locator}' not found in ed25519 store"),
                    }
                })?;

                let signing_key = ed25519_dalek::SigningKey::from_bytes(key_buf.as_bytes());
                let verifying_key = signing_key.verifying_key();
                Ok(hex_encode(verifying_key.as_bytes()))
            }
        }
    }

    /// Signs a 32-byte payload digest with the key referenced by locator.
    /// RAM is immediately zeroized when the temporary signing key leaves scope.
    pub fn sign_digest(
        &self,
        chain_id: ChainId,
        locator: &str,
        digest: &[u8; 32],
    ) -> Result<(Vec<u8>, String), SovereignError> {
        let arch = chain_id.architecture();
        match arch {
            ChainArchitecture::EvmAccount => {
                let guard = self.secp_keys.read().map_err(|_| {
                    SovereignError::EnclavePolicyViolation {
                        reason: "Enclave secp_keys lock poisoned".to_string(),
                    }
                })?;
                let key_buf = guard.get(locator).ok_or_else(|| {
                    SovereignError::EnclavePolicyViolation {
                        reason: format!("Key locator '{locator}' not found in secp256k1 store"),
                    }
                })?;

                let signing_key = k256::ecdsa::SigningKey::from_bytes(key_buf.as_bytes().into())
                    .map_err(|e| SovereignError::EnclavePolicyViolation {
                        reason: format!("Invalid secp256k1 scalar: {e}"),
                    })?;
                let (sig, recid) = signing_key
                    .sign_prehash_recoverable(digest)
                    .map_err(|e| SovereignError::EnclavePolicyViolation {
                        reason: format!("Secp256k1 signing failed: {e}"),
                    })?;

                let mut sig_bytes = Vec::with_capacity(65);
                sig_bytes.extend_from_slice(&sig.to_bytes());
                sig_bytes.push(recid.to_byte());

                let tx_hash = format!("0x{}", hex_encode(digest));
                Ok((sig_bytes, tx_hash))
            }
            ChainArchitecture::BitcoinUtxo | ChainArchitecture::TvmAccount => {
                let guard = self.secp_keys.read().map_err(|_| {
                    SovereignError::EnclavePolicyViolation {
                        reason: "Enclave secp_keys lock poisoned".to_string(),
                    }
                })?;
                let key_buf = guard.get(locator).ok_or_else(|| {
                    SovereignError::EnclavePolicyViolation {
                        reason: format!("Key locator '{locator}' not found in secp256k1 store"),
                    }
                })?;

                let signing_key = k256::ecdsa::SigningKey::from_bytes(key_buf.as_bytes().into())
                    .map_err(|e| SovereignError::EnclavePolicyViolation {
                        reason: format!("Invalid secp256k1 scalar: {e}"),
                    })?;
                let (sig, _) = signing_key
                    .sign_prehash_recoverable(digest)
                    .map_err(|e| SovereignError::EnclavePolicyViolation {
                        reason: format!("Secp256k1 signing failed: {e}"),
                    })?;

                let sig_bytes = sig.to_bytes().to_vec();
                let tx_hash = format!("0x{}", hex_encode(digest));
                Ok((sig_bytes, tx_hash))
            }
            ChainArchitecture::SvmAccount
            | ChainArchitecture::TonActor
            | ChainArchitecture::MoveObject => {
                let guard = self.ed_keys.read().map_err(|_| {
                    SovereignError::EnclavePolicyViolation {
                        reason: "Enclave ed_keys lock poisoned".to_string(),
                    }
                })?;
                let key_buf = guard.get(locator).ok_or_else(|| {
                    SovereignError::EnclavePolicyViolation {
                        reason: format!("Key locator '{locator}' not found in ed25519 store"),
                    }
                })?;

                let signing_key = ed25519_dalek::SigningKey::from_bytes(key_buf.as_bytes());
                use ed25519_dalek::Signer;
                let sig = signing_key.sign(digest);

                let sig_bytes = sig.to_bytes().to_vec();
                let tx_hash = format!("0x{}", hex_encode(digest));
                Ok((sig_bytes, tx_hash))
            }
        }
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len().saturating_mul(2));
    for &b in bytes {
        let hi = (b >> 4) as usize;
        let lo = (b & 0x0f) as usize;
        if let Some(&c) = HEX.get(hi) {
            s.push(c as char);
        }
        if let Some(&c) = HEX.get(lo) {
            s.push(c as char);
        }
    }
    s
}
