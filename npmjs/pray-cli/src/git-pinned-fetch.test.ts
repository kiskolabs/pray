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

function pinnedShallowCache(): {
  root: string;
  cloneUrl: string;
  pinned: string;
  cleanup: () => void;
} {
  const root = mkdtempSync(join(tmpdir(), "pray-pinned-git-"));
  const previousCache = process.env.PRAY_CACHE;
  process.env.PRAY_CACHE = join(root, "global-cache");
  const origin = join(root, "origin");
  mkdirSync(origin, { recursive: true });
  writeFileSync(join(origin, "catalog.txt"), "one\n");
  runGit(origin, "init", "--template=", "-b", "main");
  runGit(origin, "config", "user.name", "pray");
  runGit(origin, "config", "user.email", "pray@example.com");
  runGit(origin, "config", "uploadpack.allowReachableSHA1InWant", "true");
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
  const pinned = runGit(origin, "rev-parse", "HEAD").trim();
  writeFileSync(join(origin, "later.txt"), "two\n");
  runGit(origin, "add", "-A");
  runGit(
    origin,
    "-c",
    "commit.gpgsign=false",
    "-c",
    "core.hooksPath=/dev/null",
    "commit",
    "-m",
    "two",
  );
  const cloneUrl = `file://${origin}`;
  const cache = gitSourceCacheDirectory(root, cloneUrl);
  mkdirSync(join(cache, ".."), { recursive: true });
  runGit(root, "clone", "--depth", "1", "--no-local", cloneUrl, cache);
  const missing = spawnSync("git", ["-C", cache, "cat-file", "-e", pinned]);
  assert.notEqual(missing.status, 0, "fixture cache must lack the pin");
  return {
    root,
    cloneUrl,
    pinned,
    cleanup: () => {
      if (previousCache === undefined) delete process.env.PRAY_CACHE;
      else process.env.PRAY_CACHE = previousCache;
      rmSync(root, { recursive: true, force: true });
    },
  };
}

function runGit(directory: string, ...argumentsList: string[]): string {
  const result = spawnSync("git", ["-C", directory, ...argumentsList], {
    encoding: "utf8",
    env: {
      ...process.env,
      GIT_AUTHOR_NAME: "pray",
      GIT_AUTHOR_EMAIL: "pray@example.com",
      GIT_COMMITTER_NAME: "pray",
      GIT_COMMITTER_EMAIL: "pray@example.com",
    },
  });
  assert.equal(
    result.status,
    0,
    `git ${argumentsList.join(" ")} failed: ${result.stderr ?? result.stdout}`,
  );
  return result.stdout ?? "";
}
