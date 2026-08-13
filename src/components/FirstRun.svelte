<script lang="ts">
  import Modal from "./Modal.svelte";
  import { renderMarkdown } from "../lib/markdown";
  // The setup guide ships inside the app so the link works before any access exists.
  import guideSource from "../../docs/user-guide.md?raw";

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
  let showGuide = $state(false);

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
        Don't have a token yet?
        <button type="button" class="link" onclick={() => (showGuide = true)}>
          Open the step-by-step setup guide</button>
        — it walks you through every click, takes about 3 minutes.
      </p>
      <p class="hint">
        Already know your way around tokens? Fine-grained, <strong>this repo only</strong>, two
        repository permissions: <strong>Contents: Read and write</strong> and
        <strong>Pull requests: Read and write</strong> — everything else "No access". Either way
        it's stored safely in your computer's keychain — never in a file.
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

{#if showGuide}
  <Modal title="Setup guide">
    <div class="guide markdown-body">
      <!-- sanitized in renderMarkdown -->
      {@html renderMarkdown(guideSource)}
    </div>
    <div class="guide-close">
      <button class="primary" onclick={() => (showGuide = false)}>Done</button>
    </div>
  </Modal>
{/if}

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

  button[type="submit"] {
    width: 100%;
    margin-top: 6px;
  }

  .link {
    border: none;
    background: none;
    padding: 0;
    color: var(--accent);
    text-decoration: underline;
    font-size: inherit;
    cursor: pointer;
  }

  .link:hover {
    background: none;
    color: var(--accent-hover);
  }

  .guide {
    font-size: 14px;
  }

  .guide-close {
    display: flex;
    justify-content: flex-end;
    margin-top: 14px;
  }
</style>
