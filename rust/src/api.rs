use serde::{Deserialize, Serialize};

use nostring_inherit::backup::VaultBackup;

/// Vault summary returned after parsing and verifying a VaultBackup JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultInfo {
    pub network: String,
    pub vault_address: String,
    pub timelock_blocks: u16,
    pub heir_count: usize,
    pub heir_labels: Vec<String>,
    pub has_recovery_leaves: bool,
    pub address_verified: bool,
}

/// Claim eligibility status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimEligibility {
    pub eligible: bool,
    pub blocks_remaining: i64,
    pub days_remaining: f64,
}

/// Parse, validate, and VERIFY a VaultBackup JSON string.
///
/// Reconstructs the vault from raw key material and verifies the address matches.
/// If verification fails, returns an error — the backup may be corrupt or tampered.
pub fn import_vault_backup(json: String) -> Result<VaultInfo, String> {
    let backup: VaultBackup =
        serde_json::from_str(&json).map_err(|e| format!("Invalid JSON: {}", e))?;

    // Reconstruct vault and verify address
    let _vault = backup
        .reconstruct()
        .map_err(|e| format!("Vault verification failed: {}", e))?;

    let heir_labels: Vec<String> = backup.heirs.iter().map(|h| h.label.clone()).collect();

    Ok(VaultInfo {
        network: backup.network.clone(),
        vault_address: backup.vault_address.clone(),
        timelock_blocks: backup.timelock_blocks,
        heir_count: backup.heirs.len(),
        heir_labels,
        has_recovery_leaves: !backup.recovery_leaves.is_empty(),
        address_verified: true,
    })
}

/// Check if an heir is eligible to claim based on current block height.
pub fn check_eligibility(
    vault_json: String,
    current_height: u64,
    confirmation_height: u64,
) -> Result<ClaimEligibility, String> {
    let backup: VaultBackup =
        serde_json::from_str(&vault_json).map_err(|e| format!("Invalid JSON: {}", e))?;

    let timelock_blocks = backup.timelock_blocks as i64;
    let blocks_since_confirm = current_height as i64 - confirmation_height as i64;
    let blocks_remaining = timelock_blocks - blocks_since_confirm;
    let days_remaining = blocks_remaining as f64 * 10.0 / 1440.0;

    Ok(ClaimEligibility {
        eligible: blocks_remaining <= 0,
        blocks_remaining,
        days_remaining,
    })
}

