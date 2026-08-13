<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import Modal from "./Modal.svelte";
  import { auth, type DeviceStart } from "../lib/github";
  import { toFriendlyError } from "../lib/errors";
  import { renderMarkdown } from "../lib/markdown";
  // The setup guide ships inside the app so the link works before any access exists.
  import guideSource from "../../docs/user-guide.md?raw";

  let {
    repoLabel,
    initialName,
    errorMessage,
    connecting,
    onsignedin,
  }: {
    repoLabel: string;
    initialName: string;
    errorMessage: string;
    connecting: boolean;
    onsignedin: (name: string) => void;
  } = $props();

  // svelte-ignore state_referenced_locally — seed once; the field is the person's to edit after that
  let name = $state(initialName);
  let showGuide = $state(false);

  type Phase = "idle" | "starting" | "waiting";
  let phase = $state<Phase>("idle");
  let device = $state<DeviceStart | null>(null);
  let localError = $state("");
  let copied = $state(false);
  let pollTimer: ReturnType<typeof setInterval> | undefined;

  const canSignIn = $derived(name.trim().length > 0 && phase === "idle" && !connecting);
  const shownError = $derived(localError || errorMessage);

  function stopPolling() {
    clearInterval(pollTimer);
    pollTimer = undefined;
  }

  $effect(() => () => stopPolling());

  async function startSignIn() {
    localError = "";
    phase = "starting";
    try {
      device = await auth.start();
      phase = "waiting";
      pollTimer = setInterval(() => void pollOnce(), Math.max(device.interval, 5) * 1000);
    } catch (e) {
      localError = toFriendlyError(e).message;
      phase = "idle";
    }
  }

  async function pollOnce() {
    try {
      const outcome = await auth.poll();
      if (outcome === "pending") return;
      stopPolling();
      if (outcome === "connected") {
        onsignedin(name.trim());
        return;
      }
      phase = "idle";
      localError =
        outcome === "denied"
          ? "The sign-in was cancelled in the browser — try again when you're ready."
          : "That code expired before it was used — click the button for a fresh one.";
    } catch (e) {
      stopPolling();
      phase = "idle";
      localError = toFriendlyError(e).message;
    }
  }

  function openGithub() {
    if (device) void openUrl(device.verificationUri);
  }

  async function copyCode() {
    if (!device) return;
    await navigator.clipboard.writeText(device.userCode);
    copied = true;
    setTimeout(() => (copied = false), 1500);
  }

  function cancelSignIn() {
    stopPolling();
    phase = "idle";
    device = null;
  }
</script>

<div class="wrap">
  <div class="card">
    <h1>Docs Editor</h1>
    <p class="sub">
      Sign in with your GitHub account to open <strong>{repoLabel}</strong>. You'll only do this
      once — you stay signed in after that.
    </p>

    {#if shownError}
      <p class="error-text">{shownError}</p>
    {/if}

    {#if phase !== "waiting"}
      <div class="field">
        <label for="name">Your name</label>
        <input id="name" type="text" placeholder="e.g. Ana" bind:value={name} />
        <p class="hint">Shown in the edit history so the team knows who changed what.</p>
      </div>

      <button class="primary sign-in" onclick={() => void startSignIn()} disabled={!canSignIn}>
        {phase === "starting" || connecting ? "One moment…" : "Sign in with GitHub"}
      </button>

      <p class="hint center-hint">
        First time?
        <button type="button" class="link" onclick={() => (showGuide = true)}>
          Read the 2-minute setup guide</button>
        — all you need is a GitHub account with access to the documents.
      </p>
    {:else if device}
      <div class="device">
        <p class="step">1 · Copy this code</p>
        <button class="code" title="Click to copy" onclick={() => void copyCode()}>
          {device.userCode}
          <span class="copy-note">{copied ? "Copied ✓" : "click to copy"}</span>
        </button>
        <p class="step">2 · Enter it on GitHub and click Authorize</p>
        <button class="primary" onclick={openGithub}>Open GitHub in your browser</button>
        <p class="hint center-hint">
          Or go to <strong>{device.verificationUri}</strong> yourself.
        </p>
        <p class="waiting">Waiting for you to approve in the browser…</p>
        <button type="button" class="link" onclick={cancelSignIn}>Cancel</button>
      </div>
    {/if}
  </div>
</div>

{#if showGuide}
  <Modal title="Setup guide">
    <div class="guide markdown-body">
      <!-- sanitized in renderMarkdown -->
      {@html renderMarkdown(guideSource)}
    </div>
    <div class="guide-close">
      <button class="primary" onclick={() => (showGuide = false)}>Done</button>
    </div>
  </Modal>
{/if}

<style>
  .wrap {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-subtle);
  }

  .card {
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 28px 30px;
    width: min(460px, calc(100vw - 48px));
  }

  h1 {
    margin: 0 0 4px;
    font-size: 20px;
  }

  .sub {
    color: var(--text-muted);
    margin: 0 0 18px;
  }

  .sign-in {
    width: 100%;
    margin-top: 6px;
  }

  .center-hint {
    text-align: center;
    margin-top: 12px;
  }

  .link {
    border: none;
    background: none;
    padding: 0;
    color: var(--accent);
    text-decoration: underline;
    font-size: inherit;
    cursor: pointer;
  }

  .link:hover {
    background: none;
    color: var(--accent-hover);
  }

  .device {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
  }

  .step {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-muted);
    margin: 4px 0 0;
  }

  .code {
    font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
    font-size: 28px;
    letter-spacing: 3px;
    padding: 12px 22px;
    border: 1px dashed var(--accent);
    border-radius: var(--radius);
    background: var(--accent-soft);
    color: var(--accent);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
  }

  .copy-note {
    font-family: inherit;
    font-size: 11px;
    letter-spacing: normal;
    color: var(--text-muted);
  }

  .waiting {
    color: var(--text-muted);
    font-size: 13px;
    margin: 8px 0 0;
  }

  .guide {
    font-size: 14px;
  }

  .guide-close {
    display: flex;
    justify-content: flex-end;
    margin-top: 14px;
  }
</style>
