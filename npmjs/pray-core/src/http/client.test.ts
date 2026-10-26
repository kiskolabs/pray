import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { httpGet } from "./client.js";

describe("HTTP client errors", () => {
  it("redacts URL userinfo", async () => {
    const originalFetch = globalThis.fetch;
    globalThis.fetch = async () => new Response("refused", { status: 403 });
    try {
      await assert.rejects(
        httpGet("https://user:secret@example.test/catalog"),
        (error: unknown) => {
          assert.ok(error instanceof Error);
          assert.equal(error.message.includes("secret"), false);
          assert.match(
            error.message,
            /https:\/\/\*\*\*@example\.test\/catalog/,
          );
          return true;
        },
      );
    } finally {
      globalThis.fetch = originalFetch;
    }
  });
});