/// Validate a Bitcoin address string for the given network.
pub fn validate_address(address: String, network: String) -> Result<bool, String> {
    use std::str::FromStr;
    let net = match network.as_str() {
        "mainnet" | "bitcoin" => bitcoin::Network::Bitcoin,
        "testnet" => bitcoin::Network::Testnet,
        "signet" => bitcoin::Network::Signet,
        "regtest" => bitcoin::Network::Regtest,
        _ => return Err(format!("Unknown network: {}", network)),
    };

    match bitcoin::Address::from_str(&address) {
        Ok(addr) => Ok(addr.is_valid_for_network(net)),
        Err(e) => Err(format!("Invalid address: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_valid_backup_json() -> String {
        // Create a real vault to get a valid backup with correct address
        use bitcoin::bip32::Xpub;
        use bitcoin::secp256k1::PublicKey;
        use miniscript::DescriptorPublicKey;
        use nostring_ccd::types::{ChainCode, DelegatedKey};
        use nostring_inherit::backup::{extract_recovery_leaves, HeirBackupEntry};
        use nostring_inherit::policy::{PathInfo, Timelock};
        use std::str::FromStr;

        let owner_pubkey = PublicKey::from_slice(
            &hex::decode("02a1633cafcc01ebfb6d78e39f687a1f0995c62fc95f51ead10a02ee0be551b5dc")
                .unwrap(),
        )
        .unwrap();
        let cosigner_pubkey = PublicKey::from_slice(
            &hex::decode("03a1633cafcc01ebfb6d78e39f687a1f0995c62fc95f51ead10a02ee0be551b5dc")
                .unwrap(),
        )
        .unwrap();
        let chain_code = ChainCode([0xab; 32]);
        let delegated = DelegatedKey {
            cosigner_pubkey,
            chain_code,
            label: "test-cosigner".into(),
        };
        let heir_xpub = Xpub::from_str(
            "xpub661MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet8",
        )
        .unwrap();

        let xonly = heir_xpub.public_key.x_only_public_key().0;
        let desc = DescriptorPublicKey::from_str(&format!("{}", xonly)).unwrap();
        let path_info = PathInfo::Single(desc);
        let timelock = Timelock::from_blocks(26280).unwrap();

        let vault = nostring_inherit::taproot::create_inheritable_vault(
            &owner_pubkey,
            &delegated,
            0,
            path_info,
            timelock,
            0,
            bitcoin::Network::Bitcoin,
        )
        .unwrap();

        let backup = VaultBackup {
            version: 1,
            network: "bitcoin".into(),
            owner_pubkey: hex::encode(owner_pubkey.serialize()),
            cosigner_pubkey: hex::encode(cosigner_pubkey.serialize()),
            chain_code: "ab".repeat(32),
            address_index: 0,
            timelock_blocks: 26280,
            threshold: 1,
            heirs: vec![HeirBackupEntry {
                label: "Alice".into(),
                xpub: heir_xpub.to_string(),
                fingerprint: "00000000".into(),
                derivation_path: "m/84'/0'/0'".into(),
                recovery_index: 0,
                npub: None,
            }],
            vault_address: vault.address.to_string(),
            taproot_internal_key: Some(hex::encode(vault.aggregate_xonly.serialize())),
            recovery_leaves: extract_recovery_leaves(&vault),
            created_at: None,
        };

        serde_json::to_string(&backup).unwrap()
    }

    #[test]
    fn test_import_valid_backup() {
        let json = make_valid_backup_json();
        let result = import_vault_backup(json);
        assert!(result.is_ok(), "Error: {:?}", result.err());
        let info = result.unwrap();
        assert_eq!(info.network, "bitcoin");
        assert_eq!(info.timelock_blocks, 26280);
        assert_eq!(info.heir_count, 1);
        assert_eq!(info.heir_labels, vec!["Alice"]);
        assert!(info.has_recovery_leaves);
        assert!(info.address_verified);
    }

    #[test]
    fn test_import_invalid_json() {
        let result = import_vault_backup("not json".into());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid JSON"));
    }

    #[test]
    fn test_import_tampered_address() {
        let mut backup: VaultBackup =
            serde_json::from_str(&make_valid_backup_json()).unwrap();
        backup.vault_address = "bc1ptampered".into();
        let json = serde_json::to_string(&backup).unwrap();
        let result = import_vault_backup(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Vault verification failed"));
    }

    #[test]
    fn test_eligibility_not_ready() {
        let json = make_valid_backup_json();
        let result = check_eligibility(json, 100, 50);
        assert!(result.is_ok());
        let elig = result.unwrap();
        assert!(!elig.eligible);
        assert!(elig.blocks_remaining > 0);
    }

    #[test]
    fn test_eligibility_ready() {
        let json = make_valid_backup_json();
        let result = check_eligibility(json, 30000, 0);
        assert!(result.is_ok());
        let elig = result.unwrap();
        assert!(elig.eligible);
        assert!(elig.blocks_remaining <= 0);
    }

    #[test]
    fn test_validate_mainnet_address() {
        let result = validate_address(
            "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4".into(),
            "bitcoin".into(),
        );
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_validate_wrong_network() {
        let result = validate_address(
            "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4".into(),
            "testnet".into(),
        );
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_validate_invalid_address() {
        let result = validate_address("notanaddress".into(), "testnet".into());
        assert!(result.is_err());
    }
}
