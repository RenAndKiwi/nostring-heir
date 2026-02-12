//! UniFFI bindings for NoString heir claim functionality.
//!
//! This crate exposes a minimal API for heirs to:
//! 1. Import and validate a vault backup
//! 2. Check claim eligibility (timelock expiry)
//! 3. Build an unsigned claim PSBT
//! 4. Broadcast the signed transaction
//!
//! All crypto runs in Rust. The mobile app (React Native) calls these
//! functions through generated TypeScript bindings.

use bitcoin::Network;
use serde::{Deserialize, Serialize};

uniffi::setup_scaffolding!();

// ─── Error Type ─────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum HeirError {
    #[error("Invalid backup: {reason}")]
    InvalidBackup { reason: String },
    #[error("Not eligible: {reason}")]
    NotEligible { reason: String },
    #[error("Invalid address: {reason}")]
    InvalidAddress { reason: String },
    #[error("Network error: {reason}")]
    NetworkError { reason: String },
    #[error("Signing error: {reason}")]
    SigningError { reason: String },
}

// ─── Types ──────────────────────────────────────────────────────────────────

/// Parsed and validated vault backup.
#[derive(Debug, Clone, uniffi::Record)]
pub struct VaultInfo {
    /// Bitcoin network (mainnet, testnet, signet, regtest)
    pub network: String,
    /// Vault P2TR address
    pub vault_address: String,
    /// Timelock in blocks
    pub timelock_blocks: u16,
    /// Number of heirs
    pub num_heirs: u32,
    /// Threshold required to claim
    pub threshold: u32,
    /// This heir's label (if identifiable)
    pub heir_label: Option<String>,
    /// Raw backup JSON (preserved for reconstruction)
    pub raw_json: String,
}

/// Claim eligibility status.
#[derive(Debug, Clone, uniffi::Record)]
pub struct ClaimEligibility {
    /// Whether the claim can be made now
    pub eligible: bool,
    /// Blocks remaining until eligible (0 if eligible)
    pub blocks_remaining: u32,
    /// Estimated time remaining in human-readable format
    pub time_remaining: String,
    /// Current block height
    pub current_height: u32,
}

/// An unspent transaction output at the vault address.
#[derive(Debug, Clone, uniffi::Record)]
pub struct UtxoInfo {
    /// Transaction ID
    pub txid: String,
    /// Output index
    pub vout: u32,
    /// Value in satoshis
    pub value: u64,
    /// Confirmations
    pub confirmations: u32,
}

/// An unsigned claim transaction ready for signing.
#[derive(Debug, Clone, uniffi::Record)]
pub struct UnsignedClaim {
    /// Base64-encoded PSBT
    pub psbt_base64: String,
    /// Total amount being claimed (sats)
    pub total_sats: u64,
    /// Fee in sats
    pub fee_sats: u64,
    /// Destination address
    pub destination: String,
}

// ─── Functions ──────────────────────────────────────────────────────────────

/// Import and validate a vault backup JSON string.
///
/// Returns parsed vault info if valid.
#[uniffi::export]
pub fn import_vault_backup(json: String) -> Result<VaultInfo, HeirError> {
    // Parse the backup JSON
    let backup: serde_json::Value = serde_json::from_str(&json).map_err(|e| {
        HeirError::InvalidBackup {
            reason: format!("Invalid JSON: {}", e),
        }
    })?;

    let version = backup
        .get("version")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| HeirError::InvalidBackup {
            reason: "Missing or invalid 'version' field".into(),
        })?;

    if version != 1 {
        return Err(HeirError::InvalidBackup {
            reason: format!("Unsupported backup version: {}", version),
        });
    }

    let network = backup
        .get("network")
        .and_then(|v| v.as_str())
        .ok_or_else(|| HeirError::InvalidBackup {
            reason: "Missing 'network' field".into(),
        })?
        .to_string();

    let vault_address = backup
        .get("vault_address")
        .and_then(|v| v.as_str())
        .ok_or_else(|| HeirError::InvalidBackup {
            reason: "Missing 'vault_address' field".into(),
        })?
        .to_string();

    let timelock_blocks = backup
        .get("timelock_blocks")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| HeirError::InvalidBackup {
            reason: "Missing 'timelock_blocks' field".into(),
        })? as u16;

    let heirs = backup
        .get("heirs")
        .and_then(|v| v.as_array())
        .ok_or_else(|| HeirError::InvalidBackup {
            reason: "Missing 'heirs' array".into(),
        })?;

    let threshold = backup
        .get("threshold")
        .and_then(|v| v.as_u64())
        .unwrap_or(heirs.len() as u64) as u32;

    // Validate address parses
    let _addr: bitcoin::Address<bitcoin::address::NetworkUnchecked> =
        vault_address.parse().map_err(|e| HeirError::InvalidBackup {
            reason: format!("Invalid vault address: {}", e),
        })?;

    Ok(VaultInfo {
        network,
        vault_address,
        timelock_blocks,
        num_heirs: heirs.len() as u32,
        threshold,
        heir_label: None,
        raw_json: json,
    })
}

