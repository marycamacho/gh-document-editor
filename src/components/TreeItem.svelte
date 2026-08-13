<script lang="ts">
  import type { TreeNode } from "../lib/types";
  import TreeItem from "./TreeItem.svelte";

  let {
    node,
    depth,
    selectedPath,
    onselect,
  }: {
    node: TreeNode;
    depth: number;
    selectedPath: string | null;
    onselect: (path: string) => void;
  } = $props();

  // svelte-ignore state_referenced_locally — depth never changes for a mounted node
  let open = $state(depth === 0);
</script>

{#if node.type === "dir"}
  <button class="row dir" style:padding-left="{10 + depth * 14}px" onclick={() => (open = !open)}>
    <span class="chevron" class:open>▸</span>
    <span class="name">{node.name}</span>
  </button>
  {#if open}
    {#each node.children as child (child.path)}
      <TreeItem node={child} depth={depth + 1} {selectedPath} {onselect} />
    {/each}
  {/if}
{:else}
  <button
    class="row file"
    class:selected={node.path === selectedPath}
    style:padding-left="{26 + depth * 14}px"
    onclick={() => onselect(node.path)}
  >
    <span class="name">{node.name}</span>
  </button>
{/if}

<style>
  .row {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    text-align: left;
    border: none;
    border-radius: 0;
    background: transparent;
    padding-top: 5px;
    padding-bottom: 5px;
    padding-right: 8px;
    font-size: 14px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .row:hover {
    background: var(--bg-subtle);
  }

  .dir {
    font-weight: 500;
  }

  .file.selected {
    background: var(--accent-soft);
    color: var(--accent);
    font-weight: 500;
  }

  .chevron {
    display: inline-block;
    transition: transform 0.12s;
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .chevron.open {
    transform: rotate(90deg);
  }

  .name {
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
