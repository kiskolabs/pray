import assert from "node:assert/strict";
import { mkdirSync, rmSync, writeFileSync } from "node:fs";
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

  it("ignores a leftover full clone when the shared bare is missing", () => {
    const root = join(tmpdir(), `pray-git-cached-${process.pid}`);
    const cloneUrl = "file://repo";
    const leftover = gitSourceCacheDirectory(root, cloneUrl, "left");
    mkdirSync(join(leftover, ".git"), { recursive: true });
    try {
      assert.equal(gitSourceCachedRepository(root, cloneUrl), undefined);
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });

  it("prefers a stub .pray-git-dir over a leftover full clone", () => {
    const root = join(tmpdir(), `pray-git-stub-${process.pid}`);
    const cloneUrl = "file://repo";
    const objects = join(root, "objects");
    mkdirSync(objects, { recursive: true });
    writeFileSync(join(objects, "HEAD"), "ref: refs/heads/main\n");
    const leftover = gitSourceCacheDirectory(root, cloneUrl, "left");
    mkdirSync(join(leftover, ".git"), { recursive: true });
    const stub = gitSourceCacheDirectory(root, cloneUrl);
    mkdirSync(stub, { recursive: true });
    writeFileSync(join(stub, ".pray-git-dir"), objects);
    try {
      assert.equal(gitSourceCachedRepository(root, cloneUrl), objects);
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });
});
