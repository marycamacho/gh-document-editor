import { describe, expect, it } from "vitest";
import {
  branchStamp,
  commitMessage,
  editBranchName,
  fileSlug,
  humanStamp,
  newDocBranchName,
  prBody,
  prTitle,
  titleToFilename,
} from "./naming";

// Local-time date: 2026-08-13 10:42
const d = new Date(2026, 7, 13, 10, 42);

describe("fileSlug", () => {
  it("lowercases, drops the extension, and dashes non-alphanumerics", () => {
    expect(fileSlug("Onboarding Guide.md")).toBe("onboarding-guide");
    expect(fileSlug("onboarding-guide.md")).toBe("onboarding-guide");
    expect(fileSlug("Q3 (draft) — notes.md")).toBe("q3-draft-notes");
  });

  it("only drops a trailing .md, not interior dots", () => {
    expect(fileSlug("v1.2-release.md")).toBe("v1-2-release");
  });

  it("trims leading/trailing dashes", () => {
    expect(fileSlug("--weird--.md")).toBe("weird");
  });
});

describe("titleToFilename", () => {
  it("slugs the title and appends .md", () => {
    expect(titleToFilename("Offboarding Checklist")).toBe("offboarding-checklist.md");
  });

  it("returns empty string when nothing usable remains", () => {
    expect(titleToFilename("!!!")).toBe("");
    expect(titleToFilename("")).toBe("");
  });
});

describe("timestamps", () => {
  it("branchStamp is YYYYMMDD-HHmm local", () => {
    expect(branchStamp(d)).toBe("20260813-1042");
  });

  it("humanStamp is YYYY-MM-DD HH:mm local", () => {
    expect(humanStamp(d)).toBe("2026-08-13 10:42");
  });

  it("pads single digits", () => {
    expect(branchStamp(new Date(2026, 0, 5, 9, 7))).toBe("20260105-0907");
  });
});

describe("branch names", () => {
  it("edit branches follow docs/<file-slug>-<stamp>", () => {
    expect(editBranchName("docs/", "onboarding-guide.md", d)).toBe("docs/onboarding-guide-20260813-1042");
  });

  it("new-doc branches follow docs/new-<file-slug>-<stamp>", () => {
    expect(newDocBranchName("docs/", "offboarding-checklist.md", d)).toBe(
      "docs/new-offboarding-checklist-20260813-1042",
    );
  });
});

describe("commit messages and PR titles", () => {
  it("matches the spec examples", () => {
    expect(commitMessage("update", "onboarding-guide.md", "Ana", new Date(2026, 7, 13, 10, 47))).toBe(
      "Update onboarding-guide.md — Ana, 2026-08-13 10:47",
    );
    expect(commitMessage("create", "offboarding-checklist.md", "Ana", new Date(2026, 7, 13, 11, 15))).toBe(
      "Create offboarding-checklist.md — Ana, 2026-08-13 11:15",
    );
    expect(prTitle("update", "onboarding-guide.md", "Ana")).toBe("Docs: update onboarding-guide.md (Ana)");
    expect(prTitle("create", "offboarding-checklist.md", "Ana")).toBe(
      "Docs: new offboarding-checklist.md (Ana)",
    );
  });
});

describe("prBody", () => {
  it("carries path, session times, and commit count", () => {
    const body = prBody({
      path: "Company/onboarding-guide.md",
      startedAt: d,
      endedAt: new Date(2026, 7, 13, 11, 2),
      commitCount: 3,
    });
    expect(body).toContain("File: `Company/onboarding-guide.md`");
    expect(body).toContain("2026-08-13 10:42 → 2026-08-13 11:02");
    expect(body).toContain("Commits: 3");
    expect(body).not.toContain("conflict");
  });

  it("adds the conflict note when asked", () => {
    const body = prBody({ path: "a.md", startedAt: d, endedAt: d, commitCount: 1, conflictNote: true });
    expect(body).toContain("may need conflict resolution");
  });
});
