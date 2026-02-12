<script lang="ts">
  import { vault, eligibility, error, navigate, loading } from '$lib/stores';
  import { checkEligibility } from '$lib/wasm-bridge';
  import { getBlockHeight, getUtxos } from '$lib/esplora';

  let checking = $state(false);

  let progressPct = $derived(
    $eligibility && $vault
      ? Math.max(0, 100 - ($eligibility.blocks_remaining / $vault.timelock_blocks * 100))
      : 0
  );

  async function checkStatus() {
    const v = $vault;
    if (!v) return;

    checking = true;
    error.set(null);

    try {
      const height = await getBlockHeight(v.network);
      const utxos = await getUtxos(v.network, v.vault_address);

      if (utxos.length === 0) {
        error.set('No funds found at vault address. The vault may be empty or not yet confirmed.');
        checking = false;
        return;
      }

      // Use earliest confirmed UTXO height as the vault deposit height
      const confirmedUtxos = utxos.filter(u => u.status.confirmed && u.status.block_height);
      if (confirmedUtxos.length === 0) {
        error.set('Vault has unconfirmed transactions. Please wait for confirmation.');
        checking = false;
        return;
      }

      const depositHeight = Math.min(...confirmedUtxos.map(u => u.status.block_height!));
      const totalSats = utxos.reduce((sum, u) => sum + u.value, 0);

      const result = checkEligibility(v, height, depositHeight);
      eligibility.set(result);

    } catch (e: any) {
      error.set(e.message || 'Failed to check status');
    }
    checking = false;
  }
</script>

<div class="screen">
  <h2>Vault Status</h2>

  {#if $vault}
    <div class="info-card">
      <div class="row">
        <span class="label">Network</span>
        <span class="value">{$vault.network}</span>
      </div>
      <div class="row">
        <span class="label">Vault Address</span>
        <span class="value mono">{$vault.vault_address}</span>
      </div>
      <div class="row">
        <span class="label">Timelock</span>
        <span class="value">{$vault.timelock_blocks.toLocaleString()} blocks (~{Math.round($vault.timelock_blocks * 10 / 1440)} days)</span>
      </div>
      <div class="row">
        <span class="label">Heirs</span>
        <span class="value">{$vault.num_heirs} (threshold: {$vault.threshold})</span>
      </div>
    </div>

    <button class="btn primary" onclick={checkStatus} disabled={checking}>
      {checking ? 'Checking...' : '🔍 Check Eligibility'}
    </button>

    {#if $eligibility}
      <div class="status-card" class:eligible={$eligibility.eligible}>
        {#if $eligibility.eligible}
          <h3>✅ Ready to Claim!</h3>
          <p>The timelock has expired. You can now claim your inherited Bitcoin.</p>
          <button class="btn primary" onclick={() => navigate('claim')}>
            Claim Bitcoin →
          </button>
        {:else}
          <h3>⏳ Not Yet Eligible</h3>
          <p>{$eligibility.blocks_remaining.toLocaleString()} blocks remaining ({$eligibility.time_remaining})</p>
          <div class="progress-bar">
            <div class="progress-fill" style="width: {progressPct}%"></div>
          </div>
          <p class="hint">The vault owner must miss their check-in for the timelock to expire.</p>
        {/if}
      </div>
    {/if}
  {/if}

  <button class="btn secondary" onclick={() => navigate('import')}>
    ← Back
  </button>
</div>

<style>
  .screen { display: flex; flex-direction: column; gap: 1rem; }
  h2 { margin: 0; font-size: 1.4rem; }
  h3 { margin: 0 0 0.5rem; }

  .info-card {
    background: #1a1a1a;
    border: 1px solid #333;
    border-radius: 8px;
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .row { display: flex; justify-content: space-between; align-items: flex-start; gap: 0.5rem; }
  .label { font-size: 0.8rem; color: #888; white-space: nowrap; }
  .value { font-size: 0.9rem; text-align: right; }
  .mono {
    font-family: 'SF Mono', 'Fira Code', monospace;
    font-size: 0.75rem;
    word-break: break-all;
    color: #f7931a;
  }

  .status-card {
    background: #2d2a0d;
    border: 1px solid #5c5a1a;
    border-radius: 8px;
    padding: 1.25rem;
  }

  .status-card.eligible {
    background: #0d2818;
    border-color: #1a5c2e;
  }

  .status-card p { margin: 0.25rem 0; color: #ccc; font-size: 0.9rem; }
  .hint { font-size: 0.8rem !important; color: #888 !important; margin-top: 0.75rem !important; }

  .progress-bar {
    height: 8px;
    background: #333;
    border-radius: 4px;
    overflow: hidden;
    margin: 0.5rem 0;
  }

  .progress-fill {
    height: 100%;
    background: #f7931a;
    border-radius: 4px;
    transition: width 0.3s;
  }

  .btn {
    padding: 0.75rem 1.5rem;
    border: none;
    border-radius: 6px;
    font-size: 0.95rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s;
  }

  .btn:disabled { opacity: 0.5; cursor: not-allowed; }
  .btn.primary { background: #f7931a; color: #000; }
  .btn.primary:hover:not(:disabled) { background: #f9a84d; }
  .btn.secondary { background: #333; color: #e0e0e0; }
  .btn.secondary:hover { background: #444; }
</style>
