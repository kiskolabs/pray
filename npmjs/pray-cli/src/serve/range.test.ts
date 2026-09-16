import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { applyByteRange } from "./range.js";

describe("serve byte range", () => {
  it("returns partial content for a satisfiable range", () => {
    const result = applyByteRange(200, Buffer.from("abcdefgh"), "bytes=2-4");
    assert.equal(result.status, 206);
    assert.equal(result.body.toString(), "cde");
    assert.equal(result.contentRange, "bytes 2-4/8");
  });

  it("rejects an unsatisfiable range", () => {
    const result = applyByteRange(200, Buffer.from("ab"), "bytes=5-9");
    assert.equal(result.status, 416);
    assert.equal(result.body.length, 0);
    assert.equal(result.contentRange, "bytes */2");
  });

  it("returns partial content when the range starts at the first byte", () => {
    const result = applyByteRange(200, Buffer.from("abcdefgh"), "bytes=0-2");
    assert.equal(result.status, 206);
    assert.equal(result.body.toString(), "abc");
    assert.equal(result.contentRange, "bytes 0-2/8");
  });
});
