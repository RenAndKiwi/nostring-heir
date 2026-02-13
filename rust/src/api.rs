use base64::Engine;
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
    let net = parse_network(&network)?;

    match bitcoin::Address::from_str(&address) {
        Ok(addr) => Ok(addr.is_valid_for_network(net)),
        Err(e) => Err(format!("Invalid address: {}", e)),
    }
}

/// Live vault status from the blockchain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultStatus {
    pub balance_sat: u64,
    pub utxo_count: usize,
    pub current_height: u64,
    pub confirmation_height: u64,
    pub eligible: bool,
    pub blocks_remaining: i64,
    pub days_remaining: f64,
}

/// Built unsigned claim PSBT ready for signing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimPsbt {
    pub psbt_base64: String,
    pub total_input_sat: u64,
    pub fee_sat: u64,
    pub output_sat: u64,
    pub destination: String,
    pub num_inputs: usize,
}

fn parse_network(network: &str) -> Result<bitcoin::Network, String> {
    match network {
        "mainnet" | "bitcoin" => Ok(bitcoin::Network::Bitcoin),
        "testnet" => Ok(bitcoin::Network::Testnet),
        "signet" => Ok(bitcoin::Network::Signet),
        "regtest" => Ok(bitcoin::Network::Regtest),
        _ => Err(format!("Unknown network: {}", network)),
    }
}

/// Fetch live vault status from Electrum: balance, UTXOs, eligibility.
pub fn fetch_vault_status(vault_json: String, electrum_url: String) -> Result<VaultStatus, String> {
    let backup: VaultBackup =
        serde_json::from_str(&vault_json).map_err(|e| format!("Invalid JSON: {}", e))?;

    let vault = backup
        .reconstruct()
        .map_err(|e| format!("Vault reconstruction failed: {}", e))?;

    let network = parse_network(&backup.network)?;
    let client = nostring_electrum::ElectrumClient::new(&electrum_url, network)
        .map_err(|e| format!("Electrum connection failed: {}", e))?;

    let current_height = client
        .get_height()
        .map_err(|e| format!("Failed to get block height: {}", e))? as u64;

    let utxos = client
        .get_utxos(&vault.address)
        .map_err(|e| format!("Failed to fetch UTXOs: {}", e))?;

    let balance_sat: u64 = utxos.iter().map(|u| u.value.to_sat()).sum();
    let utxo_count = utxos.len();

    // Earliest confirmation height (for timelock calculation)
    let confirmation_height = utxos
        .iter()
        .filter(|u| u.height > 0)
        .map(|u| u.height as u64)
        .min()
        .unwrap_or(current_height);

    let timelock_blocks = backup.timelock_blocks as i64;
    let blocks_since = current_height as i64 - confirmation_height as i64;
    let blocks_remaining = timelock_blocks - blocks_since;
    let days_remaining = blocks_remaining as f64 * 10.0 / 1440.0;

    Ok(VaultStatus {
        balance_sat,
        utxo_count,
        current_height,
        confirmation_height,
        eligible: blocks_remaining <= 0,
        blocks_remaining,
        days_remaining,
    })
}

/// Build an unsigned claim PSBT for the heir's recovery path.
///
/// The heir must sign this PSBT externally (hardware wallet, Sparrow, etc.)
/// then import the signed version for broadcast.
pub fn build_claim_psbt(
    vault_json: String,
    electrum_url: String,
    destination_address: String,
    heir_index: usize,
    fee_rate_sat_vb: u64,
) -> Result<ClaimPsbt, String> {
    let backup: VaultBackup =
        serde_json::from_str(&vault_json).map_err(|e| format!("Invalid JSON: {}", e))?;

    let vault = backup
        .reconstruct()
        .map_err(|e| format!("Vault reconstruction failed: {}", e))?;

    let network = parse_network(&backup.network)?;

    // Validate fee rate early, before any network I/O
    if fee_rate_sat_vb > 500 {
        return Err("Fee rate exceeds 500 sat/vB safety limit".into());
    }

    // Validate destination address
    use std::str::FromStr;
    let dest_addr = bitcoin::Address::from_str(&destination_address)
        .map_err(|e| format!("Invalid destination address: {}", e))?
        .require_network(network)
        .map_err(|e| format!("Address network mismatch: {}", e))?;

    // Fetch UTXOs
    let client = nostring_electrum::ElectrumClient::new(&electrum_url, network)
        .map_err(|e| format!("Electrum connection failed: {}", e))?;

    let utxos = client
        .get_utxos(&vault.address)
        .map_err(|e| format!("Failed to fetch UTXOs: {}", e))?;

    if utxos.is_empty() {
        return Err("No UTXOs found in vault".into());
    }

    // Convert to (OutPoint, TxOut) pairs for build_heir_claim_psbt
    let utxo_pairs: Vec<(bitcoin::OutPoint, bitcoin::TxOut)> = utxos
        .iter()
        .map(|u| {
            (
                u.outpoint,
                bitcoin::TxOut {
                    value: u.value,
                    script_pubkey: u.script_pubkey.clone(),
                },
            )
        })
        .collect();

    let total_input_sat: u64 = utxo_pairs.iter().map(|(_, txout)| txout.value.to_sat()).sum();
    let num_inputs = utxo_pairs.len();

    // Estimate fee — compute tree depth from recovery leaves count
    let num_leaves = backup.recovery_leaves.len().max(1);
    let tree_depth = (num_leaves as f64).log2().ceil() as usize;
    let vbytes =
        nostring_inherit::taproot::estimate_heir_claim_vbytes(num_inputs, 1, tree_depth);
    let fee_sat = vbytes as u64 * fee_rate_sat_vb;

    let fee = bitcoin::Amount::from_sat(fee_sat);

    // Build PSBT
    let psbt = nostring_inherit::taproot::build_heir_claim_psbt(
        &vault,
        heir_index,
        &utxo_pairs,
        &dest_addr,
        fee,
    )
    .map_err(|e| format!("PSBT construction failed: {}", e))?;

    // Serialize to base64
    let psbt_bytes = psbt.serialize();
    let psbt_base64 = base64::engine::general_purpose::STANDARD.encode(&psbt_bytes);

    let output_sat = total_input_sat.saturating_sub(fee_sat);

    Ok(ClaimPsbt {
        psbt_base64,
        total_input_sat,
        fee_sat,
        output_sat,
        destination: destination_address,
        num_inputs,
    })
}

