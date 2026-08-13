import { describe, expect, it } from "vitest";
import { ConfigError, parseConfig } from "./config";

const base = {
  REPO_OWNER: "cirdia-wellness",
  REPO_NAME: "cirdia-documentation",
};

describe("parseConfig", () => {
  it("applies defaults for optional values", () => {
    const c = parseConfig({ ...base });
    expect(c.defaultBranch).toBe("main");
    expect(c.branchPrefix).toBe("docs/");
    expect(c.docsRoot).toBe("");
    expect(c.displayName).toBe("");
  });

  it("throws a person-readable error when the repo is not configured", () => {
    expect(() => parseConfig({})).toThrow(ConfigError);
    expect(() => parseConfig({ REPO_OWNER: "x" })).toThrow(/document library/);
  });

  it("treats .env.example placeholders as absent", () => {
    const c = parseConfig({
      ...base,
      DISPLAY_NAME: "TODO_your_first_name",
    });
    expect(c.displayName).toBe("");
  });

  it("keeps real values", () => {
    const c = parseConfig({
      ...base,
      DISPLAY_NAME: "Ana",
      DEFAULT_BRANCH: "trunk",
    });
    expect(c.displayName).toBe("Ana");
    expect(c.defaultBranch).toBe("trunk");
  });

  it("normalizes branch prefix and docs root", () => {
    const c = parseConfig({ ...base, BRANCH_PREFIX: "edits", DOCS_ROOT: "/docs/" });
    expect(c.branchPrefix).toBe("edits/");
    expect(c.docsRoot).toBe("docs");
  });

  it("trims whitespace", () => {
    const c = parseConfig({ ...base, DISPLAY_NAME: "  Ana  " });
    expect(c.displayName).toBe("Ana");
  });
});
