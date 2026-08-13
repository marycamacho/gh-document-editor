<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import FirstRun from "./components/FirstRun.svelte";
  import Sidebar from "./components/Sidebar.svelte";
  import PreviewPane from "./components/PreviewPane.svelte";
  import EditorPane from "./components/EditorPane.svelte";
  import NewDocDialog from "./components/NewDocDialog.svelte";
  import ResumeDialog from "./components/ResumeDialog.svelte";
  import Modal from "./components/Modal.svelte";
  import Toasts from "./components/Toasts.svelte";
  import { loadConfig, ConfigError } from "./lib/config";
  import { gh, session as ghAuth, isNoTokenError } from "./lib/github";
  import { toFriendlyError } from "./lib/errors";
  import { renderMarkdown } from "./lib/markdown";
  import { buildTree, folderPaths, joinPath } from "./lib/tree";
  import {
    commitMessage,
    editBranchName,
    newDocBranchName,
    prBody,
    prTitle,
    type EditAction,
  } from "./lib/naming";
  import * as db from "./lib/localdb";
  import { toast } from "./lib/toast.svelte";
  import type { AppConfig, EditSession, RepoChoice, StaleSession } from "./lib/types";

  type Screen = "loading" | "config-error" | "first-run" | "tree" | "editor" | "submitted";

  const storage = window.localStorage;

  let screen = $state<Screen>("loading");
  let config = $state<AppConfig | null>(null);
  let currentRepo = $state<RepoChoice | null>(null);
  let showRepoMenu = $state(false);
  let displayName = $state("");
  let configErrorMessage = $state("");
  let firstRunError = $state("");
  let connecting = $state(false);

  // Tree screen
  let files = $state<string[]>([]);
  const treeNodes = $derived(buildTree(files));
  const folders = $derived(folderPaths(files));
  let selectedPath = $state<string | null>(null);
  let previewHtml = $state("");
  let previewLoading = $state(false);
  let previewContent = "";
  let previewSha = "";
  let openingEditor = $state(false);
  let showNewDialog = $state(false);
  let creatingDoc = $state(false);

  // Editor screen
  let session = $state<EditSession | null>(null);
  let buffer = $state("");
  let savedContent = $state("");
  let lastSavedAt = $state<Date | null>(null);
  let saving = $state(false);
  let submitting = $state(false);
  const dirty = $derived(buffer !== savedContent);
  let draftTimer: ReturnType<typeof setTimeout> | undefined;

  // Resume / submitted / close-guard
  let staleSessions = $state<StaleSession[]>([]);
  let staleBusyBranch = $state<string | null>(null);
  let prUrl = $state("");
  let showCloseGuard = $state(false);

  let online = $state(navigator.onLine);

  const repoLabel = $derived(
    currentRepo
      ? `${currentRepo.owner}/${currentRepo.repo}`
      : config
        ? `${config.repoOwner}/${config.repoName}`
        : "",
  );

  // ── Startup ────────────────────────────────────────────────

  async function init() {
    try {
      config = await loadConfig();
    } catch (e) {
      configErrorMessage =
        e instanceof ConfigError ? e.message : "The app couldn't read its configuration.";
      screen = "config-error";
      return;
    }
    currentRepo = config.repos[0];
    await connectCurrent();
  }

  // The Rust shell resolves the current library's token (.env, then its
  // keychain entry) and connects; the token itself never reaches this side.
  async function connectCurrent(): Promise<void> {
    screen = "loading";
    try {
      const result = await ghAuth.connect(currentRepo!);
      await afterConnect(result.login);
    } catch (e) {
      // "no-token" (including a revoked sign-in) lands on the sign-in screen
      // with no error; anything else shows its plain-language message there.
      if (!isNoTokenError(e)) {
        firstRunError = toFriendlyError(e).message;
      }
      screen = "first-run";
    }
  }

  async function switchRepo(choice: RepoChoice): Promise<void> {
    showRepoMenu = false;
    if (currentRepo && choice.owner === currentRepo.owner && choice.repo === currentRepo.repo) {
      return;
    }
    if (screen === "editor") {
      toast("Finish or close your current edit before switching libraries.", "error");
      return;
    }
    currentRepo = choice;
    files = [];
    selectedPath = null;
    previewHtml = "";
    staleSessions = [];
    firstRunError = "";
    prUrl = "";
    await connectCurrent();
  }

  // The sign-in screen reports the browser approval finished; the token file
  // now exists, so a normal connect completes the flow.
  async function handleSignedIn(typedName: string): Promise<void> {
    connecting = true;
    firstRunError = "";
    try {
      const result = await ghAuth.connect(currentRepo!);
      await afterConnect(result.login, typedName);
    } catch (e) {
      firstRunError = toFriendlyError(e).message;
      screen = "first-run";
    } finally {
      connecting = false;
    }
  }

  async function afterConnect(who: string, typedName?: string): Promise<void> {
    displayName = typedName || config!.displayName || storage.getItem("displayName") || who;
    storage.setItem("displayName", displayName);
    await refreshTree();
    screen = "tree";
    void checkStaleSessions();
  }

  async function refreshTree() {
    const { files: paths, truncated } = await gh.loadTree(config!.docsRoot);
    files = paths;
    if (truncated) {
      toast("This library is very large — some documents may be missing from the list.", "error");
    }
  }

  async function checkStaleSessions() {
    try {
      staleSessions = await gh.findStaleSessions(config!.branchPrefix);
    } catch {
      // Non-critical: the resume prompt just doesn't appear this launch.
      staleSessions = [];
    }
  }

  // ── Tree screen ────────────────────────────────────────────

  async function selectFile(path: string) {
    selectedPath = path;
    previewLoading = true;
    try {
      const { content, sha } = await gh.readFile(path);
      previewContent = content;
      previewSha = sha;
      previewHtml = renderMarkdown(content);
    } catch (e) {
      previewHtml = "";
      toast(toFriendlyError(e).message, "error");
    } finally {
      previewLoading = false;
    }
  }

  function filenameOf(path: string): string {
    return path.split("/").pop() ?? path;
  }

  async function startEdit() {
    if (!selectedPath) return;
    openingEditor = true;
    const filename = filenameOf(selectedPath);
    const branch = editBranchName(config!.branchPrefix, filename, new Date());
    try {
      await gh.createBranch(branch);
      openSession(
        {
          branch,
          path: selectedPath,
          filename,
          isNew: false,
          fileSha: previewSha,
          baseFileSha: previewSha,
          commitCount: 0,
          startedAt: new Date().toISOString(),
        },
        previewContent,
        previewContent,
      );
    } catch (e) {
      toast(toFriendlyError(e).message, "error");
    } finally {
      openingEditor = false;
    }
  }

  async function createNewDoc(opts: { title: string; folder: string; filename: string }) {
    creatingDoc = true;
    const path = joinPath(opts.folder, opts.filename);
    const branch = newDocBranchName(config!.branchPrefix, opts.filename, new Date());
    try {
      await gh.createBranch(branch);
      showNewDialog = false;
      openSession(
        {
          branch,
          path,
          filename: opts.filename,
          isNew: true,
          fileSha: null,
          baseFileSha: null,
          commitCount: 0,
          startedAt: new Date().toISOString(),
        },
        `# ${opts.title}\n`,
        // Nothing is committed yet, so the pre-filled heading counts as unsaved.
        "",
      );
    } catch (e) {
      toast(toFriendlyError(e).message, "error");
    } finally {
      creatingDoc = false;
    }
  }

  function openSession(s: EditSession, initialBuffer: string, initialSaved: string) {
    session = s;
    buffer = initialBuffer;
    savedContent = initialSaved;
    lastSavedAt = null;
    db.saveSession(storage, s);
    screen = "editor";
  }

  // ── Editor screen ──────────────────────────────────────────

  function onBufferChange(content: string) {
    buffer = content;
    // The 2s debounce to local storage is what makes a crash lose at most a couple of seconds.
    clearTimeout(draftTimer);
    const branch = session!.branch;
    draftTimer = setTimeout(() => db.saveDraft(storage, branch, content), 2000);
  }

  async function doSave(): Promise<boolean> {
    const s = session!;
    saving = true;
    const content = buffer;
    const action: EditAction = s.isNew && s.commitCount === 0 ? "create" : "update";
    try {
      let result;
      try {
        result = await gh.saveFile({
          branch: s.branch,
          path: s.path,
          content,
          message: commitMessage(action, s.filename, displayName, new Date()),
          sha: s.fileSha,
        });
      } catch (e) {
        // Blob-sha mismatch on our own branch: refetch the sha and retry once.
        if (toFriendlyError(e).kind !== "conflict") throw e;
        const freshSha = await gh.fileSha(s.path, s.branch);
        result = await gh.saveFile({
          branch: s.branch,
          path: s.path,
          content,
          message: commitMessage(action, s.filename, displayName, new Date()),
          sha: freshSha,
        });
      }
      session = { ...s, fileSha: result.sha, commitCount: s.commitCount + 1 };
      db.saveSession(storage, session);
      savedContent = content;
      lastSavedAt = new Date();
      toast("Saved ✓");
      return true;
    } catch (e) {
      toast(toFriendlyError(e).message, "error");
      return false;
    } finally {
      saving = false;
    }
  }

  async function doSubmit() {
    submitting = true;
    try {
      if (buffer !== savedContent) {
        const ok = await doSave();
        if (!ok) return;
      }
      const current = session!;
      let conflictNote = false;
      if (!current.isNew && current.baseFileSha) {
        const mainSha = await gh.fileSha(current.path, config!.defaultBranch).catch(() => null);
        conflictNote = mainSha !== null && mainSha !== current.baseFileSha;
      }
      const action: EditAction = current.isNew ? "create" : "update";
      const pr = await gh.createPullRequest({
        branch: current.branch,
        title: prTitle(action, current.filename, displayName),
        body: prBody({
          path: current.path,
          startedAt: new Date(current.startedAt),
          endedAt: new Date(),
          commitCount: current.commitCount,
          conflictNote,
        }),
      });
      db.removeSession(storage, current.branch);
      prUrl = pr.url;
      session = null;
      screen = "submitted";
      void refreshTree();
    } catch (e) {
      toast(toFriendlyError(e).message, "error");
    } finally {
      submitting = false;
    }
  }

  async function discardSession() {
    const s = session!;
    if (s.commitCount > 0) return;
    try {
      await gh.deleteBranch(s.branch);
    } catch (e) {
      toast(toFriendlyError(e).message, "error");
      return;
    }
    db.removeSession(storage, s.branch);
    session = null;
    backToTree();
  }

  function backToTree() {
    screen = "tree";
    prUrl = "";
    if (selectedPath) void selectFile(selectedPath);
  }

  // ── Resume flow ────────────────────────────────────────────

  async function resumeStale(s: StaleSession) {
    if (!s.path) return;
    staleBusyBranch = s.branch;
    try {
      const record = db.findSession(storage, s.branch);
      const { content, sha } = await gh.readFile(s.path, s.branch);
      const draft = db.loadDraft(storage, s.branch);
      const isNew = record?.isNew ?? s.branch.startsWith(`${config!.branchPrefix}new-`);
      openSession(
        {
          branch: s.branch,
          path: s.path,
          filename: filenameOf(s.path),
          isNew,
          fileSha: sha,
          baseFileSha: record?.baseFileSha ?? (isNew ? null : await gh.fileSha(s.path, config!.defaultBranch)),
          commitCount: s.commitCount,
          startedAt: record?.startedAt ?? new Date().toISOString(),
        },
        draft ?? content,
        content,
      );
      staleSessions = [];
    } catch (e) {
      toast(toFriendlyError(e).message, "error");
    } finally {
      staleBusyBranch = null;
    }
  }

  async function submitStale(s: StaleSession) {
    staleBusyBranch = s.branch;
    try {
      const isNew = s.branch.startsWith(`${config!.branchPrefix}new-`);
      const filename = s.path ? filenameOf(s.path) : s.branch.slice(config!.branchPrefix.length);
      const pr = await gh.createPullRequest({
        branch: s.branch,
        title: prTitle(isNew ? "create" : "update", filename, displayName),
        body: prBody({
          path: s.path ?? "(see the changed files)",
          startedAt: new Date(),
          endedAt: new Date(),
          commitCount: s.commitCount,
        }),
      });
      db.removeSession(storage, s.branch);
      staleSessions = staleSessions.filter((x) => x.branch !== s.branch);
      prUrl = pr.url;
      screen = "submitted";
    } catch (e) {
      toast(toFriendlyError(e).message, "error");
    } finally {
      staleBusyBranch = null;
    }
  }

  async function discardStale(s: StaleSession) {
    staleBusyBranch = s.branch;
    try {
      await gh.deleteBranch(s.branch);
      db.removeSession(storage, s.branch);
      staleSessions = staleSessions.filter((x) => x.branch !== s.branch);
    } catch (e) {
      toast(toFriendlyError(e).message, "error");
    } finally {
      staleBusyBranch = null;
    }
  }

  // ── Window close guard ─────────────────────────────────────

  $effect(() => {
    const appWindow = getCurrentWindow();
    const unlisten = appWindow.onCloseRequested((event) => {
      if (screen === "editor" && dirty && !showCloseGuard) {
        event.preventDefault();
        showCloseGuard = true;
      }
    });
    return () => {
      void unlisten.then((fn) => fn());
    };
  });

  async function closeGuardSave() {
    const ok = await doSave();
    if (ok) void getCurrentWindow().destroy();
    else showCloseGuard = false;
  }

  function closeGuardDiscard() {
    // The draft buffer stays in local storage, so even "Discard" loses nothing permanently.
    void getCurrentWindow().destroy();
  }

  // ── Misc ───────────────────────────────────────────────────

  function openPr() {
    if (prUrl) void openUrl(prUrl);
  }

  $effect(() => {
    const on = () => (online = true);
    const off = () => (online = false);
    window.addEventListener("online", on);
    window.addEventListener("offline", off);
    return () => {
      window.removeEventListener("online", on);
      window.removeEventListener("offline", off);
    };
  });

  void init();