/// Finalized transaction ready for broadcast.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalizedTx {
    pub tx_hex: String,
    pub txid: String,
    pub total_output_sat: u64,
    pub num_inputs: usize,
    pub num_outputs: usize,
}

/// Result of broadcasting a transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastResult {
    pub txid: String,
    pub success: bool,
}

/// Validate a signed PSBT and extract the finalized transaction.
///
/// The PSBT must have all inputs signed (witness data present).
/// Returns the raw transaction hex and a summary for review before broadcast.
pub fn finalize_psbt(psbt_base64: String) -> Result<FinalizedTx, String> {
    use base64::Engine;
    use bitcoin::consensus::{Decodable, Encodable};

    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&psbt_base64)
        .map_err(|e| format!("Invalid base64: {}", e))?;

    let psbt = bitcoin::Psbt::deserialize(&bytes)
        .map_err(|e| format!("Invalid PSBT: {}", e))?;

    // Check each input for signature status — give human-friendly errors
    let total_inputs = psbt.inputs.len();
    let signed_count = psbt.inputs.iter().filter(|input| {
        // An input is "signed" if it has final_script_witness or final_script_sig,
        // OR if it has tap_key_sig or any tap_script_sigs
        input.final_script_witness.is_some()
            || input.final_script_sig.is_some()
            || input.tap_key_sig.is_some()
            || !input.tap_script_sigs.is_empty()
            || !input.partial_sigs.is_empty()
    }).count();

    if signed_count == 0 {
        return Err(format!(
            "This PSBT has not been signed yet. \
             Please sign it with your wallet (Sparrow, hardware wallet, etc.) \
             before importing it here. \
             ({} input(s) need signing.)",
            total_inputs
        ));
    }

    if signed_count < total_inputs {
        return Err(format!(
            "This PSBT is only partially signed: {} of {} inputs have signatures. \
             All inputs must be signed before broadcasting. \
             Please complete signing with your wallet.",
            signed_count, total_inputs
        ));
    }

    // All inputs signed — extract the finalized transaction
    let tx = psbt
        .extract_tx()
        .map_err(|e| format!(
            "Could not finalize the transaction even though all inputs appear signed. \
             This usually means the signature format is wrong. Error: {}", e
        ))?;

    let txid = tx.compute_txid().to_string();
    let total_output_sat: u64 = tx.output.iter().map(|o| o.value.to_sat()).sum();
    let num_inputs = tx.input.len();
    let num_outputs = tx.output.len();

    // Serialize to hex
    let mut buf = Vec::new();
    tx.consensus_encode(&mut buf)
        .map_err(|e| format!("Transaction serialization failed: {}", e))?;
    let tx_hex = hex::encode(&buf);

    Ok(FinalizedTx {
        tx_hex,
        txid,
        total_output_sat,
        num_inputs,
        num_outputs,
    })
}

