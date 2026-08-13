import { describe, expect, it } from "vitest";
import { buildTree, folderPaths, isDuplicatePath, joinPath } from "./tree";

const paths = [
  "README.md",
  "Company/onboarding.md",
  "Company/Policies/leave.md",
  "Brand/voice.md",
];

describe("buildTree", () => {
  it("nests files under their folders", () => {
    const tree = buildTree(paths);
    const company = tree.find((n) => n.name === "Company");
    expect(company?.type).toBe("dir");
    expect(company?.children.map((c) => c.name)).toEqual(["Policies", "onboarding.md"]);
    const policies = company?.children.find((c) => c.name === "Policies");
    expect(policies?.children[0].path).toBe("Company/Policies/leave.md");
  });

  it("sorts folders before files, alphabetically", () => {
    const tree = buildTree(paths);
    expect(tree.map((n) => n.name)).toEqual(["Brand", "Company", "README.md"]);
  });

  it("handles an empty list", () => {
    expect(buildTree([])).toEqual([]);
  });
});

describe("folderPaths", () => {
  it("includes the root and every intermediate folder", () => {
    expect(folderPaths(paths)).toEqual(["", "Brand", "Company", "Company/Policies"]);
  });
});

describe("isDuplicatePath", () => {
  it("matches case-insensitively", () => {
    expect(isDuplicatePath("company/ONBOARDING.md", paths)).toBe(true);
    expect(isDuplicatePath("Company/new-doc.md", paths)).toBe(false);
  });
});

describe("joinPath", () => {
  it("joins folder and filename, and handles the root", () => {
    expect(joinPath("Company", "a.md")).toBe("Company/a.md");
    expect(joinPath("", "a.md")).toBe("a.md");
  });
});
