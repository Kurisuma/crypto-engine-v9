#![forbid(unsafe_code)]
#![deny(
    clippy::float_arithmetic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::unwrap_used,
    clippy::indexing_slicing
)]

use gw_primitives::{ChainId, SovereignError};
use sha2::{Digest, Sha256};

pub const PDA_MARKER: &[u8] = b"ProgramDerivedAddress";
pub const SOLANA_RENT_EXEMPT_TOKEN_ACCOUNT_LAMPORTS: u64 = 2_039_280;

/// Field arithmetic for Ed25519 curve membership test: $\mathbb{F}_{2^{255}-19}$.
/// Checks whether a 32-byte candidate lies on Curve25519 / Ed25519 or is strictly off-curve (PDA).
mod ed25519_field {
    use alloy_primitives::U256;

    // p = 2^255 - 19
    const P_WORDS: [u64; 4] = [
        0xffffffffffffffed,
        0xffffffffffffffff,
        0xffffffffffffffff,
        0x7fffffffffffffff,
    ];

    fn p_u256() -> U256 {
        U256::from_limbs(P_WORDS)
    }

    // d = -121665 / 121666 mod (2^255 - 19)
    // 0x52036cee2b6ffe738cc740797779e89800700a4d4141d8ab75eb4dca135978a3
    const D_WORDS: [u64; 4] = [
        0x75eb4dca135978a3,
        0x00700a4d4141d8ab,
        0x8cc740797779e898,
        0x52036cee2b6ffe73,
    ];

    fn d_u256() -> U256 {
        U256::from_limbs(D_WORDS)
    }

    fn add_mod(a: U256, b: U256, p: U256) -> U256 {
        let (sum, carry) = a.overflowing_add(b);
        if carry || sum >= p {
            sum.wrapping_sub(p)
        } else {
            sum
        }
    }

    fn sub_mod(a: U256, b: U256, p: U256) -> U256 {
        if a >= b {
            a.wrapping_sub(b)
        } else {
            p.wrapping_sub(b.wrapping_sub(a))
        }
    }

    fn mul_mod(a: U256, b: U256, p: U256) -> U256 {
        a.mul_mod(b, p)
    }

    fn pow_mod(mut base: U256, mut exp: U256, p: U256) -> U256 {
        let mut res = U256::from(1);
        while exp > U256::ZERO {
            if (exp & U256::from(1)) != U256::ZERO {
                res = mul_mod(res, base, p);
            }
            base = mul_mod(base, base, p);
            exp >>= 1;
        }
        res
    }

    /// Determines if a 32-byte sequence represents a valid point on Ed25519.
    /// In Ed25519, the equation is $-x^2 + y^2 = 1 + d x^2 y^2 \iff x^2 = (y^2 - 1) / (d y^2 + 1)$.
    /// A candidate $y \in [0, 2^{255}-19)$ is on the curve iff $(y^2 - 1) / (d y^2 + 1)$ is a quadratic residue in $\mathbb{F}_p$.
    pub fn is_on_curve(bytes: &[u8; 32]) -> bool {
        let mut limbs = [0u64; 4];
        for (i, limb) in limbs.iter_mut().enumerate() {
            let offset = i.saturating_mul(8);
            if let Some(chunk_bytes) = bytes.get(offset..offset.saturating_add(8)) {
                let mut arr = [0u8; 8];
                arr.copy_from_slice(chunk_bytes);
                *limb = u64::from_le_bytes(arr);
            }
        }

        // Mask off the top sign bit (x-sign)
        if let Some(last) = limbs.get_mut(3) {
            *last &= 0x7fffffffffffffff;
        }
        let y = U256::from_limbs(limbs);
        let p = p_u256();

        if y >= p {
            return false;
        }

        let y2 = mul_mod(y, y, p);
        let u = sub_mod(y2, U256::from(1), p); // u = y^2 - 1
        let v = add_mod(mul_mod(d_u256(), y2, p), U256::from(1), p); // v = d*y^2 + 1

        if v == U256::ZERO {
            return false;
        }

        // u / v = u * v^(p-2) mod p
        let v_inv = pow_mod(v, p.wrapping_sub(U256::from(2)), p);
        let val = mul_mod(u, v_inv, p);

        if val == U256::ZERO {
            return true;
        }

        // Euler's criterion: val^((p-1)/2) == 1 mod p
        let exp = p.wrapping_sub(U256::from(1)) >> 1;
        let euler = pow_mod(val, exp, p);
        euler == U256::from(1)
    }
}

