import { writable } from 'svelte/store';
import type { VaultInfo, ClaimEligibility } from './wasm-bridge';

export type Screen = 'import' | 'status' | 'claim';

export const currentScreen = writable<Screen>('import');
export const vault = writable<VaultInfo | null>(null);
export const eligibility = writable<ClaimEligibility | null>(null);
export const error = writable<string | null>(null);
export const loading = writable(false);

export function navigate(screen: Screen) {
	currentScreen.set(screen);
	error.set(null);
}
