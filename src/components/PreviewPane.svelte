<script lang="ts">
  let {
    path,
    html,
    loading,
    busy,
    onedit,
  }: {
    path: string | null;
    html: string;
    loading: boolean;
    busy: boolean;
    onedit: () => void;
  } = $props();
</script>

<section>
  {#if path === null}
    <div class="placeholder">
      <p>Pick a document on the left to read it, or create a new one.</p>
    </div>
  {:else}
    <header>
      <span class="path" title={path}>{path}</span>
      <button class="primary" onclick={onedit} disabled={loading || busy}>
        {busy ? "Opening…" : "Edit"}
      </button>
    </header>
    <div class="content">
      {#if loading}
        <p class="loading">Loading…</p>
      {:else}
        <!-- eslint-disable-next-line svelte/no-at-html-tags — sanitized in renderMarkdown -->
        <div class="markdown-body">{@html html}</div>
      {/if}
    </div>
  {/if}
</section>

<style>
  section {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    /* Without this the pane grows past the window and nothing can scroll. */
    min-height: 0;
  }

  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 10px 18px;
    border-bottom: 1px solid var(--border);
  }

  .path {
    font-size: 13px;
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .content {
    flex: 1;
    overflow-y: auto;
    padding: 20px 24px;
  }

  .placeholder {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
  }

  .loading {
    color: var(--text-muted);
  }
</style>
