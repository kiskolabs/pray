import { existsSync, realpathSync, readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { gitSourceCacheDirectory } from "./paths.js";
import { globalGitCacheDirectory, globalGitCacheReady } from "./store.js";

const GIT_DIR_MARKER = ".pray-git-dir";

/**
 * Resolve the git object store for trust and diagnostics.
 * Order: ready shared bare, then a project stub's `.pray-git-dir` that points
 * at that same expected global bare. Never select a leftover full clone.
 */
export function gitSourceCachedRepository(
  projectRoot: string,
  cloneUrl: string,
): string | undefined {
  const expected = globalGitCacheDirectory(cloneUrl);
  if (expected === undefined) {
    return undefined;
  }
  if (globalGitCacheReady(expected)) {
    return expected;
  }
  return (
    stubGitDir(gitSourceCacheDirectory(projectRoot, cloneUrl), expected) ??
    findStubGitDir(projectRoot, expected)
  );
}

export function isGitCheckout(path: string): boolean {
  return existsSync(join(path, ".git"));
}

function stubGitDir(
  catalog: string,
  expected: string,
): string | undefined {
  const marker = join(catalog, GIT_DIR_MARKER);
  if (!existsSync(marker)) {
    return undefined;
  }
  const gitDir = readFileSync(marker, "utf8").trim();
  if (gitDir.length === 0 || !existsSync(gitDir)) {
    return undefined;
  }
  return pathsEqual(gitDir, expected) ? gitDir : undefined;
}

function findStubGitDir(
  projectRoot: string,
  expected: string,
): string | undefined {
  const cacheRoot = join(projectRoot, ".pray", "cache", "git");
  if (!existsSync(cacheRoot)) {
    return undefined;
  }
  for (const name of readdirSync(cacheRoot)) {
    const path = join(cacheRoot, name);
    if (isGitCheckout(path)) {
      continue;
    }
    const gitDir = stubGitDir(path, expected);
    if (gitDir !== undefined) {
      return gitDir;
    }
  }
  return undefined;
}

function pathsEqual(left: string, right: string): boolean {
  if (left === right) {
    return true;
  }
  try {
    return realpathSync(left) === realpathSync(right);
  } catch {
    return false;
  }
}
