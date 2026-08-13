<script lang="ts">
  import { toasts } from "../lib/toast.svelte";
</script>

<div class="toasts">
  {#each toasts as t (t.id)}
    <div class="toast {t.kind}">{t.text}</div>
  {/each}
</div>

<style>
  .toasts {
    position: fixed;
    bottom: 44px;
    right: 16px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    z-index: 50;
  }

  .toast {
    background: var(--text);
    color: #fff;
    border-radius: var(--radius);
    padding: 9px 14px;
    font-size: 14px;
    max-width: 380px;
    box-shadow: 0 4px 16px rgba(15, 23, 42, 0.25);
  }

  .toast.error {
    background: var(--danger);
  }
</style>
