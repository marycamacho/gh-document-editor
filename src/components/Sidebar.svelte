<script lang="ts">
  import { buildTree, filterPaths } from "../lib/tree";
  import TreeItem from "./TreeItem.svelte";

  let {
    files,
    selectedPath,
    onselect,
    onnew,
  }: {
    files: string[];
    selectedPath: string | null;
    onselect: (path: string) => void;
    onnew: () => void;
  } = $props();

  let filter = $state("");

  const filtering = $derived(filter.trim().length > 0);
  const nodes = $derived(buildTree(filterPaths(files, filter)));
</script>

<aside>
  <div class="top">
    <button class="primary new-doc" onclick={onnew}>+ New document</button>
    <input
      class="filter"
      type="search"
      placeholder="Find a document…"
      bind:value={filter}
      spellcheck="false"
    />
  </div>
  <nav>
    {#if nodes.length === 0}
      <p class="empty">
        {filtering ? "No documents match." : "No documents found."}
      </p>
    {:else}
      {#each nodes as node (node.path)}
        <TreeItem {node} depth={0} {selectedPath} forceOpen={filtering} {onselect} />
      {/each}
    {/if}
  </nav>
</aside>

<style>
  aside {
    width: 280px;
    flex-shrink: 0;
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    background: var(--bg);
  }

  .top {
    padding: 12px;
    border-bottom: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .new-doc {
    width: 100%;
  }

  .filter {
    font-size: 13px;
    padding: 6px 9px;
  }

  nav {
    overflow-y: auto;
    flex: 1;
    padding: 6px 0;
  }

  .empty {
    color: var(--text-muted);
    font-size: 14px;
    padding: 0 14px;
  }
</style>
