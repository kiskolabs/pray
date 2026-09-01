import assert from "node:assert/strict";
import { Readable } from "node:stream";
import { describe, it } from "node:test";
import { PrayError } from "../errors.js";
import { readRequestBody } from "./index.js";

describe("serve request bodies", () => {
  it("accepts bounded bodies and rejects oversized bodies", async () => {
    const small = Readable.from([Buffer.from("hello")]);
    assert.equal((await readRequestBody(small)).toString("utf8"), "hello");

    const large = Readable.from([Buffer.alloc(17 * 1024 * 1024)]);
    await assert.rejects(
      readRequestBody(large),
      (error: unknown) =>
        error instanceof PrayError &&
        error.message.includes("request body exceeds"),
    );
  });
});
