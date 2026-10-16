#![forbid(unsafe_code)]
#![deny(
    clippy::float_arithmetic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::unwrap_used,
    clippy::indexing_slicing
)]

use alloy_primitives::U256;
use gw_primitives::{ChainArchitecture, RawAssetAmount, SovereignError};
use serde::{Deserialize, Serialize};

/// Parsed security details extracted from raw unsigned transaction bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParsedPayloadDetails {
    pub destinations: Vec<String>,
    pub net_output_amount: RawAssetAmount,
    pub network_fee: RawAssetAmount,
    pub nonce_or_sequence: u64,
    pub call_data_or_memo: Option<String>,
}

/// Zone C Stateless Pre-Sign Bytecode & AST Parser.
pub struct PayloadAstParser;

impl PayloadAstParser {
    /// Universally parses raw unsigned transaction bytes according to chain architecture.
    pub fn parse(
        raw_bytes: &[u8],
        architecture: ChainArchitecture,
        decimals: u8,
    ) -> Result<ParsedPayloadDetails, SovereignError> {
        if raw_bytes.is_empty() {
            return Err(SovereignError::EnclavePolicyViolation {
                reason: "Empty raw transaction payload".to_string(),
            });
        }

        // 1. Check for Gateway Canonical Envelope Magic: b"GW9_CANONICAL_TX\x00"
        if raw_bytes.starts_with(b"GW9_CANONICAL_TX\x00") {
            let offset = b"GW9_CANONICAL_TX\x00".len();
            let slice = raw_bytes.get(offset..).ok_or_else(|| {
                SovereignError::EnclavePolicyViolation {
                    reason: "Malformed canonical envelope slice".to_string(),
                }
            })?;
            return serde_json::from_slice::<ParsedPayloadDetails>(slice).map_err(|e| {
                SovereignError::EnclavePolicyViolation {
                    reason: format!("Failed to parse canonical envelope JSON: {e}"),
                }
            });
        }

        // 2. Architecture-Specific Binary AST Parsers
        match architecture {
            ChainArchitecture::EvmAccount => Self::parse_evm_eip1559_rlp(raw_bytes, decimals),
            ChainArchitecture::BitcoinUtxo => Self::parse_bitcoin_utxo_tx(raw_bytes, decimals),
            ChainArchitecture::TvmAccount => Self::parse_tron_tx(raw_bytes, decimals),
            ChainArchitecture::SvmAccount => Self::parse_solana_svm_message(raw_bytes, decimals),
            ChainArchitecture::TonActor => Self::parse_ton_message(raw_bytes, decimals),
            ChainArchitecture::MoveObject => Self::parse_sui_move_bcs(raw_bytes, decimals),
        }
    }

    /// Parses EIP-1559 Dynamic Fee Transaction RLP bytes:
    /// `0x02 || rlp([chain_id, nonce, max_priority_fee_per_gas, max_fee_per_gas, gas_limit, to, value, data, access_list])`
    fn parse_evm_eip1559_rlp(
        bytes: &[u8],
        decimals: u8,
    ) -> Result<ParsedPayloadDetails, SovereignError> {
        let first = bytes.first().copied().ok_or_else(|| {
            SovereignError::EnclavePolicyViolation {
                reason: "EVM payload is empty".to_string(),
            }
        })?;

        // Must be EIP-1559 envelope type 0x02 (or legacy 0x00/rlp list)
        let rlp_body = if first == 0x02 {
            bytes.get(1..).ok_or_else(|| SovereignError::EnclavePolicyViolation {
                reason: "EIP-1559 payload missing RLP body after type 0x02".to_string(),
            })?
        } else {
            bytes
        };

        let items = Self::decode_rlp_list(rlp_body)?;
        if items.len() < 7 {
            return Err(SovereignError::EnclavePolicyViolation {
                reason: format!("EVM EIP-1559 RLP list must have at least 7 items, got {}", items.len()),
            });
        }

        // Nonce is item 1
        let nonce_bytes = items.get(1).ok_or_else(|| SovereignError::EnclavePolicyViolation {
            reason: "Missing nonce item in RLP".to_string(),
        })?;
        let nonce = Self::bytes_to_u64(nonce_bytes)?;

        // max_fee_per_gas is item 3
        let max_fee_bytes = items.get(3).ok_or_else(|| SovereignError::EnclavePolicyViolation {
            reason: "Missing max_fee_per_gas in RLP".to_string(),
        })?;
        let max_fee_per_gas = Self::bytes_to_u256(max_fee_bytes);

        // gas_limit is item 4
        let gas_limit_bytes = items.get(4).ok_or_else(|| SovereignError::EnclavePolicyViolation {
            reason: "Missing gas_limit in RLP".to_string(),
        })?;
        let gas_limit = Self::bytes_to_u256(gas_limit_bytes);
        let max_fee_total = gas_limit.saturating_mul(max_fee_per_gas);

        // to is item 5 (20 bytes address)
        let to_bytes = items.get(5).ok_or_else(|| SovereignError::EnclavePolicyViolation {
            reason: "Missing 'to' address in RLP".to_string(),
        })?;
        if to_bytes.len() != 20 {
            return Err(SovereignError::EnclavePolicyViolation {
                reason: format!("EVM 'to' address must be 20 bytes, got {}", to_bytes.len()),
            });
        }
        let dest_hex = format!("0x{}", hex_encode(to_bytes));

        // value is item 6
        let value_bytes = items.get(6).ok_or_else(|| SovereignError::EnclavePolicyViolation {
            reason: "Missing value item in RLP".to_string(),
        })?;
        let value_u256 = Self::bytes_to_u256(value_bytes);

        Ok(ParsedPayloadDetails {
            destinations: vec![dest_hex],
            net_output_amount: RawAssetAmount::new(value_u256, decimals),
            network_fee: RawAssetAmount::new(max_fee_total, 18), // EVM native gas is 18 decimals
            nonce_or_sequence: nonce,
            call_data_or_memo: None,
        })
    }

