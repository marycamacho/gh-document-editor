import { describe, expect, it } from "vitest";
import { toFriendlyError } from "./errors";

describe("toFriendlyError", () => {
  it("maps 401 to the expired-token message", () => {
    const e = toFriendlyError({ status: 401, message: "Bad credentials" });
    expect(e.kind).toBe("auth");
    expect(e.message).toMatch(/token/);
  });

  it("maps 404 to the no-access message", () => {
    const e = toFriendlyError({ status: 404, message: "Not Found" });
    expect(e.kind).toBe("no-access");
    expect(e.message).toMatch(/Ask Mary/);
  });

  it("maps 409 and 422 to conflict", () => {
    expect(toFriendlyError({ status: 409 }).kind).toBe("conflict");
    expect(toFriendlyError({ status: 422 }).kind).toBe("conflict");
  });

  it("maps 429 to rate-limit", () => {
    expect(toFriendlyError({ status: 429 }).kind).toBe("rate-limit");
  });

  it("maps 403 with a rate-limit message to rate-limit", () => {
    const e = toFriendlyError({
      status: 403,
      message: "API rate limit exceeded for user",
    });
    expect(e.kind).toBe("rate-limit");
  });

  it("maps a fetch TypeError to offline", () => {
    const e = toFriendlyError(new TypeError("Failed to fetch"));
    expect(e.kind).toBe("offline");
  });

  it("maps the Rust shell's network kind to offline", () => {
    const e = toFriendlyError({ kind: "network", message: "error sending request" });
    expect(e.kind).toBe("offline");
  });

  it("falls back to unknown, and never loses the text", () => {
    const e = toFriendlyError({ status: 500, message: "boom" });
    expect(e.kind).toBe("unknown");
    expect(e.message).toMatch(/text is safe/);
  });
});
