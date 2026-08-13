<script lang="ts">
  import Modal from "./Modal.svelte";
  import { titleToFilename } from "../lib/naming";
  import { isDuplicatePath, joinPath } from "../lib/tree";

  let {
    folders,
    existingPaths,
    creating,
    oncreate,
    oncancel,
  }: {
    folders: string[];
    existingPaths: string[];
    creating: boolean;
    oncreate: (opts: { title: string; folder: string; filename: string }) => void;
    oncancel: () => void;
  } = $props();

  let title = $state("");
  // svelte-ignore state_referenced_locally — folders are fixed for the dialog's lifetime
  let folder = $state(folders[0] ?? "");
  let filenameEdited = $state(false);
  let filenameManual = $state("");

  const filename = $derived(filenameEdited ? filenameManual : titleToFilename(title));
  const candidatePath = $derived(filename ? joinPath(folder, filename) : "");
  const duplicate = $derived(candidatePath !== "" && isDuplicatePath(candidatePath, existingPaths));
  const filenameValid = $derived(/^[a-z0-9][a-z0-9-]*\.md$/.test(filename));
  const canCreate = $derived(
    title.trim().length > 0 && filenameValid && !duplicate && !creating,
  );

  function editFilename(e: Event) {
    filenameEdited = true;
    filenameManual = (e.target as HTMLInputElement).value.trim();
  }

  function submit(e: Event) {
    e.preventDefault();
    if (canCreate) oncreate({ title: title.trim(), folder, filename });
  }
</script>

<Modal title="New document">
  <form onsubmit={submit}>
    <div class="field">
      <label for="doc-title">Title</label>
      <!-- svelte-ignore a11y_autofocus — a modal's single purpose is this field -->
      <input id="doc-title" type="text" bind:value={title} autofocus placeholder="e.g. Offboarding Checklist" />
    </div>

    <div class="field">
      <label for="doc-folder">Folder</label>
      <select id="doc-folder" bind:value={folder}>
        {#each folders as f (f)}
          <option value={f}>{f === "" ? "(top level)" : f}</option>
        {/each}
      </select>
    </div>

    <div class="field">
      <label for="doc-filename">Filename</label>
      <input id="doc-filename" type="text" value={filename} oninput={editFilename} spellcheck="false" />
      {#if duplicate}
        <p class="error-text">A doc with that name already exists in this folder.</p>
      {:else if filename && !filenameValid}
        <p class="error-text">Use lowercase letters, numbers and dashes, ending in .md</p>
      {:else}
        <p class="hint">Created automatically from the title — change it if you like.</p>
      {/if}
    </div>

    <div class="buttons">
      <button type="button" class="quiet" onclick={oncancel} disabled={creating}>Cancel</button>
      <button type="submit" class="primary" disabled={!canCreate}>
        {creating ? "Creating…" : "Create"}
      </button>
    </div>
  </form>
</Modal>

<style>
  .buttons {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    margin-top: 18px;
  }
</style>
