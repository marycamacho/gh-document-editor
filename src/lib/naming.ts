/** Branch, commit, and PR naming — the conventions in docs/spec.md §3. */

function slugify(text: string): string {
  return text
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
}

/** Filename lowercased, extension dropped, non-alphanumerics → "-". */
export function fileSlug(filename: string): string {
  return slugify(filename.replace(/\.md$/i, ""));
}

/** "Quarterly Goals!" → "quarterly-goals.md"; "" when nothing usable remains. */
export function titleToFilename(title: string): string {
  const slug = slugify(title);
  return slug ? `${slug}.md` : "";
}

const pad = (n: number) => String(n).padStart(2, "0");

/** YYYYMMDD-HHmm in local time. */
export function branchStamp(d: Date): string {
  return `${d.getFullYear()}${pad(d.getMonth() + 1)}${pad(d.getDate())}-${pad(d.getHours())}${pad(d.getMinutes())}`;
}

/** YYYY-MM-DD HH:mm in local time. */
export function humanStamp(d: Date): string {
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

export function editBranchName(prefix: string, filename: string, d: Date): string {
  return `${prefix}${fileSlug(filename)}-${branchStamp(d)}`;
}

export function newDocBranchName(prefix: string, filename: string, d: Date): string {
  return `${prefix}new-${fileSlug(filename)}-${branchStamp(d)}`;
}

export type EditAction = "create" | "update";

export function commitMessage(action: EditAction, filename: string, name: string, d: Date): string {
  const verb = action === "create" ? "Create" : "Update";
  return `${verb} ${filename} — ${name}, ${humanStamp(d)}`;
}

export function prTitle(action: EditAction, filename: string, name: string): string {
  return action === "create" ? `Docs: new ${filename} (${name})` : `Docs: update ${filename} (${name})`;
}

export function prBody(opts: {
  path: string;
  startedAt: Date;
  endedAt: Date;
  commitCount: number;
  conflictNote?: boolean;
}): string {
  const lines = [
    `File: \`${opts.path}\``,
    `Edit session: ${humanStamp(opts.startedAt)} → ${humanStamp(opts.endedAt)}`,
    `Commits: ${opts.commitCount}`,
  ];
  if (opts.conflictNote) {
    lines.push("", "Note: this file also changed on the default branch during the edit session — may need conflict resolution.");
  }
  return lines.join("\n");
}
