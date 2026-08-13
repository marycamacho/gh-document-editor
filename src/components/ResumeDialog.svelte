<script lang="ts">
  import Modal from "./Modal.svelte";
  import type { StaleSession } from "../lib/types";

  let {
    sessions,
    busyBranch,
    onresume,
    onsubmit,
    ondiscard,
    ondismiss,
  }: {
    sessions: StaleSession[];
    busyBranch: string | null;
    onresume: (s: StaleSession) => void;
    onsubmit: (s: StaleSession) => void;
    ondiscard: (s: StaleSession) => void;
    ondismiss: () => void;
  } = $props();

  // Discard throws away saved commits — ask twice.
  let confirmingDiscard = $state<string | null>(null);

  const label = (s: StaleSession) => s.path ?? s.branch;
</script>

<Modal title="You have unsubmitted edits">
  <p class="intro">
    These changes were saved but never submitted for review. You can pick up where you left off,
    submit them as they are, or throw them away.
  </p>

  <ul>
    {#each sessions as s (s.branch)}
      <li>
        <div class="what">
          <span class="file" title={s.branch}>{label(s)}</span>
          <span class="meta">{s.commitCount} saved {s.commitCount === 1 ? "change" : "changes"}</span>
        </div>
        <div class="row-actions">
          <button
            class="primary"
            disabled={busyBranch !== null || s.path === null}
            title={s.path === null ? "Couldn't work out which file this edit belongs to — submit or discard it" : ""}
            onclick={() => onresume(s)}
          >
            {busyBranch === s.branch ? "…" : "Resume"}
          </button>
          <button disabled={busyBranch !== null} onclick={() => onsubmit(s)}>Submit</button>
          {#if confirmingDiscard === s.branch}
            <button class="danger" disabled={busyBranch !== null} onclick={() => ondiscard(s)}>
              Really discard?
            </button>
          {:else}
            <button class="quiet" disabled={busyBranch !== null} onclick={() => (confirmingDiscard = s.branch)}>
              Discard
            </button>
          {/if}
        </div>
      </li>
    {/each}
  </ul>

  <div class="buttons">
    <button class="quiet" onclick={ondismiss} disabled={busyBranch !== null}>Decide later</button>
  </div>
</Modal>

<style>
  .intro {
    color: var(--text-muted);
    margin: 0 0 14px;
    font-size: 14px;
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  li {
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 12px;
  }

  .what {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    gap: 10px;
    margin-bottom: 10px;
  }

  .file {
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .meta {
    font-size: 13px;
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .row-actions {
    display: flex;
    gap: 8px;
  }

  .danger {
    color: #fff;
    background: var(--danger);
    border-color: var(--danger);
  }

  .buttons {
    display: flex;
    justify-content: flex-end;
    margin-top: 16px;
  }
</style>
