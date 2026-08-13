<script lang="ts">
  import type { Snippet } from "svelte";

  let { title, children }: { title: string; children: Snippet } = $props();
</script>

<div class="overlay" role="dialog" aria-modal="true" aria-label={title}>
  <div class="card">
    <h2>{title}</h2>
    {@render children()}
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(15, 23, 42, 0.4);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 40;
  }

  .card {
    background: var(--bg);
    border-radius: 12px;
    padding: 22px 24px;
    width: min(480px, calc(100vw - 48px));
    max-height: calc(100vh - 96px);
    overflow-y: auto;
    box-shadow: 0 12px 40px rgba(15, 23, 42, 0.3);
  }

  h2 {
    margin: 0 0 14px;
    font-size: 17px;
  }
</style>
