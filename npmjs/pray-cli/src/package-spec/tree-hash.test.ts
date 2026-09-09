import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { describe, it } from "node:test";
import { fileURLToPath } from "node:url";
import { treeHashFromFileBytes } from "./tree-hash.js";

type TreeHashFixture = {
  files: Array<{ path: string; content: string }>;
  tree_hash: string;
};

const here = dirname(fileURLToPath(import.meta.url));
const fixture = JSON.parse(
  readFileSync(
    join(here, "../../../../testdata/shared/package-tree/byte-order.json"),
    "utf8",
  ),
) as TreeHashFixture;
const files = fixture.files.map(
  ({ path, content }) => [path, Buffer.from(content)] as [string, Buffer],
);

describe("treeHashFromFileBytes", () => {
  it("matches the shared byte-order fixture", () => {
    const hash = treeHashFromFileBytes(new Map(files));

    assert.equal(hash, fixture.tree_hash);
  });

  it("does not depend on insertion order", () => {
    assert.equal(
      treeHashFromFileBytes(new Map(files)),
      treeHashFromFileBytes(new Map([...files].reverse())),
    );
  });
});
