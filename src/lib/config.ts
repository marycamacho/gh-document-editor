import { invoke } from "@tauri-apps/api/core";
import type { AppConfig } from "./types";

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
  if (!repoOwner || !repoName) {
    throw new ConfigError(
      "The app doesn't know which document library to open. " +
        "Check that the .env file next to the app has REPO_OWNER and REPO_NAME filled in — " +
        "your team lead can send you a ready-made copy.",
    );
  }

  let branchPrefix = get("BRANCH_PREFIX") || "docs/";
  if (!branchPrefix.endsWith("/")) branchPrefix += "/";

  let docsRoot = get("DOCS_ROOT");
  docsRoot = docsRoot.replace(/^\/+|\/+$/g, "");

  // Note: GITHUB_TOKEN never appears here — the Rust shell strips it before
  // the config crosses into the webview.
  return {
    repoOwner,
    repoName,
    defaultBranch: get("DEFAULT_BRANCH") || "main",
    docsRoot,
    branchPrefix,
    displayName: get("DISPLAY_NAME"),
  };
}

export async function loadConfig(): Promise<AppConfig> {
  const raw = await invoke<Record<string, string>>("load_config");
  return parseConfig(raw);
}
