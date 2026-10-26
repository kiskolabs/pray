import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  renameSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, it } from "node:test";
import { PrayError } from "./errors.js";
import { ensureGitRepository } from "./git/cache.js";
import { gitSourceCacheDirectory } from "./git/paths.js";
import { pinnedShallowCache, runGit } from "./git-catalog.support.js";

describe("pinned git revision fetch", () => {
  it("fetches a locked revision missing from a shallow cache", () => {
    const fixture = pinnedShallowCache();
    try {
      const result = ensureGitRepository(
        fixture.root,
        fixture.cloneUrl,
        false,
        fixture.pinned,
      );
      assert.equal(result.revision, fixture.pinned);
      const parent = spawnSync("git", [
        "-C",
        fixture.db,
        "cat-file",
        "-e",
        `${fixture.pinned}^`,
      ]);
      assert.equal(parent.status, 0);
    } finally {
      fixture.cleanup();
    }
  });

  it("refuses that fetch when offline", () => {
    const fixture = pinnedShallowCache();
    try {
      assert.throws(
        () =>
          ensureGitRepository(
            fixture.root,
            fixture.cloneUrl,
            false,
            fixture.pinned,
            undefined,
            true,
          ),
        (error: unknown) => {
          assert.ok(error instanceof PrayError);
          assert.match(error.message, new RegExp(fixture.pinned));
          assert.match(error.message, /offline/);
          assert.equal(error.message.includes("--locked"), false);
          return true;
        },
      );
    } finally {
      fixture.cleanup();
    }
  });

  it("refuses to clone when offline and the cache is missing", () => {
    const root = mkdtempSync(join(tmpdir(), "pray-offline-git-clone-"));
    const previousCache = process.env.PRAY_CACHE;
    process.env.PRAY_CACHE = join(root, "global-cache");
    try {
      const origin = join(root, "origin");
      mkdirSync(origin, { recursive: true });
      writeFileSync(join(origin, "catalog.txt"), "one\n");
      runGit(origin, "init", "--template=", "-b", "main");
      runGit(origin, "config", "user.name", "pray");
      runGit(origin, "config", "user.email", "pray@example.com");
      runGit(origin, "add", "-A");
      runGit(
        origin,
        "-c",
        "commit.gpgsign=false",
        "-c",
        "core.hooksPath=/dev/null",
        "commit",
        "-m",
        "one",
      );
      const cloneUrl = `file://${origin}`;
      assert.throws(
        () =>
          ensureGitRepository(
            root,
            cloneUrl,
            false,
            undefined,
            undefined,
            true,
          ),
        (error: unknown) => {
          assert.ok(error instanceof PrayError);
          assert.match(error.message, /offline/);
          assert.match(error.message, /not cached/);
          return true;
        },
      );
      assert.equal(existsSync(gitSourceCacheDirectory(root, cloneUrl)), false);
    } finally {
      if (previousCache === undefined) delete process.env.PRAY_CACHE;
      else process.env.PRAY_CACHE = previousCache;
      rmSync(root, { recursive: true, force: true });
    }
  });

  it("seeds from the global cache when offline and the origin is gone", () => {
    const root = mkdtempSync(join(tmpdir(), "pray-offline-git-seed-"));
    const previousCache = process.env.PRAY_CACHE;
    process.env.PRAY_CACHE = join(root, "global-cache");
    try {
      const origin = join(root, "origin");
      mkdirSync(join(origin, "v1/packages"), { recursive: true });
      writeFileSync(join(origin, "v1/packages/sample.json"), "{}\n");
      runGit(origin, "init", "--template=", "-b", "main");
      runGit(origin, "config", "user.name", "pray");
      runGit(origin, "config", "user.email", "pray@example.com");
      runGit(origin, "add", "-A");
      runGit(
        origin,
        "-c",
        "commit.gpgsign=false",
        "-c",
        "core.hooksPath=/dev/null",
        "commit",
        "-m",
        "one",
      );
      const cloneUrl = `file://${origin}`;
      const first = ensureGitRepository(root, cloneUrl, false);
      rmSync(gitSourceCacheDirectory(root, cloneUrl), {
        recursive: true,
        force: true,
      });
      const moved = join(root, "origin-away");
      renameSync(origin, moved);
      const seeded = ensureGitRepository(
        root,
        cloneUrl,
        false,
        undefined,
        undefined,
        true,
      );
      assert.equal(seeded.revision, first.revision);
    } finally {
      if (previousCache === undefined) delete process.env.PRAY_CACHE;
      else process.env.PRAY_CACHE = previousCache;
      rmSync(root, { recursive: true, force: true });
    }
  });
});
