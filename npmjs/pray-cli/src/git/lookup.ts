import { existsSync, readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { gitSourceCacheDirectory } from "./paths.js";
import { globalGitCacheDirectory, globalGitCacheReady } from "./store.js";

const GIT_DIR_MARKER = ".pray-git-dir";

/**
 * Resolve the git object store for trust and diagnostics.
 * Order: ready shared bare, then a project stub's `.pray-git-dir`.
 * Never select a leftover full clone by scanning `.git`.
 */
export function gitSourceCachedRepository(
  projectRoot: string,
  cloneUrl: string,
): string | undefined {
  const globalCache = globalGitCacheDirectory(cloneUrl);
  if (globalCache !== undefined && globalGitCacheReady(globalCache)) {
    return globalCache;
  }
  return (
    stubGitDir(gitSourceCacheDirectory(projectRoot, cloneUrl)) ??
    findStubGitDir(projectRoot, cloneUrl)
  );
}

export function isGitCheckout(path: string): boolean {
  return existsSync(join(path, ".git"));
}

function stubGitDir(catalog: string): string | undefined {
  const marker = join(catalog, GIT_DIR_MARKER);
  if (!existsSync(marker)) {
    return undefined;
  }
  const gitDir = readFileSync(marker, "utf8").trim();
  if (gitDir.length === 0 || !existsSync(gitDir)) {
    return undefined;
  }
  return gitDir;
}

function findStubGitDir(
  projectRoot: string,
  cloneUrl: string,
): string | undefined {
  const expected = globalGitCacheDirectory(cloneUrl);
  if (expected === undefined) {
    return undefined;
  }
  const cacheRoot = join(projectRoot, ".pray", "cache", "git");
  if (!existsSync(cacheRoot)) {
    return undefined;
  }
  for (const name of readdirSync(cacheRoot)) {
    const path = join(cacheRoot, name);
    if (isGitCheckout(path)) {
      continue;
    }
    const gitDir = stubGitDir(path);
    if (gitDir === expected) {
      return gitDir;
    }
  }
  return undefined;
}