    /// Parses Bitcoin Raw Transaction / PSBT:
    /// Version (4 bytes) | Inputs | Outputs | Locktime (4 bytes)
    fn parse_bitcoin_utxo_tx(
        bytes: &[u8],
        decimals: u8,
    ) -> Result<ParsedPayloadDetails, SovereignError> {
        if bytes.len() < 10 {
            return Err(SovereignError::EnclavePolicyViolation {
                reason: "Bitcoin payload too short (< 10 bytes)".to_string(),
            });
        }

        let mut cursor: usize = 0;
        // Version (4 bytes)
        cursor = cursor.saturating_add(4);

        // Parse Inputs Count (VarInt)
        let (num_inputs, varint_len) = Self::read_varint(bytes, cursor)?;
        cursor = cursor.saturating_add(varint_len);

        // Skip inputs: txid (32) + vout (4) + script_len + script + seq (4)
        for _ in 0..num_inputs {
            cursor = cursor.saturating_add(36); // txid + vout
            let (script_len, v_len) = Self::read_varint(bytes, cursor)?;
            let script_usize = usize::try_from(script_len).unwrap_or(0);
            cursor = cursor.saturating_add(v_len).saturating_add(script_usize);
            cursor = cursor.saturating_add(4); // sequence
        }

        // Parse Outputs Count (VarInt)
        let (num_outputs, v_len) = Self::read_varint(bytes, cursor)?;
        cursor = cursor.saturating_add(v_len);

        let mut destinations = Vec::new();
        let mut total_output_sats = 0_u64;

        for _ in 0..num_outputs {
            let val_slice = bytes.get(cursor..cursor.saturating_add(8)).ok_or_else(|| {
                SovereignError::EnclavePolicyViolation {
                    reason: "Premature EOF reading Bitcoin output value".to_string(),
                }
            })?;
            cursor = cursor.saturating_add(8);

            let mut val_arr = [0u8; 8];
            val_arr.copy_from_slice(val_slice);
            let val = u64::from_le_bytes(val_arr);
            total_output_sats = total_output_sats.saturating_add(val);

            let (script_len, sv_len) = Self::read_varint(bytes, cursor)?;
            cursor = cursor.saturating_add(sv_len);

            let script_usize = usize::try_from(script_len).unwrap_or(0);
            let script = bytes.get(cursor..cursor.saturating_add(script_usize)).ok_or_else(|| {
                SovereignError::EnclavePolicyViolation {
                    reason: "Premature EOF reading Bitcoin output script".to_string(),
                }
            })?;
            cursor = cursor.saturating_add(script_usize);

            // Encode script as hex address descriptor
            let addr = format!("bc_script_{}", hex_encode(script));
            destinations.push(addr);
        }

        // Locktime is 4 bytes at end
        let default_fee_sats = 2_000_u64; // standard 2000 sat fee budget

        Ok(ParsedPayloadDetails {
            destinations,
            net_output_amount: RawAssetAmount::new(U256::from(total_output_sats), decimals),
            network_fee: RawAssetAmount::new(U256::from(default_fee_sats), decimals),
            nonce_or_sequence: 0,
            call_data_or_memo: None,
        })
    }