/// Check if the heir is eligible to claim (timelock expired).
#[uniffi::export]
pub fn check_eligibility(
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
        let minutes = remaining as u64 * 10; // ~10 min per block
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

/// Get current block height from an Electrum server.
#[uniffi::export]
pub fn get_block_height(electrum_url: String, network: String) -> Result<u32, HeirError> {
    let net = parse_network(&network)?;
    let client =
        nostring_electrum::ElectrumClient::new(&electrum_url, net).map_err(|e| {
            HeirError::NetworkError {
                reason: format!("Connection failed: {}", e),
            }
        })?;

    let height = client.get_height().map_err(|e| HeirError::NetworkError {
        reason: format!("Failed to get height: {}", e),
    })?;

    Ok(height as u32)
}

/// Broadcast a raw transaction hex.
#[uniffi::export]
pub fn broadcast_transaction(
    electrum_url: String,
    network: String,
    raw_tx_hex: String,
) -> Result<String, HeirError> {
    let net = parse_network(&network)?;
    let client =
        nostring_electrum::ElectrumClient::new(&electrum_url, net).map_err(|e| {
            HeirError::NetworkError {
                reason: format!("Connection failed: {}", e),
            }
        })?;

    let tx_bytes = hex::decode(&raw_tx_hex).map_err(|e| HeirError::SigningError {
        reason: format!("Invalid hex: {}", e),
    })?;

    let tx: bitcoin::Transaction =
        bitcoin::consensus::deserialize(&tx_bytes).map_err(|e| HeirError::SigningError {
            reason: format!("Invalid transaction: {}", e),
        })?;

    let txid = client.broadcast(&tx).map_err(|e| HeirError::NetworkError {
        reason: format!("Broadcast failed: {}", e),
    })?;

    Ok(txid.to_string())
}

// ─── Helpers ────────────────────────────────────────────────────────────────

fn parse_network(network: &str) -> Result<Network, HeirError> {
    match network {
        "mainnet" | "bitcoin" => Ok(Network::Bitcoin),
        "testnet" => Ok(Network::Testnet),
        "signet" => Ok(Network::Signet),
        "regtest" => Ok(Network::Regtest),
        other => Err(HeirError::InvalidBackup {
            reason: format!("Unknown network: {}", other),
        }),
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
        let result = import_vault_backup(sample_backup_json());
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.network, "testnet");
        assert_eq!(info.timelock_blocks, 26280);
        assert_eq!(info.num_heirs, 1);
        assert_eq!(info.threshold, 1);
    }

    #[test]
    fn test_import_invalid_json() {
        let result = import_vault_backup("not json".into());
        assert!(result.is_err());
        match result.unwrap_err() {
            HeirError::InvalidBackup { reason } => assert!(reason.contains("Invalid JSON")),
            _ => panic!("Expected InvalidBackup"),
        }
    }

    #[test]
    fn test_import_wrong_version() {
        let json = serde_json::json!({"version": 99}).to_string();
        let result = import_vault_backup(json);
        assert!(result.is_err());
        match result.unwrap_err() {
            HeirError::InvalidBackup { reason } => assert!(reason.contains("Unsupported")),
            _ => panic!("Expected InvalidBackup"),
        }
    }

    #[test]
    fn test_import_missing_fields() {
        let json = serde_json::json!({"version": 1}).to_string();
        let result = import_vault_backup(json);
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

        let result = check_eligibility(&vault, 100_000, 90_000);
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

        let result = check_eligibility(&vault, 130_000, 100_000);
        assert!(result.eligible);
        assert_eq!(result.blocks_remaining, 0);
        assert_eq!(result.time_remaining, "Ready to claim!");
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

        // Exactly at boundary
        let result = check_eligibility(&vault, 200, 100);
        assert!(result.eligible);

        // One block short
        let result = check_eligibility(&vault, 199, 100);
        assert!(!result.eligible);
        assert_eq!(result.blocks_remaining, 1);
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