/// Derives a deterministic ephemeral reference public key $R_i \in \mathbb{E}_{\text{Ed25519}}$
/// for Direct Mode zero-sweep checkouts.
pub fn derive_ephemeral_reference(
    master_seed: &[u8; 32],
    invoice_id: i64,
) -> (String, [u8; 32]) {
    let mut hasher = Sha256::new();
    hasher.update(b"SOLANA_DIRECT_MODE_EPHEMERAL_REFERENCE_V9");
    hasher.update(master_seed);
    hasher.update(invoice_id.to_le_bytes());
    let mut digest = [0u8; 32];
    digest.copy_from_slice(&hasher.finalize());

    // Format as Base58 address
    let b58 = bs58::encode(digest).into_string();
    (b58, digest)
}

/// Computes a Program Derived Address (PDA) with canonical bump $b^* \in [0, 255]$
/// asserting that the resulting address is strictly off-curve:
///
/// $$A_{\text{PDA}}(s, b^*) \notin \mathbb{E}_{\text{Ed25519}}$$
pub fn find_program_address(
    seeds: &[&[u8]],
    program_id: &[u8; 32],
) -> Result<([u8; 32], u8), SovereignError> {
    for bump in (0..=255).rev() {
        let mut hasher = Sha256::new();
        for seed in seeds {
            hasher.update(seed);
        }
        hasher.update([bump]);
        hasher.update(program_id);
        hasher.update(PDA_MARKER);
        let digest = hasher.finalize();

        let mut candidate = [0u8; 32];
        candidate.copy_from_slice(&digest);

        // Must NOT be on the Ed25519 curve
        if !ed25519_field::is_on_curve(&candidate) {
            return Ok((candidate, bump));
        }
    }

    Err(SovereignError::InvalidBlockchainAddress {
        chain: ChainId::SolanaMainnet,
        address: "Failed to find valid off-curve PDA bump".to_string(),
    })
}

/// Derives a CEX Withdrawal Mode counterfactual PDA address for a given invoice.
pub fn derive_solana_invoice_pda(
    program_id: &[u8; 32],
    invoice_id: i64,
) -> Result<(String, [u8; 32], u8), SovereignError> {
    let id_bytes = invoice_id.to_le_bytes();
    let seeds = [b"deposit".as_slice(), id_bytes.as_slice()];
    let (pda_bytes, bump) = find_program_address(&seeds, program_id)?;
    let b58 = bs58::encode(pda_bytes).into_string();
    Ok((b58, pda_bytes, bump))
}

/// Validates that a string is a valid Solana 32-byte Base58 address.
pub fn validate_solana_address(address: &str) -> Result<[u8; 32], SovereignError> {
    let decoded = bs58::decode(address).into_vec().map_err(|e| {
        SovereignError::InvalidBlockchainAddress {
            chain: ChainId::SolanaMainnet,
            address: format!("Solana Base58 decode error: {e}"),
        }
    })?;

    if decoded.len() != 32 {
        return Err(SovereignError::InvalidBlockchainAddress {
            chain: ChainId::SolanaMainnet,
            address: format!("Solana address must be 32 bytes decoded, got {}", decoded.len()),
        });
    }

    let mut out = [0u8; 32];
    out.copy_from_slice(&decoded);
    Ok(out)
}

/// Verifies whether a 32-byte candidate point lies on the Ed25519 curve.
pub fn is_ed25519_curve_point(bytes: &[u8; 32]) -> bool {
    ed25519_field::is_on_curve(bytes)
}