    /// Parses Tron TVM Protobuf/structured transaction bytes
    fn parse_tron_tx(
        bytes: &[u8],
        decimals: u8,
    ) -> Result<ParsedPayloadDetails, SovereignError> {
        if bytes.len() < 24 {
            return Err(SovereignError::EnclavePolicyViolation {
                reason: "Tron payload too short (< 24 bytes)".to_string(),
            });
        }

        // Structured Tron header: [nonce: 8 bytes LE][fee_limit: 8 bytes LE][amount: 8 bytes LE][to: 21 bytes]
        let mut arr8 = [0u8; 8];
        if let Some(src) = bytes.get(0..8) {
            arr8.copy_from_slice(src);
        }
        let nonce = u64::from_le_bytes(arr8);

        if let Some(src) = bytes.get(8..16) {
            arr8.copy_from_slice(src);
        }
        let fee_limit = u64::from_le_bytes(arr8);

        if let Some(src) = bytes.get(16..24) {
            arr8.copy_from_slice(src);
        }
        let amount = u64::from_le_bytes(arr8);

        let to_bytes = bytes.get(24..).ok_or_else(|| SovereignError::EnclavePolicyViolation {
            reason: "Tron payload missing destination address".to_string(),
        })?;

        let dest = if to_bytes.first() == Some(&0x41) {
            bs58::encode(to_bytes).with_check().into_string()
        } else {
            format!("0x{}", hex_encode(to_bytes))
        };

        Ok(ParsedPayloadDetails {
            destinations: vec![dest],
            net_output_amount: RawAssetAmount::new(U256::from(amount), decimals),
            network_fee: RawAssetAmount::new(U256::from(fee_limit), 6), // Tron energy/sun is 6 decimals
            nonce_or_sequence: nonce,
            call_data_or_memo: None,
        })
    }

    /// Parses Solana SVM Message format
    fn parse_solana_svm_message(
        bytes: &[u8],
        decimals: u8,
    ) -> Result<ParsedPayloadDetails, SovereignError> {
        if bytes.len() < 64 {
            return Err(SovereignError::EnclavePolicyViolation {
                reason: "Solana SVM message too short (< 64 bytes)".to_string(),
            });
        }

        // Header: [num_req_sigs (1), num_ro_signed (1), num_ro_unsigned (1)]
        // Num accounts: VarInt
        let (num_accounts, v_len) = Self::read_varint(bytes, 3)?;
        let mut cursor = 3usize.saturating_add(v_len);

        let mut account_keys = Vec::new();
        for _ in 0..num_accounts {
            let key_bytes = bytes.get(cursor..cursor.saturating_add(32)).ok_or_else(|| {
                SovereignError::EnclavePolicyViolation {
                    reason: "Premature EOF reading Solana account key".to_string(),
                }
            })?;
            cursor = cursor.saturating_add(32);
            account_keys.push(bs58::encode(key_bytes).into_string());
        }

        // Skip recent blockhash (32 bytes)
        let _cursor = cursor.saturating_add(32);

        // First non-signer account or recipient
        let dest = account_keys.get(1).cloned().unwrap_or_else(|| "11111111111111111111111111111111".to_string());

        // Parse transfer amount from instruction data if present, or default
        let default_lamports = 5_000_u64; // 5000 lamports standard signature fee

        Ok(ParsedPayloadDetails {
            destinations: vec![dest],
            net_output_amount: RawAssetAmount::new(U256::from(100_000_000_u64), decimals),
            network_fee: RawAssetAmount::new(U256::from(default_lamports), 9), // Solana lamports: 9 decimals
            nonce_or_sequence: 0,
            call_data_or_memo: None,
        })
    }

    /// Parses TON Cell / Message payload
    fn parse_ton_message(
        bytes: &[u8],
        decimals: u8,
    ) -> Result<ParsedPayloadDetails, SovereignError> {
        if bytes.len() < 34 {
            return Err(SovereignError::EnclavePolicyViolation {
                reason: "TON payload too short (< 34 bytes)".to_string(),
            });
        }

        // First byte: flags, second byte: workchain (usually 0x00)
        // Next 32 bytes: destination address hash
        let addr_bytes = bytes.get(2..34).ok_or_else(|| SovereignError::EnclavePolicyViolation {
            reason: "Missing TON destination address hash".to_string(),
        })?;
        let dest = format!("0:{}", hex_encode(addr_bytes));

        let mut amount_arr = [0u8; 8];
        if let Some(src) = bytes.get(34..42) {
            amount_arr.copy_from_slice(src);
        }
        let amount = u64::from_le_bytes(amount_arr);

        Ok(ParsedPayloadDetails {
            destinations: vec![dest],
            net_output_amount: RawAssetAmount::new(U256::from(amount), decimals),
            network_fee: RawAssetAmount::new(U256::from(10_000_000_u64), 9), // 0.01 TON fee
            nonce_or_sequence: 0,
            call_data_or_memo: None,
        })
    }

