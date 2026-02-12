/**
 * Esplora REST API client for network operations.
 * All Bitcoin crypto runs in Rust WASM; this handles network I/O only.
 */

export interface Utxo {
	txid: string;
	vout: number;
	value: number;
	status: {
		confirmed: boolean;
		block_height?: number;
		block_time?: number;
	};
}

const ENDPOINTS: Record<string, string> = {
	mainnet: 'https://mempool.space/api',
	bitcoin: 'https://mempool.space/api',
	testnet: 'https://mempool.space/testnet/api',
	signet: 'https://mempool.space/signet/api'
};

function getBase(network: string): string {
	const base = ENDPOINTS[network];
	if (!base) throw new Error(`Unknown network: ${network}`);
	return base;
}

export async function getBlockHeight(network: string): Promise<number> {
	const res = await fetch(`${getBase(network)}/blocks/tip/height`);
	if (!res.ok) throw new Error(`Failed to get block height: ${res.statusText}`);
	return parseInt(await res.text());
}

export async function getUtxos(network: string, address: string): Promise<Utxo[]> {
	const res = await fetch(`${getBase(network)}/address/${address}/utxo`);
	if (!res.ok) throw new Error(`Failed to get UTXOs: ${res.statusText}`);
	return res.json();
}

export async function broadcast(network: string, txHex: string): Promise<string> {
	const res = await fetch(`${getBase(network)}/tx`, {
		method: 'POST',
		body: txHex
	});
	if (!res.ok) {
		const body = await res.text();
		throw new Error(`Broadcast failed: ${body}`);
	}
	return res.text(); // txid
}
