/** Repo configuration, resolved from .env / process env by the Rust shell. */
export interface AppConfig {
  repoOwner: string;
  repoName: string;
  defaultBranch: string;
  /** Subtree to show; "" means the whole repo. */
  docsRoot: string;
  /** Always ends with "/". */
  branchPrefix: string;
  /** May be "" until the person supplies one (first-run screen). */
  displayName: string;
}

export interface TreeNode {
  name: string;
  path: string;
  type: "dir" | "file";
  children: TreeNode[];
}

/** One editing session = one branch. Persisted locally so drafts survive a crash. */
export interface EditSession {
  branch: string;
  path: string;
  filename: string;
  isNew: boolean;
  /** Current blob sha of the file on the branch; null until a new file's first save. */
  fileSha: string | null;
  /** Blob sha of the file on the default branch when the session started (null for new files). */
  baseFileSha: string | null;
  commitCount: number;
  /** ISO timestamp. */
  startedAt: string;
}

export interface StaleSession {
  branch: string;
  /** Path of the edited file, when it could be determined. */
  path: string | null;
  commitCount: number;
}