</script>

<main>
  {#if !online && screen !== "loading"}
    <div class="offline-banner">
      You're offline — you can keep typing, and save again once you're reconnected.
    </div>
  {/if}

  {#if screen === "loading"}
    <div class="center"><p class="muted">Starting up…</p></div>
  {:else if screen === "config-error"}
    <div class="center">
      <div class="config-error">
        <h1>Almost there</h1>
        <p>{configErrorMessage}</p>
      </div>
    </div>
  {:else if screen === "first-run"}
    <FirstRun
      {repoLabel}
      initialName={config?.displayName || storage.getItem("displayName") || ""}
      errorMessage={firstRunError}
      {connecting}
      onsignedin={(name) => void handleSignedIn(name)}
    />
  {:else if screen === "tree"}
    <div class="split">
      <Sidebar
        nodes={treeNodes}
        {selectedPath}
        onselect={(p) => void selectFile(p)}
        onnew={() => (showNewDialog = true)}
      />
      <PreviewPane
        path={selectedPath}
        html={previewHtml}
        loading={previewLoading}
        busy={openingEditor}
        onedit={() => void startEdit()}
      />
    </div>
  {:else if screen === "editor" && session}
    {#key session.branch}
      <EditorPane
        path={session.path}
        initialContent={buffer}
        {dirty}
        {saving}
        {submitting}
        commitCount={session.commitCount}
        {lastSavedAt}
        {online}
        onchange={onBufferChange}
        onsave={() => void doSave()}
        onsubmit={() => void doSubmit()}
        ondiscard={() => void discardSession()}
      />
    {/key}
  {:else if screen === "submitted"}
    <div class="center">
      <div class="submitted">
        <div class="check">✓</div>
        <h1>Submitted for review</h1>
        <p class="muted">Your changes are in — the team will review and publish them.</p>
        <div class="submitted-actions">
          <button onclick={openPr}>View on GitHub</button>
          <button class="primary" onclick={backToTree}>Back to documents</button>
        </div>
      </div>
    </div>
  {/if}

  {#if screen === "tree" || screen === "editor" || screen === "submitted" || screen === "first-run"}
    <footer>
      {#if config && config.repos.length > 1}
        <div class="repo-switch">
          {#if showRepoMenu}
            <button
              class="menu-backdrop"
              aria-label="Close library menu"
              onclick={() => (showRepoMenu = false)}
            ></button>
            <div class="repo-menu" role="menu">
              {#each config.repos as r (`${r.owner}/${r.repo}`)}
                <button class="repo-option" role="menuitem" onclick={() => void switchRepo(r)}>
                  <span class="tick">
                    {currentRepo && r.owner === currentRepo.owner && r.repo === currentRepo.repo
                      ? "✓"
                      : ""}
                  </span>
                  {r.owner}/{r.repo}
                </button>
              {/each}
            </div>
          {/if}
          <button
            class="repo-current"
            title="Switch document library"
            onclick={() => (showRepoMenu = !showRepoMenu)}
          >
            {repoLabel} ▴
          </button>
        </div>
      {:else}
        <span>{repoLabel}</span>
      {/if}
      <span>{displayName}</span>
    </footer>
  {/if}

  {#if showNewDialog}
    <NewDocDialog
      {folders}
      existingPaths={files}
      creating={creatingDoc}
      oncreate={(opts) => void createNewDoc(opts)}
      oncancel={() => (showNewDialog = false)}
    />
  {/if}

  {#if staleSessions.length > 0 && screen === "tree"}
    <ResumeDialog
      sessions={staleSessions}
      busyBranch={staleBusyBranch}
      onresume={(s) => void resumeStale(s)}
      onsubmit={(s) => void submitStale(s)}
      ondiscard={(s) => void discardStale(s)}
      ondismiss={() => (staleSessions = [])}
    />
  {/if}

  {#if showCloseGuard}
    <Modal title="Save before closing?">
      <p class="muted" style="margin-top: 0">
        You have unsaved changes to <strong>{session?.filename}</strong>.
      </p>
      <div class="guard-buttons">
        <button class="quiet" onclick={() => (showCloseGuard = false)}>Cancel</button>
        <button onclick={closeGuardDiscard}>Don't save</button>
        <button class="primary" onclick={() => void closeGuardSave()} disabled={saving}>
          {saving ? "Saving…" : "Save & close"}
        </button>
      </div>
    </Modal>
  {/if}

  <Toasts />
</main>

<style>
  main {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .split {
    flex: 1;
    display: flex;
    min-height: 0;
  }

  .center {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .muted {
    color: var(--text-muted);
  }

  .config-error {
    max-width: 440px;
    text-align: center;
  }

  .config-error h1 {
    font-size: 20px;
  }

  .offline-banner {
    background: #fef3c7;
    color: #92400e;
    text-align: center;
    padding: 6px 12px;
    font-size: 13px;
  }

  .submitted {
    text-align: center;
  }

  .check {
    width: 56px;
    height: 56px;
    border-radius: 50%;
    background: var(--accent-soft);
    color: var(--accent);
    font-size: 30px;
    line-height: 56px;
    margin: 0 auto 12px;
  }

  .submitted h1 {
    font-size: 20px;
    margin: 0 0 6px;
  }

  .submitted-actions {
    display: flex;
    gap: 10px;
    justify-content: center;
    margin-top: 18px;
  }

  .guard-buttons {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    margin-top: 18px;
  }

  footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 6px 14px;
    border-top: 1px solid var(--border);
    font-size: 12px;
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .repo-switch {
    position: relative;
  }

  .repo-current {
    border: none;
    background: none;
    padding: 2px 6px;
    font-size: 12px;
    color: var(--text-muted);
    border-radius: 6px;
  }

  .repo-current:hover {
    color: var(--text);
    background: var(--bg-subtle);
  }

  .menu-backdrop {
    position: fixed;
    inset: 0;
    border: none;
    background: transparent;
    z-index: 30;
    cursor: default;
  }

  .repo-menu {
    position: absolute;
    bottom: calc(100% + 6px);
    left: 0;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    box-shadow: 0 8px 24px rgba(15, 23, 42, 0.18);
    min-width: 280px;
    padding: 4px;
    z-index: 31;
  }

  .repo-option {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    text-align: left;
    border: none;
    background: none;
    padding: 7px 10px;
    font-size: 13px;
    border-radius: 6px;
    white-space: nowrap;
  }

  .repo-option:hover {
    background: var(--bg-subtle);
  }

  .tick {
    width: 14px;
    color: var(--accent);
    flex-shrink: 0;
  }
</style>
