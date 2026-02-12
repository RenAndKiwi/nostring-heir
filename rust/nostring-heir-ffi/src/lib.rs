//! WASM bindings for NoString heir claim functionality.
//!
//! This crate exposes a minimal API for heirs:
//! 1. Import and validate a vault backup
//! 2. Check claim eligibility (timelock expiry)
//! 3. Validate Bitcoin addresses
//!
//! All crypto runs in Rust. Network I/O (block height, UTXOs, broadcast)
//! is handled by the JavaScript layer via Esplora REST API.

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

// ─── Types ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultInfo {
    pub network: String,
    pub vault_address: String,
    pub timelock_blocks: u16,
    pub num_heirs: u32,
    pub threshold: u32,
    pub heir_label: Option<String>,
    pub raw_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimEligibility {
    pub eligible: bool,
    pub blocks_remaining: u32,
    pub time_remaining: String,
    pub current_height: u32,
}

// ─── Core Logic (testable without WASM) ─────────────────────────────────────

pub fn do_import_vault_backup(json: &str) -> Result<VaultInfo, String> {
    let backup: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("Invalid JSON: {}", e))?;

    let version = backup
        .get("version")
        .and_then(|v| v.as_u64())
        .ok_or("Missing or invalid 'version' field")?;

    if version != 1 {
        return Err(format!("Unsupported backup version: {}", version));
    }

    let network = backup
        .get("network")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'network' field")?
        .to_string();

    let vault_address = backup
        .get("vault_address")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'vault_address' field")?
        .to_string();

    let timelock_blocks = backup
        .get("timelock_blocks")
        .and_then(|v| v.as_u64())
        .ok_or("Missing 'timelock_blocks' field")? as u16;

    let heirs = backup
        .get("heirs")
        .and_then(|v| v.as_array())
        .ok_or("Missing 'heirs' array")?;

    let threshold = backup
        .get("threshold")
        .and_then(|v| v.as_u64())
        .unwrap_or(heirs.len() as u64) as u32;

    validate_bitcoin_address(&vault_address)?;

    Ok(VaultInfo {
        network,
        vault_address,
        timelock_blocks,
        num_heirs: heirs.len() as u32,
        threshold,
        heir_label: None,
        raw_json: json.to_string(),
    })
}

pub fn do_check_eligibility(
    vault: &VaultInfo,
    current_block_height: u32,
    vault_confirmed_height: u32,
) -> ClaimEligibility {
    let blocks_needed = vault.timelock_blocks as u32;
    let blocks_since_confirm = current_block_height.saturating_sub(vault_confirmed_height);

    if blocks_since_confirm >= blocks_needed {
        ClaimEligibility {
            eligible: true,
            blocks_remaining: 0,
            time_remaining: "Ready to claim!".into(),
            current_height: current_block_height,
        }
    } else {
        let remaining = blocks_needed - blocks_since_confirm;
        let minutes = remaining as u64 * 10;
        let time_str = if minutes > 1440 {
            format!("~{} days", minutes / 1440)
        } else if minutes > 60 {
            format!("~{} hours", minutes / 60)
        } else {
            format!("~{} minutes", minutes)
        };

        ClaimEligibility {
            eligible: false,
            blocks_remaining: remaining,
            time_remaining: time_str,
            current_height: current_block_height,
        }
    }
}

/// Validate that a string is a plausible Bitcoin address.
/// Uses bech32 decoding for segwit addresses, basic checks for legacy.
fn validate_bitcoin_address(address: &str) -> Result<(), String> {
    if address.starts_with("bc1") || address.starts_with("tb1") || address.starts_with("bcrt1") {
        // Bech32/bech32m segwit address
        let (_hrp, _data) = bech32::decode(address)
            .map_err(|e| format!("Invalid bech32 address: {}", e))?;
        Ok(())
    } else if address.len() >= 25 && address.len() <= 35 {
        // Legacy base58check — basic length check (full validation would need base58)
        Ok(())
    } else {
        Err(format!("Invalid address format: {}", address))
    }
}

pub fn do_validate_address(address: &str, network: &str) -> Result<bool, String> {
    validate_bitcoin_address(address)?;

    let valid = match network {
        "mainnet" | "bitcoin" => {
            address.starts_with("bc1") || address.starts_with('1') || address.starts_with('3')
        }
        "testnet" => {
            address.starts_with("tb1")
                || address.starts_with('m')
                || address.starts_with('n')
                || address.starts_with('2')
        }
        "signet" => address.starts_with("tb1") || address.starts_with("sb1"),
        "regtest" => address.starts_with("bcrt1"),
        _ => return Err(format!("Unknown network: {}", network)),
    };

    if !valid {
        return Err(format!("Address doesn't match network '{}'", network));
    }

    Ok(true)
}

// ─── WASM Exports (thin wrappers) ───────────────────────────────────────────

#[wasm_bindgen]
pub fn import_vault_backup(json: &str) -> Result<String, JsError> {
    let info = do_import_vault_backup(json).map_err(|e| JsError::new(&e))?;
    serde_json::to_string(&info).map_err(|e| JsError::new(&format!("Serialization: {}", e)))
}