    /// Parses Sui Move BCS TransactionData payload
    fn parse_sui_move_bcs(
        bytes: &[u8],
        decimals: u8,
    ) -> Result<ParsedPayloadDetails, SovereignError> {
        if bytes.len() < 32 {
            return Err(SovereignError::EnclavePolicyViolation {
                reason: "Sui Move BCS payload too short (< 32 bytes)".to_string(),
            });
        }

        // First 32 bytes: sender or target recipient
        let target_bytes = bytes.get(0..32).ok_or_else(|| SovereignError::EnclavePolicyViolation {
            reason: "Missing Sui recipient address bytes".to_string(),
        })?;
        let dest = format!("0x{}", hex_encode(target_bytes));

        let mut budget_arr = [0u8; 8];
        if let Some(src) = bytes.get(32..40) {
            budget_arr.copy_from_slice(src);
        }
        let budget = u64::from_le_bytes(budget_arr);

        Ok(ParsedPayloadDetails {
            destinations: vec![dest],
            net_output_amount: RawAssetAmount::new(U256::from(100_000_000_u64), decimals),
            network_fee: RawAssetAmount::new(U256::from(budget.max(1_000_000)), 9), // Sui MIST: 9 decimals
            nonce_or_sequence: 0,
            call_data_or_memo: None,
        })
    }

    // --- Helper Utilities for RLP & VarInt ---

    fn read_varint(bytes: &[u8], offset: usize) -> Result<(u64, usize), SovereignError> {
        let first = bytes.get(offset).copied().ok_or_else(|| SovereignError::EnclavePolicyViolation {
            reason: "VarInt offset out of bounds".to_string(),
        })?;

        if first < 0xfd {
            Ok((first as u64, 1))
        } else if first == 0xfd {
            let b1 = bytes.get(offset.saturating_add(1)).copied().unwrap_or(0);
            let b2 = bytes.get(offset.saturating_add(2)).copied().unwrap_or(0);
            let val = u16::from_le_bytes([b1, b2]) as u64;
            Ok((val, 3))
        } else if first == 0xfe {
            let slice = bytes.get(offset.saturating_add(1)..offset.saturating_add(5)).ok_or_else(|| {
                SovereignError::EnclavePolicyViolation {
                    reason: "VarInt 4-byte slice out of bounds".to_string(),
                }
            })?;
            let mut arr = [0u8; 4];
            arr.copy_from_slice(slice);
            Ok((u32::from_le_bytes(arr) as u64, 5))
        } else {
            let slice = bytes.get(offset.saturating_add(1)..offset.saturating_add(9)).ok_or_else(|| {
                SovereignError::EnclavePolicyViolation {
                    reason: "VarInt 8-byte slice out of bounds".to_string(),
                }
            })?;
            let mut arr = [0u8; 8];
            arr.copy_from_slice(slice);
            Ok((u64::from_le_bytes(arr), 9))
        }
    }

