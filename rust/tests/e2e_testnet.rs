//! End-to-end integration test for the heir app FFI layer.
//!
//! Proves the full inheritance claim flow through the same code paths
//! the Flutter app uses:
//!
//! 1. Create inheritable vault (owner + cosigner + heir)
//! 2. Generate VaultBackup JSON
//! 3. import_vault_backup → verified VaultInfo
//! 4. Build claim PSBT (with mock UTXOs)
//! 5. Sign PSBT with heir's test key
//! 6. finalize_psbt → FinalizedTx
//! 7. Verify via bitcoinconsensus

use base64::Engine;
use bitcoin::hashes::Hash as _;
use bitcoin::secp256k1::{Keypair, PublicKey, Secp256k1, SecretKey};
use bitcoin::sighash::{Prevouts, SighashCache, TapSighashType};
use bitcoin::taproot::LeafVersion;
use bitcoin::{Address, Amount, Network, OutPoint, TxOut, Txid, Witness};
use miniscript::descriptor::DescriptorPublicKey;
use std::str::FromStr;

use nostring_ccd::register_cosigner_with_chain_code;
use nostring_ccd::types::ChainCode;
use nostring_inherit::backup::{extract_recovery_leaves, HeirBackupEntry, VaultBackup};
use nostring_inherit::policy::{PathInfo, Timelock};
use nostring_inherit::taproot::{build_heir_claim_psbt, create_inheritable_vault};

// Re-use the FFI functions directly
use nostring_heir_ffi::api::{finalize_psbt, import_vault_backup};

fn test_keypair(seed: u8) -> (SecretKey, PublicKey) {
    let secp = Secp256k1::new();
    let mut bytes = [0u8; 32];
    bytes[31] = seed;
    bytes[0] = 0x01;
    let sk = SecretKey::from_slice(&bytes).unwrap();
    let pk = sk.public_key(&secp);
    (sk, pk)
}

