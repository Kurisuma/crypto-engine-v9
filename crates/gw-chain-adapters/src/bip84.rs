#![forbid(unsafe_code)]
#![deny(
    clippy::float_arithmetic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::unwrap_used,
    clippy::indexing_slicing
)]

use bech32::Hrp;
use gw_primitives::SovereignError;
use hmac::{Hmac, Mac};
use k256::elliptic_curve::group::prime::PrimeCurveAffine;
use k256::elliptic_curve::sec1::{FromEncodedPoint, ToEncodedPoint};
use k256::elliptic_curve::PrimeField;
use k256::{AffinePoint, EncodedPoint, ProjectivePoint, Scalar};
use ripemd::Ripemd160;
use sha2::{Digest, Sha256, Sha512};

type HmacSha512 = Hmac<Sha512>;

/// Watch-Only BIP-84 Non-Hardened Public Child Deriver.
/// Operates strictly within Zone A (zero private keys).
#[derive(Debug, Clone)]
pub struct Bip84WatchOnlyDeriver {
    pub compressed_pubkey: [u8; 33],
    pub chaincode: [u8; 32],
    pub hrp: String,
}

impl Bip84WatchOnlyDeriver {
    pub fn new(compressed_pubkey: [u8; 33], chaincode: [u8; 32], hrp: impl Into<String>) -> Result<Self, SovereignError> {
        let hrp = hrp.into();
        // Verify public key is a valid point on secp256k1
        let encoded = EncodedPoint::from_bytes(compressed_pubkey).map_err(|e| {
            SovereignError::InvalidBlockchainAddress {
                chain: gw_primitives::ChainId::BitcoinMainnet,
                address: format!("Invalid SEC1 pubkey bytes: {e}"),
            }
        })?;
        let affine_opt: Option<AffinePoint> = AffinePoint::from_encoded_point(&encoded).into();
        if affine_opt.is_none() {
            return Err(SovereignError::InvalidBlockchainAddress {
                chain: gw_primitives::ChainId::BitcoinMainnet,
                address: "Public key is not on secp256k1 curve".to_string(),
            });
        }

        Ok(Self {
            compressed_pubkey,
            chaincode,
            hrp,
        })
    }

    /// Derives child public key and chaincode at non-hardened index $i \in [0, 2^{31}-1]$.
    ///
    /// $$I = \text{HMAC-SHA512}\left(c_{\text{ext}}, \; \text{Ser}_{33}(K_{\text{ext}}) \parallel \text{Ser}_{32}(i)\right)$$
    /// $$K_i = \text{parse}_{256}(I[0..32]) \cdot G + K_{\text{ext}}$$
    pub fn derive_child_pubkey(&self, index: u32) -> Result<([u8; 33], [u8; 32]), SovereignError> {
        // Enforce non-hardened index
        if index >= 0x80000000 {
            return Err(SovereignError::InvalidBlockchainAddress {
                chain: gw_primitives::ChainId::BitcoinMainnet,
                address: format!("Hardened derivation index {index:#x} forbidden in watch-only mode"),
            });
        }

        let mut mac = HmacSha512::new_from_slice(&self.chaincode).map_err(|e| {
            SovereignError::InvalidBlockchainAddress {
                chain: gw_primitives::ChainId::BitcoinMainnet,
                address: format!("HMAC initialization failed: {e}"),
            }
        })?;

        mac.update(&self.compressed_pubkey);
        mac.update(&index.to_be_bytes());
        let i_result = mac.finalize().into_bytes();

        let mut i_l = [0u8; 32];
        let mut i_r = [0u8; 32];
        if let Some(src_l) = i_result.get(0..32) {
            i_l.copy_from_slice(src_l);
        }
        if let Some(src_r) = i_result.get(32..64) {
            i_r.copy_from_slice(src_r);
        }

        // Parse I_L as scalar s
        let scalar_opt = Scalar::from_repr(i_l.into());
        let scalar: Scalar = if bool::from(scalar_opt.is_some()) {
            scalar_opt.unwrap_or(Scalar::ZERO)
        } else {
            return Err(SovereignError::InvalidBlockchainAddress {
                chain: gw_primitives::ChainId::BitcoinMainnet,
                address: "Derived scalar exceeds secp256k1 order".to_string(),
            });
        };

        if scalar == Scalar::ZERO {
            return Err(SovereignError::InvalidBlockchainAddress {
                chain: gw_primitives::ChainId::BitcoinMainnet,
                address: "Derived scalar is zero".to_string(),
            });
        }

        // Parent point P_parent
        let encoded_parent = EncodedPoint::from_bytes(self.compressed_pubkey).map_err(|e| {
            SovereignError::InvalidBlockchainAddress {
                chain: gw_primitives::ChainId::BitcoinMainnet,
                address: format!("Invalid parent SEC1 bytes: {e}"),
            }
        })?;
        let affine_parent: AffinePoint = AffinePoint::from_encoded_point(&encoded_parent)
            .into_option()
            .ok_or_else(|| SovereignError::InvalidBlockchainAddress {
                chain: gw_primitives::ChainId::BitcoinMainnet,
                address: "Invalid parent point".to_string(),
            })?;
        let proj_parent = ProjectivePoint::from(affine_parent);

        // K_i = s * G + K_parent
        let proj_child = ProjectivePoint::GENERATOR * scalar + proj_parent;
        let affine_child = proj_child.to_affine();

        if bool::from(affine_child.is_identity()) {
            return Err(SovereignError::InvalidBlockchainAddress {
                chain: gw_primitives::ChainId::BitcoinMainnet,
                address: "Derived point is identity at infinity".to_string(),
            });
        }

        let encoded_child = affine_child.to_encoded_point(true);
        let child_bytes = encoded_child.as_bytes();
        let mut child_pubkey = [0u8; 33];
        if child_bytes.len() == 33 {
            child_pubkey.copy_from_slice(child_bytes);
        } else {
            return Err(SovereignError::InvalidBlockchainAddress {
                chain: gw_primitives::ChainId::BitcoinMainnet,
                address: "Unexpected encoded child point length".to_string(),
            });
        }

        Ok((child_pubkey, i_r))
    }

