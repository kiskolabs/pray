import { existsSync, mkdirSync, rmSync, statSync } from "node:fs";
import { homedir } from "node:os";
import { isAbsolute, join, resolve } from "node:path";
import { PrayError } from "../errors.js";
import { applySparseCheckout, cloneGitCache } from "./clone.js";
import { cacheKey, gitSourceCacheDirectory } from "./paths.js";
import { runGit, runGitCapture, tryRunGit } from "./run.js";

export { gitSourceCacheDirectory } from "./paths.js";

export function ensureGitRepository(
  projectRoot: string,
  cloneUrl: string,
  refresh: boolean,
  pinnedRevision?: string,
  sparseSubdir?: string,
  offline = false,
): { cacheDirectory: string; revision: string } {
  const shared = gitSourceCacheDirectory(projectRoot, cloneUrl);
  ensureSharedGitRepository(
    projectRoot,
    cloneUrl,
    shared,
    refresh,
    pinnedRevision,
    offline,
  );
  const cacheDirectory = gitSourceCacheDirectory(
    projectRoot,
    cloneUrl,
    sparseSubdir,
  );
  if (cacheDirectory !== shared) {
    ensureLinkedWorktree(shared, cacheDirectory);
    applySparseCheckout(cacheDirectory, sparseSubdir);
    if (pinnedRevision) {
      checkoutGitRevision(cacheDirectory, pinnedRevision, !offline);
    } else if (refresh) {
      runGit(cacheDirectory, "reset", "--hard", gitHeadRevision(shared));
    }
    return { cacheDirectory, revision: gitHeadRevision(cacheDirectory) };
  }
  applySparseCheckout(shared);
  return { cacheDirectory: shared, revision: gitHeadRevision(shared) };
}

function ensureSharedGitRepository(
  projectRoot: string,
  cloneUrl: string,
  shared: string,
  refresh: boolean,
  pinnedRevision: string | undefined,
  offline: boolean,
): void {
  if (gitDirectory(shared)) {
    if (pinnedRevision) {
      checkoutGitRevision(shared, pinnedRevision, !offline);
    } else if (refresh) {
      refreshGitWorktree(shared);
    }
    if (refresh) {
      refreshGlobalFromProject(cloneUrl, shared);
    }
    return;
  }

  if (offline && !gitGlobalSeedAvailable(cloneUrl)) {
    offlineGitSourceUncached(cloneUrl);
  }

  if (existsSync(shared)) {
    rmSync(shared, { recursive: true, force: true });
  }
  mkdirSync(join(shared, ".."), { recursive: true });

  const seeded = seedGitCacheFromGlobal(cloneUrl, shared, projectRoot);
  if (seeded) {
    runGit(shared, "remote", "set-url", "origin", cloneUrl);
  } else if (offline) {
    offlineGitSourceUncached(cloneUrl);
  } else {
    cloneGitCache(projectRoot, cloneUrl, shared, false);
    mirrorGitCacheToGlobal(cloneUrl, shared);
  }
  applySparseCheckout(shared);

  if (pinnedRevision) {
    checkoutGitRevision(shared, pinnedRevision, !offline);
  } else if (refresh && seeded) {
    refreshGitWorktree(shared);
  }
  if (refresh && seeded) {
    refreshGlobalFromProject(cloneUrl, shared);
  }
}

function gitDirectory(repository: string): boolean {
  try {
    return statSync(join(repository, ".git")).isDirectory();
  } catch {
    return false;
  }
}

function ensureLinkedWorktree(shared: string, checkout: string): void {
  if (sameObjectStore(shared, checkout)) {
    return;
  }
  if (existsSync(checkout)) {
    rmSync(checkout, { recursive: true, force: true });
  }
  mkdirSync(join(checkout, ".."), { recursive: true });
  if (
    !tryRunGit(shared, "worktree", "add", "--detach", "--no-checkout", checkout)
  ) {
    runGit(shared, "worktree", "add", "--detach", checkout);
  }
}

function sameObjectStore(shared: string, checkout: string): boolean {
  if (!existsSync(join(checkout, ".git"))) {
    return false;
  }
  try {
    return gitCommonDir(shared) === gitCommonDir(checkout);
  } catch {
    return false;
  }
}

