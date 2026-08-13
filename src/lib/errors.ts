/** Map raw GitHub / network failures to the plain-language messages in docs/spec.md §8. */

export type FriendlyErrorKind =
  | "auth"
  | "no-access"
  | "offline"
  | "rate-limit"
  | "conflict"
  | "unknown";

export interface FriendlyError {
  kind: FriendlyErrorKind;
  message: string;
}

/** The shape the Rust shell's ApiError serializes to (src-tauri/src/github.rs). */
interface RawApiError {
  kind?: string;
  status?: number;
  message?: string;
}

function isRateLimited(e: RawApiError): boolean {
  return e.status === 429 || (e.status === 403 && /rate limit/i.test(e.message ?? ""));
}

export function toFriendlyError(err: unknown): FriendlyError {
  const e = (err ?? {}) as RawApiError;

  // A request that never got an HTTP status means the network is down or unreachable.
  const networkFailure =
    e.kind === "network" ||
    ((err instanceof TypeError || /fetch failed|failed to fetch|network|connect/i.test(e.message ?? "")) &&
      e.status === undefined);
  const explicitlyOffline = typeof navigator !== "undefined" && navigator.onLine === false;
  if (networkFailure || explicitlyOffline) {
    return {
      kind: "offline",
      message: "You appear to be offline. Your text is safe — reconnect and try again.",
    };
  }

  switch (e.status) {
    case 401:
      return {
        kind: "auth",
        message: "Your token has expired or was revoked — you'll need to make a new one.",
      };
    case 404:
      return {
        kind: "no-access",
        message:
          "We can't reach the document library. Ask Mary to check you've been added to the repo.",
      };
    case 409:
    case 422:
      return {
        kind: "conflict",
        message: "The document changed on GitHub while saving. Your text is safe — trying again usually fixes it.",
      };
  }

  if (isRateLimited(e)) {
    return {
      kind: "rate-limit",
      message: "GitHub is asking us to slow down — try again in a minute.",
    };
  }

  return {
    kind: "unknown",
    message: "Something went wrong talking to GitHub. Your text is safe — try again.",
  };
}