    /// Derives the Bech32 Native SegWit (v0 P2WPKH) Bitcoin address for index $i$.
    ///
    /// $$\text{Addr}_{\text{BTC}}(i) = \text{Bech32Encode}\left(\text{"bc"}, \; 0, \; \text{RIPEMD160}(\text{SHA256}(\text{Ser}_{33}(K_i)))\right)$$
    pub fn derive_child_address(&self, index: u32) -> Result<String, SovereignError> {
        let (child_pubkey, _) = self.derive_child_pubkey(index)?;

        // Hash160 = RIPEMD160(SHA256(pubkey))
        let sha = Sha256::digest(child_pubkey);
        let hash160 = Ripemd160::digest(sha);

        let hrp = Hrp::parse(&self.hrp).map_err(|e| {
            SovereignError::InvalidBlockchainAddress {
                chain: gw_primitives::ChainId::BitcoinMainnet,
                address: format!("Invalid Bech32 HRP '{}': {e}", self.hrp),
            }
        })?;

        bech32::segwit::encode(hrp, bech32::segwit::VERSION_0, &hash160).map_err(|e| {
            SovereignError::InvalidBlockchainAddress {
                chain: gw_primitives::ChainId::BitcoinMainnet,
                address: format!("Bech32 encode error: {e}"),
            }
        })
    }
}

/// Validates that a string is a well-formed Bech32 Native SegWit address with the expected HRP.
pub fn validate_bip84_address(address: &str, expected_hrp: &str) -> Result<[u8; 20], SovereignError> {
    let (hrp, version, program) = bech32::segwit::decode(address).map_err(|e| {
        SovereignError::InvalidBlockchainAddress {
            chain: gw_primitives::ChainId::BitcoinMainnet,
            address: format!("Bech32 decode error: {e}"),
        }
    })?;

    if hrp.as_str() != expected_hrp {
        return Err(SovereignError::InvalidBlockchainAddress {
            chain: gw_primitives::ChainId::BitcoinMainnet,
            address: format!("HRP mismatch: expected '{expected_hrp}', got '{}'", hrp.as_str()),
        });
    }

    if version != bech32::segwit::VERSION_0 {
        return Err(SovereignError::InvalidBlockchainAddress {
            chain: gw_primitives::ChainId::BitcoinMainnet,
            address: format!("Witness version mismatch: expected 0, got {version:?}"),
        });
    }

    if program.len() != 20 {
        return Err(SovereignError::InvalidBlockchainAddress {
            chain: gw_primitives::ChainId::BitcoinMainnet,
            address: format!("Witness program length mismatch: expected 20 bytes, got {}", program.len()),
        });
    }

    let mut out = [0u8; 20];
    out.copy_from_slice(&program);
    Ok(out)
}
