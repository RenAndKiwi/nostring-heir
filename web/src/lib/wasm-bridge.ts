/**
 * WASM bridge — loads the Rust module and exposes typed functions.
 */

import init, {
	import_vault_backup as _import_vault_backup,
	check_eligibility as _check_eligibility,
	validate_address as _validate_address
} from './wasm/nostring_heir_ffi.js';

export interface VaultInfo {
	network: string;
	vault_address: string;
	timelock_blocks: number;
	num_heirs: number;
	threshold: number;
	heir_label: string | null;
	raw_json: string;
}

export interface ClaimEligibility {
	eligible: boolean;
	blocks_remaining: number;
	time_remaining: string;
	current_height: number;
}

let initialized = false;

export async function initWasm(): Promise<void> {
	if (initialized) return;
	await init('/wasm/nostring_heir_ffi_bg.wasm');
	initialized = true;
}

export function importVaultBackup(json: string): VaultInfo {
	const result = _import_vault_backup(json);
	return JSON.parse(result);
}

export function checkEligibility(
	vault: VaultInfo,
	currentBlockHeight: number,
	vaultConfirmedHeight: number
): ClaimEligibility {
	const vaultJson = JSON.stringify(vault);
	const result = _check_eligibility(vaultJson, currentBlockHeight, vaultConfirmedHeight);
	return JSON.parse(result);
}

export function validateAddress(address: string, network: string): boolean {
	return _validate_address(address, network);
}