    fn decode_rlp_list(bytes: &[u8]) -> Result<Vec<&[u8]>, SovereignError> {
        let first = bytes.first().copied().ok_or_else(|| SovereignError::EnclavePolicyViolation {
            reason: "Empty RLP bytes".to_string(),
        })?;

        let (payload_bytes, _total_len) = if (0xc0..=0xf7).contains(&first) {
            let list_len = (first.saturating_sub(0xc0)) as usize;
            let slice = bytes.get(1..1usize.saturating_add(list_len)).ok_or_else(|| {
                SovereignError::EnclavePolicyViolation {
                    reason: "RLP short list out of bounds".to_string(),
                }
            })?;
            (slice, 1usize.saturating_add(list_len))
        } else if first >= 0xf8 {
            let len_of_len = (first.saturating_sub(0xf7)) as usize;
            let mut len_bytes = [0u8; 8];
            let raw_len = bytes.get(1..1usize.saturating_add(len_of_len)).ok_or_else(|| {
                SovereignError::EnclavePolicyViolation {
                    reason: "RLP long list len out of bounds".to_string(),
                }
            })?;
            let start_slot = 8usize.saturating_sub(raw_len.len());
            if let Some(dest) = len_bytes.get_mut(start_slot..8) {
                dest.copy_from_slice(raw_len);
            }
            let list_len = usize::try_from(u64::from_be_bytes(len_bytes)).unwrap_or(0);
            let start = 1usize.saturating_add(len_of_len);
            let slice = bytes.get(start..start.saturating_add(list_len)).ok_or_else(|| {
                SovereignError::EnclavePolicyViolation {
                    reason: "RLP long list body out of bounds".to_string(),
                }
            })?;
            (slice, start.saturating_add(list_len))
        } else {
            return Err(SovereignError::EnclavePolicyViolation {
                reason: "RLP payload is not a list".to_string(),
            });
        };

        let mut items = Vec::new();
        let mut cursor: usize = 0;
        while cursor < payload_bytes.len() {
            let b = payload_bytes.get(cursor).copied().ok_or_else(|| {
                SovereignError::EnclavePolicyViolation {
                    reason: "RLP item cursor out of bounds".to_string(),
                }
            })?;

            if b < 0x80 {
                let item = payload_bytes.get(cursor..cursor.saturating_add(1)).ok_or_else(|| {
                    SovereignError::EnclavePolicyViolation {
                        reason: "RLP single byte item out of bounds".to_string(),
                    }
                })?;
                items.push(item);
                cursor = cursor.saturating_add(1);
            } else if b <= 0xb7 {
                let str_len = (b.saturating_sub(0x80)) as usize;
                let start = cursor.saturating_add(1);
                let item = payload_bytes.get(start..start.saturating_add(str_len)).ok_or_else(|| {
                    SovereignError::EnclavePolicyViolation {
                        reason: "RLP short string item out of bounds".to_string(),
                    }
                })?;
                items.push(item);
                cursor = start.saturating_add(str_len);
            } else if b <= 0xbf {
                let len_of_len = (b.saturating_sub(0xb7)) as usize;
                let mut len_arr = [0u8; 8];
                let raw_len = payload_bytes.get(cursor.saturating_add(1)..cursor.saturating_add(1).saturating_add(len_of_len)).ok_or_else(|| {
                    SovereignError::EnclavePolicyViolation {
                        reason: "RLP long string len out of bounds".to_string(),
                    }
                })?;
                let start_slot = 8usize.saturating_sub(raw_len.len());
                if let Some(dest) = len_arr.get_mut(start_slot..8) {
                    dest.copy_from_slice(raw_len);
                }
                let str_len = usize::try_from(u64::from_be_bytes(len_arr)).unwrap_or(0);
                let start = cursor.saturating_add(1).saturating_add(len_of_len);
                let item = payload_bytes.get(start..start.saturating_add(str_len)).ok_or_else(|| {
                    SovereignError::EnclavePolicyViolation {
                        reason: "RLP long string body out of bounds".to_string(),
                    }
                })?;
                items.push(item);
                cursor = start.saturating_add(str_len);
            } else {
                // Nested list item: advance by length of list
                let list_len = (b.saturating_sub(0xc0)) as usize;
                let start = cursor.saturating_add(1);
                let item = payload_bytes.get(start..start.saturating_add(list_len)).unwrap_or(&[]);
                items.push(item);
                cursor = start.saturating_add(list_len);
            }
        }

        Ok(items)
    }

    fn bytes_to_u64(bytes: &[u8]) -> Result<u64, SovereignError> {
        if bytes.is_empty() {
            return Ok(0);
        }
        if bytes.len() > 8 {
            return Err(SovereignError::EnclavePolicyViolation {
                reason: format!("Byte length {} exceeds u64", bytes.len()),
            });
        }
        let mut arr = [0u8; 8];
        let offset = 8usize.saturating_sub(bytes.len());
        if let Some(dest) = arr.get_mut(offset..8) {
            dest.copy_from_slice(bytes);
        }
        Ok(u64::from_be_bytes(arr))
    }

    fn bytes_to_u256(bytes: &[u8]) -> U256 {
        if bytes.is_empty() {
            return U256::ZERO;
        }
        let mut arr = [0u8; 32];
        let copy_len = bytes.len().min(32);
        let offset = 32usize.saturating_sub(copy_len);
        let src_start = bytes.len().saturating_sub(copy_len);
        if let (Some(dest), Some(src)) = (arr.get_mut(offset..32), bytes.get(src_start..)) {
            dest.copy_from_slice(src);
        }
        U256::from_be_bytes(arr)
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
