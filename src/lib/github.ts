import { invoke } from "@tauri-apps/api/core";
import type { RepoChoice, StaleSession } from "./types";

/**
 * Typed wrappers over the Rust GitHub client (src-tauri/src/github.rs).
 * All network traffic happens in the Rust layer — the ecosystem pattern —
 * and the webview never holds the token.
 */

export interface ConnectResult {
  login: string;
  /** False when the keychain refused to store a newly entered token. */
  stored: boolean;
}

/** Errors from the Rust side carry { kind, status, message }. */
export function isNoTokenError(e: unknown): boolean {
  return (e as { kind?: string } | null)?.kind === "no-token";
}

const repoArgs = (r?: RepoChoice) =>
  r ? { owner: r.owner, repo: r.repo, defaultBranch: r.defaultBranch } : {};

export const session = {
  /**
   * Connect to a library with its .env or keychain token; rejects with kind
   * "no-token" when neither works. Omitting the choice uses the configured default.
   */
  connect: (r?: RepoChoice) => invoke<ConnectResult>("session_connect", repoArgs(r)),
  /** First-run (once per library): validate a pasted token, keep it in the keychain, connect. */
  connectWithToken: (token: string, r?: RepoChoice) =>
    invoke<ConnectResult>("session_connect_token", { token, ...repoArgs(r) }),
};

export const gh = {
  loadTree: (docsRoot: string) =>
    invoke<{ files: string[]; truncated: boolean }>("gh_load_tree", { docsRoot }),

  readFile: (path: string, gitRef?: string) =>
    invoke<{ content: string; sha: string }>("gh_read_file", { path, gitRef: gitRef ?? null }),

  /** The file's blob sha at a ref, or null if it does not exist there. */
  fileSha: (path: string, gitRef: string) =>
    invoke<string | null>("gh_file_sha", { path, gitRef }),

  createBranch: (branch: string) => invoke<void>("gh_create_branch", { branch }),

  /** `sha` is the file's current blob sha on the branch; null for a new file's first save. */
  saveFile: (opts: { branch: string; path: string; content: string; message: string; sha: string | null }) =>
    invoke<{ sha: string }>("gh_save_file", opts),

  createPullRequest: (opts: { branch: string; title: string; body: string }) =>
    invoke<{ number: number; url: string }>("gh_create_pr", opts),

  deleteBranch: (branch: string) => invoke<void>("gh_delete_branch", { branch }),

  findStaleSessions: (branchPrefix: string) =>
    invoke<StaleSession[]>("gh_find_stale_sessions", { branchPrefix }),
};
