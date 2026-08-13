import type { TreeNode } from "./types";

/**
 * Build a nested folder tree from flat .md file paths (relative to the docs root).
 * Folders sort before files; both alphabetically, case-insensitive.
 */
export function buildTree(paths: string[]): TreeNode[] {
  const root: TreeNode = { name: "", path: "", type: "dir", children: [] };

  for (const path of [...paths].sort((a, b) => a.localeCompare(b))) {
    const parts = path.split("/");
    let node = root;
    for (let i = 0; i < parts.length; i++) {
      const isFile = i === parts.length - 1;
      const childPath = parts.slice(0, i + 1).join("/");
      let child = node.children.find((c) => c.path === childPath);
      if (!child) {
        child = { name: parts[i], path: childPath, type: isFile ? "file" : "dir", children: [] };
        node.children.push(child);
      }
      node = child;
    }
  }

  const sortChildren = (node: TreeNode) => {
    node.children.sort((a, b) => {
      if (a.type !== b.type) return a.type === "dir" ? -1 : 1;
      return a.name.localeCompare(b.name, undefined, { sensitivity: "base" });
    });
    node.children.forEach(sortChildren);
  };
  sortChildren(root);

  return root.children;
}

/** All folder paths in the tree, root ("") included, for the new-document dropdown. */
export function folderPaths(paths: string[]): string[] {
  const folders = new Set<string>([""]);
  for (const path of paths) {
    const parts = path.split("/");
    for (let i = 1; i < parts.length; i++) {
      folders.add(parts.slice(0, i).join("/"));
    }
  }
  return [...folders].sort((a, b) => a.localeCompare(b));
}

/** Case-insensitive substring filter over paths; folder names match too. */
export function filterPaths(paths: string[], query: string): string[] {
  const q = query.trim().toLowerCase();
  if (!q) return paths;
  return paths.filter((p) => p.toLowerCase().includes(q));
}

/** Case-insensitive duplicate check for a candidate path against existing files. */
export function isDuplicatePath(candidate: string, existing: string[]): boolean {
  const lower = candidate.toLowerCase();
  return existing.some((p) => p.toLowerCase() === lower);
}

export function joinPath(folder: string, filename: string): string {
  return folder ? `${folder}/${filename}` : filename;
}
