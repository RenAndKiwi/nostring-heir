//! WASM bindings for NoString heir claim functionality.
//!
//! This crate exposes a minimal API for heirs to:
//! 1. Import and validate a vault backup
//! 2. Check claim eligibility (timelock expiry)
//! 3. Build an unsigned claim PSBT
//! 4. Finalize and broadcast the signed transaction
//!
//! All crypto runs in Rust. The SvelteKit PWA calls these
//! functions through wasm-bindgen generated TypeScript bindings.

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

// ─── Types (serialized as JSON across WASM boundary) ────────────────────────

/// Parsed and validated vault backup.
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

/// Claim eligibility status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimEligibility {
    pub eligible: bool,
    pub blocks_remaining: u32,
    pub time_remaining: String,
    pub current_height: u32,
}

/// An unspent transaction output at the vault address.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtxoInfo {
    pub txid: String,
    pub vout: u32,
    pub value: u64,
    pub confirmations: u32,
}

/// An unsigned claim transaction ready for signing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsignedClaim {
    pub psbt_base64: String,
    pub total_sats: u64,
    pub fee_sats: u64,
    pub destination: String,
}

// ─── WASM-exported Functions ────────────────────────────────────────────────

/// Import and validate a vault backup JSON string.
/// Returns VaultInfo as JSON string, or throws on error.
#[wasm_bindgen]
pub fn import_vault_backup(json: &str) -> Result<String, JsError> {
    let backup: serde_json::Value =
        serde_json::from_str(json).map_err(|e| JsError::new(&format!("Invalid JSON: {}", e)))?;

    let version = backup
        .get("version")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| JsError::new("Missing or invalid 'version' field"))?;

    if version != 1 {
        return Err(JsError::new(&format!(
            "Unsupported backup version: {}",
            version
        )));
    }

    let network = backup
        .get("network")
        .and_then(|v| v.as_str())
        .ok_or_else(|| JsError::new("Missing 'network' field"))?
        .to_string();

    let vault_address = backup
        .get("vault_address")
        .and_then(|v| v.as_str())
        .ok_or_else(|| JsError::new("Missing 'vault_address' field"))?
        .to_string();

    let timelock_blocks = backup
        .get("timelock_blocks")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| JsError::new("Missing 'timelock_blocks' field"))? as u16;

    let heirs = backup
        .get("heirs")
        .and_then(|v| v.as_array())
        .ok_or_else(|| JsError::new("Missing 'heirs' array"))?;

    let threshold = backup
        .get("threshold")
        .and_then(|v| v.as_u64())
        .unwrap_or(heirs.len() as u64) as u32;

    // Validate address parses
    let _addr: bitcoin::Address<bitcoin::address::NetworkUnchecked> = vault_address
        .parse()
        .map_err(|e| JsError::new(&format!("Invalid vault address: {}", e)))?;

    let info = VaultInfo {
        network,
        vault_address,
        timelock_blocks,
        num_heirs: heirs.len() as u32,
        threshold,
        heir_label: None,
        raw_json: json.to_string(),
    };

    serde_json::to_string(&info).map_err(|e| JsError::new(&format!("Serialization error: {}", e)))
}

/// Check if the heir is eligible to claim (timelock expired).
/// Returns ClaimEligibility as JSON string.
#[wasm_bindgen]
pub fn check_eligibility(
    vault_json: &str,
    current_block_height: u32,
    vault_confirmed_height: u32,
) -> Result<String, JsError> {
    let vault: VaultInfo = serde_json::from_str(vault_json)
        .map_err(|e| JsError::new(&format!("Invalid vault info: {}", e)))?;

    let blocks_needed = vault.timelock_blocks as u32;
    let blocks_since_confirm = current_block_height.saturating_sub(vault_confirmed_height);

    let result = if blocks_since_confirm >= blocks_needed {
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
    };

    serde_json::to_string(&result)
        .map_err(|e| JsError::new(&format!("Serialization error: {}", e)))
}

// ─── Non-WASM functions (for native tests) ──────────────────────────────────

/// Parse network string to bitcoin::Network.
fn parse_network(network: &str) -> Result<bitcoin::Network, String> {
    match network {
        "mainnet" | "bitcoin" => Ok(bitcoin::Network::Bitcoin),
        "testnet" => Ok(bitcoin::Network::Testnet),
        "signet" => Ok(bitcoin::Network::Signet),
        "regtest" => Ok(bitcoin::Network::Regtest),
        other => Err(format!("Unknown network: {}", other)),
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

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
            "vault_address": "tb1pexample"
        })
        .to_string()
    }

    #[test]
    fn test_import_valid_backup() {
        let result = import_vault_backup(&sample_backup_json());
        assert!(result.is_ok());
        let info: VaultInfo = serde_json::from_str(&result.unwrap()).unwrap();
        assert_eq!(info.network, "testnet");
        assert_eq!(info.timelock_blocks, 26280);
        assert_eq!(info.num_heirs, 1);
        assert_eq!(info.threshold, 1);
    }

    #[test]
    fn test_import_invalid_json() {
        let result = import_vault_backup("not json");
        assert!(result.is_err());
    }

    #[test]
    fn test_import_wrong_version() {
        let json = serde_json::json!({"version": 99}).to_string();
        let result = import_vault_backup(&json);
        assert!(result.is_err());
    }

    #[test]
    fn test_import_missing_fields() {
        let json = serde_json::json!({"version": 1}).to_string();
        let result = import_vault_backup(&json);
        assert!(result.is_err());
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
        let vault_json = serde_json::to_string(&vault).unwrap();

        let result_json = check_eligibility(&vault_json, 100_000, 90_000).unwrap();
        let result: ClaimEligibility = serde_json::from_str(&result_json).unwrap();
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
        let vault_json = serde_json::to_string(&vault).unwrap();

        let result_json = check_eligibility(&vault_json, 130_000, 100_000).unwrap();
        let result: ClaimEligibility = serde_json::from_str(&result_json).unwrap();
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
        let vault_json = serde_json::to_string(&vault).unwrap();

        // Exactly at boundary = eligible
        let r = check_eligibility(&vault_json, 200, 100).unwrap();
        let e: ClaimEligibility = serde_json::from_str(&r).unwrap();
        assert!(e.eligible);

        // One block short = not eligible
        let r = check_eligibility(&vault_json, 199, 100).unwrap();
        let e: ClaimEligibility = serde_json::from_str(&r).unwrap();
        assert!(!e.eligible);
        assert_eq!(e.blocks_remaining, 1);
    }

    #[test]
    fn test_parse_network() {
        assert!(parse_network("mainnet").is_ok());
        assert!(parse_network("testnet").is_ok());
        assert!(parse_network("signet").is_ok());
        assert!(parse_network("regtest").is_ok());
        assert!(parse_network("bitcoin").is_ok());
        assert!(parse_network("invalid").is_err());
    }
}
