<script lang="ts">
  import { vault, error, navigate, loading } from '$lib/stores';
  import { importVaultBackup } from '$lib/wasm-bridge';

  let jsonInput = $state('');

  function handleImport() {
    if (!jsonInput.trim()) {
      error.set('Please paste your vault backup JSON');
      return;
    }

    loading.set(true);
    try {
      const info = importVaultBackup(jsonInput.trim());
      vault.set(info);
      error.set(null);
      navigate('status');
    } catch (e: any) {
      error.set(e.message || 'Failed to import vault backup');
    }
    loading.set(false);
  }

  function handleFile(event: Event) {
    const input = event.target as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;

    const reader = new FileReader();
    reader.onload = () => {
      jsonInput = reader.result as string;
    };
    reader.readAsText(file);
  }
</script>

<div class="screen">
  <h2>Import Vault Backup</h2>
  <p class="info">
    Your vault backup was shared by the vault owner. It contains the information
    needed to claim your inheritance when the timelock expires.
  </p>

  <textarea
    bind:value={jsonInput}
    placeholder='Paste vault backup JSON here...'
    rows="8"
  ></textarea>

  <div class="actions">
    <button class="btn primary" onclick={handleImport}>
      Import Backup
    </button>

    <label class="btn secondary file-btn">
      📄 Load from File
      <input type="file" accept=".json,.txt" onchange={handleFile} hidden />
    </label>
  </div>
</div>

<style>
  .screen { display: flex; flex-direction: column; gap: 1rem; }
  h2 { margin: 0; font-size: 1.4rem; }
  .info { color: #999; font-size: 0.9rem; margin: 0; line-height: 1.5; }

  textarea {
    width: 100%;
    background: #1a1a1a;
    color: #e0e0e0;
    border: 1px solid #333;
    border-radius: 8px;
    padding: 0.75rem;
    font-family: 'SF Mono', 'Fira Code', monospace;
    font-size: 0.85rem;
    resize: vertical;
    box-sizing: border-box;
  }

  textarea:focus { border-color: #f7931a; outline: none; }

  .actions { display: flex; gap: 0.75rem; flex-wrap: wrap; }

  .btn {
    padding: 0.75rem 1.5rem;
    border: none;
    border-radius: 6px;
    font-size: 0.95rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s;
  }

  .btn.primary { background: #f7931a; color: #000; flex: 1; }
  .btn.primary:hover { background: #f9a84d; }
  .btn.secondary { background: #333; color: #e0e0e0; }
  .btn.secondary:hover { background: #444; }
  .file-btn { text-align: center; }
</style>
