import assert from "node:assert/strict";
import { describe, it } from "node:test";
import {
  checksumManagedBodyLineRefs,
  checksumManagedSpanContent,
} from "./hashing.js";

describe("hashing", () => {
  it("checksum body line refs matches joined content", () => {
    const joined = checksumManagedSpanContent("alpha\nbeta\n\n");
    const refs = checksumManagedBodyLineRefs(["alpha", "beta", "", ""]);
    assert.equal(joined, refs);
  });
});
