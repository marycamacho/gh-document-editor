import type { EditSession } from "./types";

/**
 * Local persistence: edit sessions and draft buffers, keyed by branch.
 * Storage is injected so the module is testable; the app passes window.localStorage.
 * The draft buffer is what makes "typed text is never lost" true across a crash.
 */
export interface StorageLike {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
  removeItem(key: string): void;
}

const SESSIONS_KEY = "sessions";
const draftKey = (branch: string) => `draft:${branch}`;

export function listSessions(storage: StorageLike): EditSession[] {
  try {
    const raw = storage.getItem(SESSIONS_KEY);
    return raw ? (JSON.parse(raw) as EditSession[]) : [];
  } catch {
    return [];
  }
}

export function saveSession(storage: StorageLike, session: EditSession): void {
  const sessions = listSessions(storage).filter((s) => s.branch !== session.branch);
  sessions.push(session);
  storage.setItem(SESSIONS_KEY, JSON.stringify(sessions));
}

export function removeSession(storage: StorageLike, branch: string): void {
  const sessions = listSessions(storage).filter((s) => s.branch !== branch);
  storage.setItem(SESSIONS_KEY, JSON.stringify(sessions));
  storage.removeItem(draftKey(branch));
}

export function findSession(storage: StorageLike, branch: string): EditSession | null {
  return listSessions(storage).find((s) => s.branch === branch) ?? null;
}

export function saveDraft(storage: StorageLike, branch: string, content: string): void {
  storage.setItem(draftKey(branch), content);
}

export function loadDraft(storage: StorageLike, branch: string): string | null {
  return storage.getItem(draftKey(branch));
}

export function clearDraft(storage: StorageLike, branch: string): void {
  storage.removeItem(draftKey(branch));
}
