import assert from "node:assert/strict";
import { mkdirSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, it } from "node:test";
import {
  gitSourceCacheDirectory,
  gitSourceCachedRepository,
} from "./git/sources.js";

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

  it("prefers the URL-only cache when that checkout exists", () => {
    const root = join(tmpdir(), `pray-git-cached-${process.pid}`);
    const cloneUrl = "file://repo";
    const shared = gitSourceCacheDirectory(root, cloneUrl);
    mkdirSync(join(shared, ".git"), { recursive: true });
    try {
      assert.equal(gitSourceCachedRepository(root, cloneUrl), shared);
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });
});