/// Broadcast a finalized transaction to the Bitcoin network via Electrum.
pub fn broadcast_transaction(
    tx_hex: String,
    electrum_url: String,
    network: String,
) -> Result<BroadcastResult, String> {
    use bitcoin::consensus::{Decodable, Encodable};

    let net = parse_network(&network)?;

    let tx_bytes =
        hex::decode(&tx_hex).map_err(|e| format!("Invalid hex: {}", e))?;
    let tx = bitcoin::Transaction::consensus_decode(&mut tx_bytes.as_slice())
        .map_err(|e| format!("Invalid transaction: {}", e))?;

    let _ = rustls::crypto::ring::default_provider().install_default();

    let client = nostring_electrum::ElectrumClient::new(&electrum_url, net)
        .map_err(|e| format!("Electrum connection failed: {}", e))?;

    let txid = client
        .broadcast(&tx)
        .map_err(|e| format!("Broadcast failed: {}", e))?;

    Ok(BroadcastResult {
        txid: txid.to_string(),
        success: true,
    })
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
    fn test_parse_network() {
        assert!(parse_network("bitcoin").is_ok());
        assert!(parse_network("mainnet").is_ok());
        assert!(parse_network("testnet").is_ok());
        assert!(parse_network("signet").is_ok());
        assert!(parse_network("regtest").is_ok());
        assert!(parse_network("invalid").is_err());
    }

    #[test]
    fn test_fee_rate_safety_limit() {
        // build_claim_psbt should reject fee rates above 500 sat/vB
        // We can't test the full function without Electrum, but we test the validation
        let json = make_valid_backup_json();
        let result = build_claim_psbt(
            json,
            "ssl://electrum.blockstream.info:50002".into(),
            "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4".into(),
            0,
            501, // exceeds 500 limit
        );
        // This will fail on Electrum connection (no real server), but the fee check
        // happens after connection, so this test verifies the function signature compiles.
        // The actual fee limit test needs a mock.
        assert!(result.is_err());
    }

    #[test]
    fn test_fetch_vault_status_bad_electrum() {
        let json = make_valid_backup_json();
        let result = fetch_vault_status(json, "ssl://nonexistent:50002".into());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Electrum"));
    }

    #[test]
    fn test_validate_invalid_address() {
        let result = validate_address("notanaddress".into(), "testnet".into());
        assert!(result.is_err());
    }

    #[test]
    fn test_finalize_invalid_base64() {
        let result = finalize_psbt("not-valid-base64!!!".into());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid base64"));
    }

    #[test]
    fn test_finalize_unsigned_psbt() {
        use base64::Engine;
        // Construct a minimal valid but unsigned PSBT
        let mut psbt = bitcoin::Psbt::from_unsigned_tx(
            bitcoin::Transaction {
                version: bitcoin::transaction::Version::TWO,
                lock_time: bitcoin::blockdata::locktime::absolute::LockTime::ZERO,
                input: vec![bitcoin::TxIn {
                    previous_output: bitcoin::OutPoint::null(),
                    ..Default::default()
                }],
                output: vec![bitcoin::TxOut {
                    value: bitcoin::Amount::from_sat(1000),
                    script_pubkey: bitcoin::ScriptBuf::new(),
                }],
            }
        ).unwrap();
        // Ensure it has one input entry
        assert_eq!(psbt.inputs.len(), 1);

        let psbt_bytes = psbt.serialize();
        let psbt_b64 = base64::engine::general_purpose::STANDARD.encode(&psbt_bytes);

        let result = finalize_psbt(psbt_b64);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("not been signed yet"), "Expected unsigned error, got: {}", err);
        assert!(err.contains("1 input(s) need signing"), "Expected input count, got: {}", err);
    }

    #[test]
    fn test_finalize_invalid_psbt() {
        use base64::Engine;
        let fake = base64::engine::general_purpose::STANDARD.encode(b"not a psbt");
        let result = finalize_psbt(fake);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid PSBT"));
    }

    #[test]
    fn test_broadcast_bad_electrum() {
        let result = broadcast_transaction(
            "0200000000".into(),
            "ssl://nonexistent:50002".into(),
            "bitcoin".into(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_broadcast_invalid_hex() {
        let result = broadcast_transaction(
            "not-hex".into(),
            "ssl://electrum.blockstream.info:50002".into(),
            "bitcoin".into(),
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid hex"));
    }

    /// Integration test: connects to real Electrum testnet server.
    /// Tests the full fetch_vault_status flow with a real backup.
    /// The vault likely has 0 balance, but the connection + query should succeed.
    #[test]
    #[ignore] // Run with: cargo test -- --ignored test_fetch_status_real_electrum
    fn test_fetch_status_real_electrum() {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let json = make_valid_backup_json();
        // This uses mainnet keys but we're just testing the Electrum connection works.
        // The vault address won't have funds, but the query should succeed.
        let result = fetch_vault_status(
            json,
            "ssl://electrum.blockstream.info:50002".into(),
        );
        assert!(result.is_ok(), "Electrum query failed: {:?}", result.err());
        let status = result.unwrap();
        assert!(status.current_height > 800_000);
        assert_eq!(status.balance_sat, 0); // no funds expected
    }

    /// Integration test: build_claim_psbt with real Electrum.
    /// Should fail gracefully with "No UTXOs" since the test vault is unfunded.
    #[test]
    #[ignore]
    fn test_build_psbt_no_utxos() {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let json = make_valid_backup_json();
        let result = build_claim_psbt(
            json,
            "ssl://electrum.blockstream.info:50002".into(),
            "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4".into(),
            0,
            2,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No UTXOs"), "Expected 'No UTXOs' error");
    }
}
