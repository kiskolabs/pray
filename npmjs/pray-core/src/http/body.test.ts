import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { PrayError } from "../errors.js";
import { readBoundedHttpBody } from "./body.js";

function responseWithBody(body: string, contentLength?: number): Response {
  const headers = new Headers();
  if (contentLength !== undefined) {
    headers.set("content-length", String(contentLength));
  }
  return new Response(body, { status: 200, headers });
}

describe("bounded HTTP bodies", () => {
  it("accepts bodies within the ceiling and rejects oversized bodies", async () => {
    const accepted = await readBoundedHttpBody(responseWithBody("ok"), 8);
    assert.equal(accepted.toString("utf8"), "ok");

    await assert.rejects(
      () => readBoundedHttpBody(responseWithBody("too-large"), 4),
      (error: unknown) =>
        error instanceof PrayError &&
        error.kind === "resolution" &&
        error.message.includes("HTTP response exceeds"),
    );

    await assert.rejects(
      () => readBoundedHttpBody(responseWithBody("ok", 64), 8),
      (error: unknown) =>
        error instanceof PrayError &&
        error.message.includes("HTTP response exceeds"),
    );
  });
});
