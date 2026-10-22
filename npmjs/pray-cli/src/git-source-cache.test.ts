import assert from "node:assert/strict";
import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, it } from "node:test";
import { spawnSync } from "node:child_process";
import {
  gitSourceCacheDirectory,
  gitSourceCachedRepository,
} from "./git/sources.js";
import { globalGitCacheDirectory } from "./git/store.js";

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
    const root = join(tmpdir(), `pray-git-stub-${process.pid}-${Date.now()}`);
    const cloneUrl = "file://repo";
    const previous = process.env.PRAY_CACHE;
    process.env.PRAY_CACHE = join(root, "global-cache");
    try {
      const shared = globalGitCacheDirectory(cloneUrl);
      assert.ok(shared);
      mkdirSync(shared, { recursive: true });
      const init = spawnSync("git", ["init", "--bare", "-b", "main", shared], {
        encoding: "utf8",
      });
      assert.equal(init.status, 0, init.stderr);
      const leftover = gitSourceCacheDirectory(root, cloneUrl, "left");
      mkdirSync(join(leftover, ".git"), { recursive: true });
      const stub = gitSourceCacheDirectory(root, cloneUrl);
      mkdirSync(stub, { recursive: true });
      writeFileSync(join(stub, ".pray-git-dir"), shared);
      assert.equal(gitSourceCachedRepository(root, cloneUrl), shared);
    } finally {
      if (previous === undefined) {
        delete process.env.PRAY_CACHE;
      } else {
        process.env.PRAY_CACHE = previous;
      }
      rmSync(root, { recursive: true, force: true });
    }
  });

  it("rejects a stub .pray-git-dir outside the expected global bare", () => {
    const root = join(tmpdir(), `pray-git-stub-wrong-${process.pid}-${Date.now()}`);
    const cloneUrl = "file://repo";
    const previous = process.env.PRAY_CACHE;
    process.env.PRAY_CACHE = join(root, "global-cache");
    try {
      const foreign = join(root, "foreign-bare");
      mkdirSync(foreign, { recursive: true });
      const init = spawnSync("git", ["init", "--bare", "-b", "main", foreign], {
        encoding: "utf8",
      });
      assert.equal(init.status, 0, init.stderr);
      const stub = gitSourceCacheDirectory(root, cloneUrl);
      mkdirSync(stub, { recursive: true });
      writeFileSync(join(stub, ".pray-git-dir"), foreign);
      assert.equal(gitSourceCachedRepository(root, cloneUrl), undefined);
    } finally {
      if (previous === undefined) {
        delete process.env.PRAY_CACHE;
      } else {
        process.env.PRAY_CACHE = previous;
      }
      rmSync(root, { recursive: true, force: true });
    }
  });
});
