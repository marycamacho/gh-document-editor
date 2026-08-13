import { invoke } from "@tauri-apps/api/core";
import type { AppConfig, RepoChoice } from "./types";

/** Placeholder values from .env.example are treated as absent. */
const PLACEHOLDER = /^TODO_/;

export class ConfigError extends Error {}

/**
 * Turn the raw KEY=VALUE map from the Rust shell into a validated config.
 * Throws ConfigError with a person-readable message when required values are missing.
 */
export function parseConfig(raw: Record<string, string>): AppConfig {
  const get = (key: string): string => {
    const v = (raw[key] ?? "").trim();
    return PLACEHOLDER.test(v) ? "" : v;
  };

  const repoOwner = get("REPO_OWNER");
  const repoName = get("REPO_NAME");
  // Unreachable in normal use — the libraries are baked into the app
  // (src-tauri/src/config.rs DEFAULTS); only a broken dev override gets here.
  if (!repoOwner || !repoName) {
    throw new ConfigError(
      "The app's library configuration is broken — a .env override next to the app is " +
        "clearing REPO_OWNER or REPO_NAME. Remove or fix that file and relaunch.",
    );
  }

  let branchPrefix = get("BRANCH_PREFIX") || "docs/";
  if (!branchPrefix.endsWith("/")) branchPrefix += "/";

  let docsRoot = get("DOCS_ROOT");
  docsRoot = docsRoot.replace(/^\/+|\/+$/g, "");

  const defaultBranch = get("DEFAULT_BRANCH") || "main";
  const primary: RepoChoice = { owner: repoOwner, repo: repoName, defaultBranch };

  // Note: GITHUB_TOKEN never appears here — the Rust shell strips it before
  // the config crosses into the webview.
  return {
    repoOwner,
    repoName,
    defaultBranch,
    repos: parseRepoList(get("REPOS"), primary),
    docsRoot,
    branchPrefix,
    displayName: get("DISPLAY_NAME"),
  };
}

/**
 * Parse the optional REPOS list — comma-separated `owner/repo` entries, each
 * with an optional `@branch` (default main) — into the switcher's choices.
 * The primary repo always comes first; duplicates and malformed entries drop.
 */
export function parseRepoList(raw: string, primary: RepoChoice): RepoChoice[] {
  const choices: RepoChoice[] = [primary];
  for (const entry of raw.split(",")) {
    const trimmed = entry.trim();
    if (!trimmed) continue;
    const [spec, branch] = trimmed.split("@");
    const parts = spec.split("/");
    if (parts.length !== 2 || !parts[0] || !parts[1]) continue;
    const choice: RepoChoice = {
      owner: parts[0].trim(),
      repo: parts[1].trim(),
      defaultBranch: branch?.trim() || "main",
    };
    if (!choices.some((c) => c.owner === choice.owner && c.repo === choice.repo)) {
      choices.push(choice);
    }
  }
  return choices;
}

export async function loadConfig(): Promise<AppConfig> {
  const raw = await invoke<Record<string, string>>("load_config");
  return parseConfig(raw);
}
