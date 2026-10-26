import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { globalGitCacheDirectory } from "./git/store.js";

export interface PinnedShallowCache {
  root: string;
  origin: string;
  cloneUrl: string;
  pinned: string;
  db: string;
  cleanup: () => void;
}

export function pinnedShallowCache(): PinnedShallowCache {
  const root = mkdtempSync(join(tmpdir(), "pray-pinned-git-"));
  const previousCache = process.env.PRAY_CACHE;
  process.env.PRAY_CACHE = join(root, "global-cache");
  const origin = join(root, "origin");
  mkdirSync(join(origin, "v1/packages"), { recursive: true });
  writeFileSync(join(origin, "v1/packages/sample.json"), "{}\n");
  runGit(origin, "init", "--template=", "-b", "main");
  runGit(origin, "config", "user.name", "pray");
  runGit(origin, "config", "user.email", "pray@example.com");
  runGit(origin, "config", "uploadpack.allowReachableSHA1InWant", "true");
  commitAll(origin, "one");
  writeFileSync(join(origin, "middle.txt"), "pin\n");
  commitAll(origin, "middle");
  const pinned = runGit(origin, "rev-parse", "HEAD").trim();
  writeFileSync(join(origin, "later.txt"), "two\n");
  commitAll(origin, "two");
  const cloneUrl = `file://${origin}`;
  const db = globalGitCacheDirectory(cloneUrl);
  assert.ok(db);
  mkdirSync(join(db, ".."), { recursive: true });
  runGit(root, "clone", "--bare", "--depth", "1", "--no-local", cloneUrl, db);
  const missing = spawnSync("git", ["-C", db, "cat-file", "-e", pinned]);
  assert.notEqual(missing.status, 0, "fixture cache must lack the pin");
  return {
    root,
    origin,
    cloneUrl,
    pinned,
    db,
    cleanup: () => {
      if (previousCache === undefined) delete process.env.PRAY_CACHE;
      else process.env.PRAY_CACHE = previousCache;
      rmSync(root, { recursive: true, force: true });
    },
  };
}

export function commitAll(directory: string, message: string): void {
  runGit(directory, "add", "-A");
  runGit(
    directory,
    "-c",
    "commit.gpgsign=false",
    "-c",
    "core.hooksPath=/dev/null",
    "commit",
    "-m",
    message,
  );
}

export function runGit(directory: string, ...argumentsList: string[]): string {
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
