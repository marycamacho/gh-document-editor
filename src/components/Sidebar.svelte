<script lang="ts">
  import type { TreeNode } from "../lib/types";
  import TreeItem from "./TreeItem.svelte";

  let {
    nodes,
    selectedPath,
    onselect,
    onnew,
  }: {
    nodes: TreeNode[];
    selectedPath: string | null;
    onselect: (path: string) => void;
    onnew: () => void;
  } = $props();
</script>

<aside>
  <div class="top">
    <button class="primary new-doc" onclick={onnew}>+ New document</button>
  </div>
  <nav>
    {#if nodes.length === 0}
      <p class="empty">No documents found.</p>
    {:else}
      {#each nodes as node (node.path)}
        <TreeItem {node} depth={0} {selectedPath} {onselect} />
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
  }

  .new-doc {
    width: 100%;
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