function gitCommonDir(repository: string): string {
  const reported = runGitCapture(
    repository,
    "rev-parse",
    "--git-common-dir",
  ).trim();
  const resolved = isAbsolute(reported)
    ? reported
    : resolve(repository, reported);
  return resolve(resolved);
}

function globalCacheRoot(): string | undefined {
  if (process.env.PRAY_CACHE) {
    return process.env.PRAY_CACHE;
  }
  if (process.env.PRAY_HOME) {
    return join(process.env.PRAY_HOME, "cache");
  }
  return join(homedir(), ".cache", "pray");
}

function globalGitCacheDirectory(cloneUrl: string): string | undefined {
  const root = globalCacheRoot();
  return root ? join(root, "git", cacheKey(cloneUrl)) : undefined;
}

function globalGitCacheReady(globalCache: string): boolean {
  return (
    existsSync(join(globalCache, ".git")) ||
    existsSync(join(globalCache, "HEAD"))
  );
}

function gitGlobalSeedAvailable(cloneUrl: string): boolean {
  const globalCache = globalGitCacheDirectory(cloneUrl);
  return globalCache !== undefined && globalGitCacheReady(globalCache);
}

function offlineGitSourceUncached(cloneUrl: string): never {
  throw PrayError.resolution(
    `git source ${cloneUrl} is not cached locally and offline mode is enabled`,
  );
}

function seedGitCacheFromGlobal(
  cloneUrl: string,
  destination: string,
  workingDirectory: string,
): boolean {
  const globalCache = globalGitCacheDirectory(cloneUrl);
  if (!globalCache || !globalGitCacheReady(globalCache)) {
    return false;
  }
  cloneGitCache(workingDirectory, globalCache, destination, true);
  return true;
}

function mirrorGitCacheToGlobal(cloneUrl: string, projectCache: string): void {
  const globalCache = globalGitCacheDirectory(cloneUrl);
  if (!globalCache || globalGitCacheReady(globalCache)) {
    return;
  }
  mkdirSync(join(globalCache, ".."), { recursive: true });
  if (existsSync(globalCache)) {
    rmSync(globalCache, { recursive: true, force: true });
  }
  runGit(
    join(projectCache, ".."),
    "clone",
    "--bare",
    "--quiet",
    projectCache,
    globalCache,
  );
}

function refreshGlobalFromProject(
  cloneUrl: string,
  projectCache: string,
): void {
  const globalCache = globalGitCacheDirectory(cloneUrl);
  if (!globalCache) {
    return;
  }
  if (globalGitCacheReady(globalCache) || existsSync(globalCache)) {
    rmSync(globalCache, { recursive: true, force: true });
  }
  mirrorGitCacheToGlobal(cloneUrl, projectCache);
}

function fetchOrigin(repository: string, revision?: string): void {
  const extra = revision === undefined ? [] : [revision];
  if (
    !tryRunGit(
      repository,
      "fetch",
      "--depth",
      "1",
      "--filter=blob:none",
      "origin",
      ...extra,
    )
  ) {
    runGit(repository, "fetch", "--depth", "1", "origin", ...extra);
  }
}

function checkoutGitRevision(
  repository: string,
  revision: string,
  allowFetch: boolean,
): void {
  if (tryRunGit(repository, "cat-file", "-e", revision)) {
    runGit(repository, "checkout", "--force", revision);
    return;
  }
  if (!allowFetch) {
    throw PrayError.resolution(
      `git source ${JSON.stringify(repository)} is locked to revision ${revision}, but that commit is not available locally and offline mode is enabled`,
    );
  }
  fetchOrigin(repository, revision);
  if (!tryRunGit(repository, "cat-file", "-e", revision)) {
    runGit(repository, "fetch", "origin", revision);
  }
  runGit(repository, "checkout", "--force", revision);
}

function refreshGitWorktree(repository: string): void {
  fetchOrigin(repository);
  runGit(repository, "reset", "--hard", "origin/HEAD");
}

function gitHeadRevision(repository: string): string {
  const output = runGitCapture(repository, "rev-parse", "HEAD").trim();
  if (output.length === 0) {
    throw PrayError.resolution("git repository has no HEAD revision");
  }
  return output;
}
