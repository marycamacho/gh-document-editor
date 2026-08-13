import { beforeEach, describe, expect, it } from "vitest";
import type { EditSession } from "./types";
import {
  clearDraft,
  findSession,
  listSessions,
  loadDraft,
  removeSession,
  saveDraft,
  saveSession,
  type StorageLike,
} from "./localdb";

function memoryStorage(): StorageLike {
  const map = new Map<string, string>();
  return {
    getItem: (k) => map.get(k) ?? null,
    setItem: (k, v) => void map.set(k, v),
    removeItem: (k) => void map.delete(k),
  };
}

const session: EditSession = {
  branch: "docs/guide-20260813-1042",
  path: "Company/guide.md",
  filename: "guide.md",
  isNew: false,
  fileSha: "abc",
  baseFileSha: "abc",
  commitCount: 0,
  startedAt: "2026-08-13T10:42:00.000Z",
};

let storage: StorageLike;
beforeEach(() => {
  storage = memoryStorage();
});

describe("sessions", () => {
  it("saves, finds, and lists sessions", () => {
    saveSession(storage, session);
    expect(listSessions(storage)).toHaveLength(1);
    expect(findSession(storage, session.branch)?.path).toBe("Company/guide.md");
  });

  it("updates in place by branch", () => {
    saveSession(storage, session);
    saveSession(storage, { ...session, commitCount: 2 });
    expect(listSessions(storage)).toHaveLength(1);
    expect(findSession(storage, session.branch)?.commitCount).toBe(2);
  });

  it("removeSession also clears the draft", () => {
    saveSession(storage, session);
    saveDraft(storage, session.branch, "typed text");
    removeSession(storage, session.branch);
    expect(listSessions(storage)).toHaveLength(0);
    expect(loadDraft(storage, session.branch)).toBeNull();
  });

  it("survives corrupted storage", () => {
    storage.setItem("sessions", "{not json");
    expect(listSessions(storage)).toEqual([]);
  });
});

describe("drafts", () => {
  it("round-trips draft content", () => {
    saveDraft(storage, "b", "# hello");
    expect(loadDraft(storage, "b")).toBe("# hello");
    clearDraft(storage, "b");
    expect(loadDraft(storage, "b")).toBeNull();
  });
});