/// Full E2E: vault creation → VaultBackup → import → claim PSBT → sign → finalize → consensus verify
#[test]
fn test_e2e_heir_claim_through_ffi() {
    let secp = Secp256k1::new();

    // === 1. Create vault ===
    let (_owner_sk, owner_pk) = test_keypair(1);
    let (_cosigner_sk, cosigner_pk) = test_keypair(2);
    let chain_code = ChainCode([0xAB; 32]);
    let delegated = register_cosigner_with_chain_code(cosigner_pk, chain_code, "test-cosigner");

    // Generate heir keypair — we need the secret key for signing
    let (heir_sk, _heir_pk) = test_keypair(3);
    let heir_keypair = Keypair::from_secret_key(&secp, &heir_sk);
    let heir_xonly = heir_keypair.x_only_public_key().0;
    let heir_desc = DescriptorPublicKey::from_str(&format!("{}", heir_xonly)).unwrap();
    let path_info = PathInfo::Single(heir_desc);

    // 1-block CSV timelock for testability
    let timelock = Timelock::from_blocks(1).unwrap();

    let vault = create_inheritable_vault(
        &owner_pk,
        &delegated,
        0,
        path_info,
        timelock,
        0,
        Network::Testnet,
    )
    .unwrap();

    println!("Vault address: {}", vault.address);

    // === 2. Generate VaultBackup JSON ===
    // Build a valid xpub from the heir's key by encoding raw bytes
    // xpub format: 4 bytes version + 1 byte depth + 4 bytes fingerprint + 4 bytes child + 32 bytes chaincode + 33 bytes pubkey + 4 bytes checksum
    let heir_compressed = _heir_pk.serialize();
    let mut xpub_payload = Vec::with_capacity(78);
    xpub_payload.extend_from_slice(&[0x04, 0x88, 0xB2, 0x1E]); // xpub version
    xpub_payload.push(0x00); // depth
    xpub_payload.extend_from_slice(&[0x00; 4]); // parent fingerprint
    xpub_payload.extend_from_slice(&[0x00; 4]); // child number
    xpub_payload.extend_from_slice(&[0x00; 32]); // chain code
    xpub_payload.extend_from_slice(&heir_compressed);
    // base58check encode
    let heir_xpub = bitcoin::bip32::Xpub::decode(&xpub_payload).unwrap();
    let heir_xpub_str = heir_xpub.to_string();

    let backup = VaultBackup {
        version: 1,
        network: "testnet".into(),
        owner_pubkey: hex::encode(owner_pk.serialize()),
        cosigner_pubkey: hex::encode(cosigner_pk.serialize()),
        chain_code: "ab".repeat(32),
        address_index: 0,
        timelock_blocks: 1,
        threshold: 1,
        heirs: vec![HeirBackupEntry {
            label: "TestHeir".into(),
            xpub: heir_xpub_str.clone(),
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

    let backup_json = serde_json::to_string_pretty(&backup).unwrap();
    println!("VaultBackup JSON:\n{}", backup_json);

    // === 3. Import via FFI ===
    let vault_info = import_vault_backup(backup_json.clone()).unwrap();
    assert_eq!(vault_info.network, "testnet");
    assert_eq!(vault_info.heir_count, 1);
    assert_eq!(vault_info.heir_labels, vec!["TestHeir"]);
    assert!(vault_info.has_recovery_leaves);
    assert!(vault_info.address_verified);
    println!("✅ import_vault_backup succeeded — address verified");

    // === 4. Build claim PSBT with mock UTXO ===
    // Simulate a funded vault with 50,000 sats
    let mock_txid =
        Txid::from_slice(&[0x42; 32]).unwrap();
    let mock_outpoint = OutPoint::new(mock_txid, 0);
    let mock_txout = TxOut {
        value: Amount::from_sat(50_000),
        script_pubkey: vault.address.script_pubkey(),
    };

    let fee = Amount::from_sat(300);
    let destination = Address::from_str("tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx")
        .unwrap()
        .assume_checked();

    let psbt = build_heir_claim_psbt(
        &vault,
        0, // recovery_index
        &[(mock_outpoint, mock_txout.clone())],
        &destination,
        fee,
    )
    .unwrap();

    println!("✅ build_heir_claim_psbt succeeded — {} inputs, {} outputs",
        psbt.unsigned_tx.input.len(), psbt.unsigned_tx.output.len());

    // === 5. Sign the PSBT ===
    let mut signed_psbt = psbt.clone();
    let (_recovery_timelock, recovery_script) = &vault.recovery_scripts[0];

    // Compute the sighash for script-path spend
    let mut sighash_cache = SighashCache::new(&psbt.unsigned_tx);
    let leaf_hash = bitcoin::taproot::TapLeafHash::from_script(
        recovery_script,
        LeafVersion::TapScript,
    );

    let sighash = sighash_cache
        .taproot_script_spend_signature_hash(
            0,
            &Prevouts::All(&[mock_txout.clone()]),
            leaf_hash,
            TapSighashType::Default,
        )
        .unwrap();

    let msg = bitcoin::secp256k1::Message::from_digest(sighash.to_byte_array());
    let sig = secp.sign_schnorr_no_aux_rand(&msg, &heir_keypair);

    let tap_sig = bitcoin::taproot::Signature {
        signature: sig,
        sighash_type: TapSighashType::Default,
    };

    // Build witness: <sig> <script> <control_block>
    let control_block = vault
        .taproot_spend_info
        .control_block(&(recovery_script.clone(), LeafVersion::TapScript))
        .expect("control block must exist");

    let mut witness = Witness::new();
    witness.push(tap_sig.serialize());
    witness.push(recovery_script.as_bytes());
    witness.push(control_block.serialize());

    signed_psbt.inputs[0].final_script_witness = Some(witness.clone());

    println!("✅ PSBT signed with heir's key");

    // === 6. Finalize via FFI ===
    let psbt_bytes = signed_psbt.serialize();
    let psbt_base64 = base64::engine::general_purpose::STANDARD.encode(&psbt_bytes);

    let finalized = finalize_psbt(psbt_base64).unwrap();
    assert!(!finalized.tx_hex.is_empty());
    assert!(!finalized.txid.is_empty());
    assert_eq!(finalized.num_inputs, 1);
    assert_eq!(finalized.num_outputs, 1);
    println!("✅ finalize_psbt succeeded — txid: {}", finalized.txid);

    // === 7. Consensus verification ===
    let tx_bytes = hex::decode(&finalized.tx_hex).unwrap();

    let script_bytes = mock_txout.script_pubkey.as_bytes();
    let all_utxos: Vec<bitcoinconsensus::Utxo> = vec![{
        let sb = script_bytes;
        bitcoinconsensus::Utxo {
            script_pubkey: sb.as_ptr(),
            script_pubkey_len: sb.len() as u32,
            value: mock_txout.value.to_sat() as i64,
        }
    }];

    let result = bitcoinconsensus::verify(
        script_bytes,
        mock_txout.value.to_sat(),
        &tx_bytes,
        Some(&all_utxos),
        0,
    );

    assert!(result.is_ok(), "Consensus verification failed: {:?}", result);
    println!("✅ bitcoinconsensus verification PASSED");
    println!("\n🎉 Full E2E: vault → backup → import → claim PSBT → sign → finalize → consensus verify");
}

/// Integration test: real Electrum testnet query
/// Requires a funded vault on testnet3. Run manually.
#[test]
#[ignore]
fn test_e2e_live_electrum_status() {
    let _ = rustls::crypto::ring::default_provider().install_default();

    let secp = Secp256k1::new();
    let (_owner_sk, owner_pk) = test_keypair(1);
    let (__cosigner_sk, cosigner_pk) = test_keypair(2);
    let (heir_sk, _heir_pk) = test_keypair(3);

    let chain_code = ChainCode([0xAB; 32]);
    let delegated = register_cosigner_with_chain_code(cosigner_pk, chain_code, "test-cosigner");

    let heir_keypair = Keypair::from_secret_key(&secp, &heir_sk);
    let heir_xonly = heir_keypair.x_only_public_key().0;
    let heir_desc = DescriptorPublicKey::from_str(&format!("{}", heir_xonly)).unwrap();
    let path_info = PathInfo::Single(heir_desc);
    let timelock = Timelock::from_blocks(1).unwrap();

    let vault = create_inheritable_vault(
        &owner_pk, &delegated, 0, path_info, timelock, 0, Network::Testnet,
    )
    .unwrap();

    println!("Vault address (fund this for live test): {}", vault.address);

    // Query Electrum for this vault's status
    let backup = VaultBackup {
        version: 1,
        network: "testnet".into(),
        owner_pubkey: hex::encode(owner_pk.serialize()),
        cosigner_pubkey: hex::encode(cosigner_pk.serialize()),
        chain_code: "ab".repeat(32),
        address_index: 0,
        timelock_blocks: 1,
        threshold: 1,
        heirs: vec![HeirBackupEntry {
            label: "TestHeir".into(),
            xpub: "xpub661MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet8".into(),
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

    let json = serde_json::to_string(&backup).unwrap();
    let status = nostring_heir_ffi::api::fetch_vault_status(
        json,
        "ssl://electrum.blockstream.info:60002".into(),
    )
    .unwrap();

    println!("Balance: {} sats", status.balance_sat);
    println!("UTXOs: {}", status.utxo_count);
    println!("Height: {}", status.current_height);
    println!("Eligible: {}", status.eligible);
}
