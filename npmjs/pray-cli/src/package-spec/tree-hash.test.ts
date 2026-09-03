import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { treeHashFromFileBytes } from "./tree-hash.js";

// The tree hash is a cross-implementation contract: the Ruby and Rust CLIs sort
// entries by UTF-8 bytes, so "README.md" precedes "exports/rules.md". Locale
// collation orders them the other way and yields a different hash, which no
// implementation would accept. The expected values are pinned rather than
// recomputed here, so a change to the ordering has to be argued for.
const README: [string, Buffer] = ["README.md", Buffer.from("# demo\n")];
const EXPORT: [string, Buffer] = ["exports/rules.md", Buffer.from("rules\n")];
const BYTE_ORDER_HASH =
  "sha256:7b4684d7808de1237a0b6e204e00fe62a9cca479b8169b4380fc3f2a094bf4d3";
const LOCALE_ORDER_HASH =
  "sha256:70147986a4ea0fa27174064d897abf2b73f5ebf92252c1e877c983b643f0b0d9";

describe("treeHashFromFileBytes", () => {
  it("orders entries by bytes, not by locale collation", () => {
    const hash = treeHashFromFileBytes(new Map([EXPORT, README]));

    assert.equal(hash, BYTE_ORDER_HASH);
    assert.notEqual(hash, LOCALE_ORDER_HASH);
  });

  it("does not depend on insertion order", () => {
    assert.equal(
      treeHashFromFileBytes(new Map([README, EXPORT])),
      treeHashFromFileBytes(new Map([EXPORT, README])),
    );
  });
});
