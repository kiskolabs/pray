import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { gitSourceCacheDirectory } from "./git/sources.js";

describe("git source cache directory", () => {
  it("gives different directories to the same URL with different subdirs", () => {
    const root = "/tmp/pray-project";
    const cloneUrl = "file://repo";
    const left = gitSourceCacheDirectory(root, cloneUrl, "left");
    const right = gitSourceCacheDirectory(root, cloneUrl, "right");
    const shared = gitSourceCacheDirectory(root, cloneUrl);
    assert.notEqual(left, right);
    assert.notEqual(left, shared);
    assert.notEqual(right, shared);
  });
});
