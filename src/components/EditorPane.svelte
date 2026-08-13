<script lang="ts">
  import { untrack } from "svelte";
  import { EditorView, basicSetup } from "codemirror";
  import { markdown } from "@codemirror/lang-markdown";
  import { languages } from "@codemirror/language-data";
  import { search, openSearchPanel } from "@codemirror/search";
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

  // Split mode: the pane the person is actually interacting with drives the
  // other one. One-directional by construction — the driven pane's own scroll
  // events are ignored, so there is no feedback loop to guard against.
  let activePane: "editor" | "preview" = "editor";

  function syncScroll(source: HTMLElement, target: HTMLElement) {
    const ratio = source.scrollTop / Math.max(1, source.scrollHeight - source.clientHeight);
    target.scrollTop = ratio * (target.scrollHeight - target.clientHeight);
  }

  function onPreviewScroll() {
    if (activePane === "preview" && viewMode === "split" && view && previewEl) {
      syncScroll(previewEl, view.scrollDOM);
    }
  }

  $effect(() => {
    // untrack: this effect must run exactly once per mounted session. The
    // buffer flows back into `initialContent` on every keystroke, and a
    // tracked read here would rebuild the editor per keypress — cursor and
    // scroll snapping to the top of the document (the v0.1.1 editing bug).
    const doc = untrack(() => initialContent);
    const v = new EditorView({
      doc,
      extensions: [
        basicSetup,
        search({ top: true }),
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
      if (activePane === "editor" && untrack(() => viewMode) === "split" && previewEl) {
        syncScroll(v.scrollDOM, previewEl);
      }
    });
    // Interacting with the editor (pointer or keyboard) makes it the driver.
    v.scrollDOM.addEventListener("pointerenter", () => (activePane = "editor"));
    v.dom.addEventListener("focusin", () => (activePane = "editor"));
    v.focus();
    return () => {
      view = undefined;
      v.destroy();
    };
  });

  function openFind() {
    if (!view) return;
    // Search lives in the source editor; make sure it's on screen first.
    if (viewMode === "preview") setMode("split");
    openSearchPanel(view);
  }

  function onkeydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "s") {
      e.preventDefault();
      if (dirty && online && !busy) onsave();
    }
    // CodeMirror handles Mod-F when focused; catch it app-wide so Find works
    // from the preview too.
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "f" && !e.shiftKey && !e.altKey) {
      e.preventDefault();
      openFind();
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
      <button class="quiet" title="Find in document (⌘F / Ctrl+F)" onclick={openFind}>Find</button>
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
      <!-- svelte-ignore a11y_no_static_element_interactions — pointerenter only picks the scroll-sync driver; not an interactive control -->
      <div
        class="preview"
        bind:this={previewEl}
        onscroll={onPreviewScroll}
        onpointerenter={() => (activePane = "preview")}
      >
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

  /* Search panel, restyled to match the app */
  .editor :global(.cm-panels) {
    background: var(--bg-subtle);
    border-bottom: 1px solid var(--border);
    color: var(--text);
  }

  .editor :global(.cm-panel.cm-search) {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px;
    padding: 8px 10px;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    font-size: 13px;
  }

  .editor :global(.cm-panel.cm-search input) {
    font: inherit;
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 4px 8px;
    background: var(--bg);
    width: auto;
  }

  .editor :global(.cm-panel.cm-search input:focus) {
    outline: 2px solid var(--accent);
    outline-offset: -1px;
  }

  .editor :global(.cm-panel.cm-search button) {
    font: inherit;
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 4px 10px;
    background: var(--bg);
    color: var(--text);
    cursor: pointer;
    background-image: none;
  }

  .editor :global(.cm-panel.cm-search button:hover) {
    background: var(--accent-soft);
  }

  .editor :global(.cm-panel.cm-search label) {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 12px;
    color: var(--text-muted);
  }

  .editor :global(.cm-panel.cm-search [name="close"]) {
    border: none;
    background: none;
    font-size: 16px;
    color: var(--text-muted);
    margin-left: auto;
  }

  .editor :global(.cm-searchMatch) {
    background: rgba(15, 118, 110, 0.2);
  }

  .editor :global(.cm-searchMatch-selected) {
    background: rgba(15, 118, 110, 0.45);
  }

  .preview {
    flex: 1;
    min-width: 0;
    overflow-y: auto;
    padding: 20px 24px;
  }
</style>
