<script lang="ts">
  import { onMount } from 'svelte';
  import { currentScreen, error } from '$lib/stores';
  import { initWasm } from '$lib/wasm-bridge';
  import Import from '$lib/components/Import.svelte';
  import Status from '$lib/components/Status.svelte';
  import Claim from '$lib/components/Claim.svelte';

  let wasmReady = $state(false);
  let wasmError = $state<string | null>(null);

  onMount(async () => {
    try {
      await initWasm();
      wasmReady = true;
    } catch (e: any) {
      wasmError = e.message || 'Failed to load WASM module';
    }
  });
</script>

<div class="app">
  <header>
    <h1>🔑 NoString Heir</h1>
    <p class="subtitle">Claim your inherited Bitcoin</p>
  </header>

  {#if wasmError}
    <div class="error-card">
      <p>Failed to initialize: {wasmError}</p>
    </div>
  {:else if !wasmReady}
    <div class="loading">Loading crypto module...</div>
  {:else}
    {#if $error}
      <div class="error-card">
        <p>{$error}</p>
        <button onclick={() => error.set(null)}>Dismiss</button>
      </div>
    {/if}

    {#if $currentScreen === 'import'}
      <Import />
    {:else if $currentScreen === 'status'}
      <Status />
    {:else if $currentScreen === 'claim'}
      <Claim />
    {/if}
  {/if}
</div>

<style>
  :global(body) {
    margin: 0;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    background: #0a0a0a;
    color: #e0e0e0;
    min-height: 100vh;
  }

  .app {
    max-width: 480px;
    margin: 0 auto;
    padding: 1.5rem;
  }

  header {
    text-align: center;
    margin-bottom: 2rem;
  }

  header h1 {
    font-size: 1.8rem;
    margin: 0;
    color: #f7931a;
  }

  .subtitle {
    color: #888;
    margin: 0.25rem 0 0;
    font-size: 0.95rem;
  }

  .loading {
    text-align: center;
    color: #888;
    padding: 3rem 0;
  }

  .error-card {
    background: #2d0d0d;
    border: 1px solid #5c1a1a;
    border-radius: 8px;
    padding: 1rem;
    margin-bottom: 1rem;
  }

  .error-card p { margin: 0 0 0.5rem; }

  .error-card button {
    background: #444;
    color: #e0e0e0;
    border: none;
    border-radius: 4px;
    padding: 0.35rem 0.75rem;
    cursor: pointer;
    font-size: 0.85rem;
  }
</style>