#[wasm_bindgen]
pub fn check_eligibility(
    vault_json: &str,
    current_block_height: u32,
    vault_confirmed_height: u32,
) -> Result<String, JsError> {
    let vault: VaultInfo = serde_json::from_str(vault_json)
        .map_err(|e| JsError::new(&format!("Invalid vault info: {}", e)))?;
    let result = do_check_eligibility(&vault, current_block_height, vault_confirmed_height);
    serde_json::to_string(&result).map_err(|e| JsError::new(&format!("Serialization: {}", e)))
}

#[wasm_bindgen]
pub fn validate_address(address: &str, network: &str) -> Result<bool, JsError> {
    do_validate_address(address, network).map_err(|e| JsError::new(&e))
}

// ─── Tests (use do_* functions directly) ────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_backup_json() -> String {
        serde_json::json!({
            "version": 1,
            "network": "testnet",
            "owner_pubkey": "02a1633cafcc01ebfb6d78e39f687a1f0995c62fc95f51ead10a02ee0be551b5dc",
            "cosigner_pubkey": "03d5a0bb72d71993e435d6c5a70e2aa4db500a62cfaae33c56050deefee64ec0",
            "chain_code": "cc".repeat(32),
            "address_index": 0,
            "timelock_blocks": 26280,
            "threshold": 1,
            "heirs": [{
                "label": "Alice",
                "xpub": "tpubD6NzVbkrYhZ4YkEfMbQLHRP4uigbFPUTEBGDSGMKs2M3Cfo38QkU8Eq4FJfDWqL4bJfLSMJpFDL2SuELpfMiRPRiLMFJdSeqSucxp3hP2w9",
                "fingerprint": "aabbccdd",
                "derivation_path": "m/86'/1'/0'",
                "recovery_index": 0,
                "npub": "npub1abc"
            }],
            "vault_address": "tb1pqqqqp399et2xygdj5xreqhjjvcmzhxw4aywxecjdzew6hylgvsesf3hn0c"
        })
        .to_string()
    }

    #[test]
    fn test_import_valid_backup() {
        let info = do_import_vault_backup(&sample_backup_json()).unwrap();
        assert_eq!(info.network, "testnet");
        assert_eq!(info.timelock_blocks, 26280);
        assert_eq!(info.num_heirs, 1);
        assert_eq!(info.threshold, 1);
    }

    #[test]
    fn test_import_invalid_json() {
        assert!(do_import_vault_backup("not json").is_err());
    }

    #[test]
    fn test_import_wrong_version() {
        let json = serde_json::json!({"version": 99}).to_string();
        assert!(do_import_vault_backup(&json).is_err());
    }

    #[test]
    fn test_import_missing_fields() {
        let json = serde_json::json!({"version": 1}).to_string();
        assert!(do_import_vault_backup(&json).is_err());
    }

    #[test]
    fn test_eligibility_not_ready() {
        let vault = VaultInfo {
            network: "testnet".into(),
            vault_address: "tb1ptest".into(),
            timelock_blocks: 26280,
            num_heirs: 1,
            threshold: 1,
            heir_label: None,
            raw_json: "{}".into(),
        };
        let result = do_check_eligibility(&vault, 100_000, 90_000);
        assert!(!result.eligible);
        assert_eq!(result.blocks_remaining, 16280);
        assert!(result.time_remaining.contains("days"));
    }

    #[test]
    fn test_eligibility_ready() {
        let vault = VaultInfo {
            network: "testnet".into(),
            vault_address: "tb1ptest".into(),
            timelock_blocks: 26280,
            num_heirs: 1,
            threshold: 1,
            heir_label: None,
            raw_json: "{}".into(),
        };
        let result = do_check_eligibility(&vault, 130_000, 100_000);
        assert!(result.eligible);
        assert_eq!(result.blocks_remaining, 0);
    }

    #[test]
    fn test_eligibility_exact_boundary() {
        let vault = VaultInfo {
            network: "testnet".into(),
            vault_address: "tb1ptest".into(),
            timelock_blocks: 100,
            num_heirs: 1,
            threshold: 1,
            heir_label: None,
            raw_json: "{}".into(),
        };

        let result = do_check_eligibility(&vault, 200, 100);
        assert!(result.eligible);

        let result = do_check_eligibility(&vault, 199, 100);
        assert!(!result.eligible);
        assert_eq!(result.blocks_remaining, 1);
    }

    #[test]
    fn test_validate_address_testnet() {
        assert!(
            do_validate_address("tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx", "testnet").is_ok()
        );
        assert!(
            do_validate_address("bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4", "testnet").is_err()
        );
    }

    #[test]
    fn test_validate_address_mainnet() {
        assert!(
            do_validate_address("bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4", "mainnet").is_ok()
        );
        assert!(do_validate_address("tb1qtest", "mainnet").is_err());
    }

    #[test]
    fn test_validate_address_invalid() {
        assert!(do_validate_address("not_an_address", "mainnet").is_err());
    }
}
