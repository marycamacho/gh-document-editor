<script lang="ts">
  import { EditorView, basicSetup } from "codemirror";
  import { markdown } from "@codemirror/lang-markdown";
  import { languages } from "@codemirror/language-data";
  import { renderMarkdown } from "../lib/markdown";
  import { humanStamp } from "../lib/naming";

  let {
    path,
    initialContent,
    dirty,
    saving,
    submitting,
    commitCount,
    lastSavedAt,
    online,
    onchange,
    onsave,
    onsubmit,
    ondiscard,
  }: {
    path: string;
    initialContent: string;
    dirty: boolean;
    saving: boolean;
    submitting: boolean;
    commitCount: number;
    lastSavedAt: Date | null;
    online: boolean;
    onchange: (content: string) => void;
    onsave: () => void;
    onsubmit: () => void;
    ondiscard: () => void;
  } = $props();

  let editorEl: HTMLElement;
  let previewEl = $state<HTMLElement | null>(null);
  let view: EditorView | undefined;
  // svelte-ignore state_referenced_locally — CodeMirror owns the doc after mount; App remounts per session
  let current = $state(initialContent);

  // HackMD-style view modes: source only, side-by-side, or rendered only.
  type ViewMode = "write" | "split" | "preview";
  const MODES: Array<{ id: ViewMode; label: string }> = [
    { id: "write", label: "Write" },
    { id: "split", label: "Split" },
    { id: "preview", label: "Preview" },
  ];
  let viewMode = $state<ViewMode>(
    ((): ViewMode => {
      const saved = localStorage.getItem("viewMode");
      return saved === "split" || saved === "preview" ? saved : "write";
    })(),
  );

  function setMode(mode: ViewMode) {
    viewMode = mode;
    localStorage.setItem("viewMode", mode);
  }

  const showEditor = $derived(viewMode !== "preview");
  const showPreview = $derived(viewMode !== "write");
  const previewHtml = $derived(showPreview ? renderMarkdown(current) : "");
  const busy = $derived(saving || submitting);

  // Split mode: keep the two panes at the same proportional position, in both
  // directions. The guard stops the panes from ping-ponging each other.
  let syncing = false;

  function syncScroll(source: HTMLElement, target: HTMLElement) {
    if (syncing) return;
    syncing = true;
    const ratio = source.scrollTop / Math.max(1, source.scrollHeight - source.clientHeight);
    target.scrollTop = ratio * (target.scrollHeight - target.clientHeight);
    requestAnimationFrame(() => (syncing = false));
  }

  function onPreviewScroll() {
    if (viewMode === "split" && view && previewEl) syncScroll(previewEl, view.scrollDOM);
  }

  $effect(() => {
    const v = new EditorView({
      doc: initialContent,
      extensions: [
        basicSetup,
        markdown({ codeLanguages: languages }),
        EditorView.lineWrapping,
        EditorView.updateListener.of((update) => {
          if (update.docChanged) {
            current = update.state.doc.toString();
            onchange(current);
          }
        }),
      ],
      parent: editorEl,
    });
    view = v;
    v.scrollDOM.addEventListener("scroll", () => {
      if (viewMode === "split" && previewEl) syncScroll(v.scrollDOM, previewEl);
    });
    v.focus();
    return () => {
      view = undefined;
      v.destroy();
    };
  });

  function onkeydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "s") {
      e.preventDefault();
      if (dirty && online && !busy) onsave();
    }
  }

  const saveTooltip = $derived(
    !online ? "You're offline — your text is kept and you can save when you're back" : "",
  );
</script>

<svelte:window {onkeydown} />

<section>
  <header>
    <span class="path" title={path}>
      {path}
      {#if dirty}<span class="dirty" title="Unsaved changes">●</span>{/if}
    </span>
    <div class="actions">
      {#if lastSavedAt}
        <span class="saved-at">Saved {humanStamp(lastSavedAt)}</span>
      {/if}
      <div class="segmented" role="group" aria-label="View mode">
        {#each MODES as mode (mode.id)}
          <button
            class="seg"
            class:active={viewMode === mode.id}
            aria-pressed={viewMode === mode.id}
            onclick={() => setMode(mode.id)}
          >
            {mode.label}
          </button>
        {/each}
      </div>
      <button onclick={onsave} disabled={!dirty || !online || busy} title={saveTooltip}>
        {saving ? "Saving…" : "Save"}
      </button>
      <button class="primary" onclick={onsubmit} disabled={busy || !online || (commitCount === 0 && !dirty)}>
        {submitting ? "Submitting…" : "Close & Submit"}
      </button>
      <button
        class="quiet"
        onclick={ondiscard}
        disabled={commitCount > 0 || busy}
        title={commitCount > 0
          ? "Changes are already saved to your branch — use Close & Submit"
          : ""}
      >
        {commitCount > 0 ? "Discard" : "Close without saving"}
      </button>
    </div>
  </header>

  <div class="body">
    <!-- CodeMirror stays mounted across mode switches so the buffer and cursor survive. -->
    <div class="editor" class:with-preview={viewMode === "split"} bind:this={editorEl} hidden={!showEditor}></div>
    {#if showPreview}
      <div class="preview" bind:this={previewEl} onscroll={onPreviewScroll}>
        <!-- sanitized in renderMarkdown -->
        <div class="markdown-body">{@html previewHtml}</div>
      </div>
    {/if}
  </div>
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

  .dirty {
    color: var(--accent);
    margin-left: 6px;
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }

  .saved-at {
    font-size: 12px;
    color: var(--text-muted);
  }

  .body {
    flex: 1;
    min-height: 0;
    display: flex;
  }

  .segmented {
    display: flex;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    overflow: hidden;
  }

  .seg {
    border: none;
    border-radius: 0;
    padding: 5px 12px;
    font-size: 13px;
    color: var(--text-muted);
  }

  .seg + .seg {
    border-left: 1px solid var(--border);
  }

  .seg.active {
    background: var(--accent-soft);
    color: var(--accent);
    font-weight: 500;
  }

  .editor {
    flex: 1;
    min-width: 0;
    overflow: hidden;
  }

  .editor.with-preview {
    border-right: 1px solid var(--border);
  }

  .editor :global(.cm-editor) {
    height: 100%;
    font-size: 14px;
  }

  .editor :global(.cm-editor.cm-focused) {
    outline: none;
  }

  .editor :global(.cm-scroller) {
    font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
    line-height: 1.55;
  }

  .preview {
    flex: 1;
    min-width: 0;
    overflow-y: auto;
    padding: 20px 24px;
  }
</style>
