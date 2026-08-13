/** Tiny toast store (Svelte 5 runes). */

export interface Toast {
  id: number;
  text: string;
  kind: "ok" | "error";
}

let nextId = 1;

export const toasts = $state<Toast[]>([]);

export function toast(text: string, kind: Toast["kind"] = "ok"): void {
  const id = nextId++;
  toasts.push({ id, text, kind });
  const ttl = kind === "error" ? 6000 : 2500;
  setTimeout(() => {
    const i = toasts.findIndex((t) => t.id === id);
    if (i >= 0) toasts.splice(i, 1);
  }, ttl);
}
