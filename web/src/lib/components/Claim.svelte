<script lang="ts">
  import { vault, error, navigate } from '$lib/stores';
  import { validateAddress } from '$lib/wasm-bridge';

  let destinationAddress = $state('');
  let addressValid = $state<boolean | null>(null);

  function checkAddress() {
    if (!destinationAddress.trim() || !$vault) {
      addressValid = null;
      return;
    }

    try {
      validateAddress(destinationAddress.trim(), $vault.network);
      addressValid = true;
    } catch {
      addressValid = false;
    }
  }

  function handleClaim() {
    if (!addressValid) {
      error.set('Please enter a valid Bitcoin address');
      return;
    }
    // Future: build PSBT, sign, broadcast
    error.set('Claim functionality coming soon. PSBT building is in development.');
  }
</script>

<div class="screen">
  <h2>Claim Bitcoin</h2>

  <p class="info">
    Enter the Bitcoin address where you want to receive your inheritance.
    This can be any address you control, such as a wallet, exchange, or CashApp address.
  </p>

  <div class="input-group">
    <label for="dest">Destination Address</label>
    <input
      id="dest"
      type="text"
      bind:value={destinationAddress}
      oninput={checkAddress}
      placeholder="bc1q... or tb1q..."
      class:valid={addressValid === true}
      class:invalid={addressValid === false}
    />
    {#if addressValid === true}
      <span class="validation ok">✅ Valid address</span>
    {:else if addressValid === false}
      <span class="validation bad">❌ Invalid address for {$vault?.network}</span>
    {/if}
  </div>

  <button class="btn primary" onclick={handleClaim} disabled={!addressValid}>
    Build Claim Transaction
  </button>

  <button class="btn secondary" onclick={() => navigate('status')}>
    ← Back to Status
  </button>
</div>

<style>
  .screen { display: flex; flex-direction: column; gap: 1rem; }
  h2 { margin: 0; font-size: 1.4rem; }
  .info { color: #999; font-size: 0.9rem; margin: 0; line-height: 1.5; }

  .input-group {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .input-group label { font-size: 0.85rem; color: #aaa; }

  input {
    width: 100%;
    background: #1a1a1a;
    color: #e0e0e0;
    border: 1px solid #333;
    border-radius: 8px;
    padding: 0.75rem;
    font-family: 'SF Mono', 'Fira Code', monospace;
    font-size: 0.85rem;
    box-sizing: border-box;
  }

  input:focus { border-color: #f7931a; outline: none; }
  input.valid { border-color: #2e7d32; }
  input.invalid { border-color: #c62828; }

  .validation { font-size: 0.8rem; }
  .validation.ok { color: #4caf50; }
  .validation.bad { color: #ef5350; }

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
