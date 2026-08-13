<script lang="ts">
  let {
    repoLabel,
    initialName,
    errorMessage,
    connecting,
    onconnect,
  }: {
    repoLabel: string;
    initialName: string;
    errorMessage: string;
    connecting: boolean;
    onconnect: (token: string, name: string) => void;
  } = $props();

  let token = $state("");
  // svelte-ignore state_referenced_locally — seed once; the field is the person's to edit after that
  let name = $state(initialName);

  const canConnect = $derived(token.trim().length > 0 && name.trim().length > 0 && !connecting);

  function submit(e: Event) {
    e.preventDefault();
    if (canConnect) onconnect(token.trim(), name.trim());
  }
</script>

<div class="wrap">
  <form class="card" onsubmit={submit}>
    <h1>Docs Editor</h1>
    <p class="sub">Connect to <strong>{repoLabel}</strong> to get started. You'll only do this once.</p>

    {#if errorMessage}
      <p class="error-text">{errorMessage}</p>
    {/if}

    <div class="field">
      <label for="token">Your GitHub token</label>
      <input
        id="token"
        type="password"
        placeholder="github_pat_…"
        bind:value={token}
        autocomplete="off"
        spellcheck="false"
      />
      <p class="hint">
        From the setup guide your team lead sent you. It's stored safely in your computer's keychain
        — never in a file.
      </p>
    </div>

    <div class="field">
      <label for="name">Your name</label>
      <input id="name" type="text" placeholder="e.g. Ana" bind:value={name} />
      <p class="hint">Shown in the edit history so the team knows who changed what.</p>
    </div>

    <button class="primary" type="submit" disabled={!canConnect}>
      {connecting ? "Connecting…" : "Connect"}
    </button>
  </form>
</div>

<style>
  .wrap {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-subtle);
  }

  .card {
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 28px 30px;
    width: min(440px, calc(100vw - 48px));
  }

  h1 {
    margin: 0 0 4px;
    font-size: 20px;
  }

  .sub {
    color: var(--text-muted);
    margin: 0 0 18px;
  }

  button {
    width: 100%;
    margin-top: 6px;
  }
</style>
