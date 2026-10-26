import assert from "node:assert/strict";
import { describe, it } from "node:test";
import {
  checksumManagedBodyLineRefs,
  checksumManagedSpanContent,
  sha256Hex,
} from "./hashing.js";

describe("hashing", () => {
  it("checksum body line refs matches joined content", () => {
    const joined = checksumManagedSpanContent("alpha\nbeta\n\n");
    const refs = checksumManagedBodyLineRefs(["alpha", "beta", "", ""]);
    assert.equal(joined, refs);
  });

  it("matches NIST SHA-256 test vectors", () => {
    assert.equal(
      sha256Hex(""),
      "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    );
    assert.equal(
      sha256Hex("abc"),
      "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
    );
  });
});
