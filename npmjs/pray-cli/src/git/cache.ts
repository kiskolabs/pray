import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, rmSync, statSync } from "node:fs";
import { homedir } from "node:os";
import { isAbsolute, join, resolve } from "node:path";
import { PrayError } from "../errors.js";
import { cacheKey, gitSourceCacheDirectory } from "./paths.js";

export { gitSourceCacheDirectory } from "./paths.js";

export function ensureGitRepository(
  projectRoot: string,
  cloneUrl: string,
  refresh: boolean,
  pinnedRevision?: string,
  sparseSubdir?: string,
): { cacheDirectory: string; revision: string } {
  const shared = gitSourceCacheDirectory(projectRoot, cloneUrl);
  ensureSharedGitRepository(
    projectRoot,
    cloneUrl,
    shared,
    refresh,
    pinnedRevision,
  );
  const cacheDirectory = gitSourceCacheDirectory(
    projectRoot,
    cloneUrl,
    sparseSubdir,
  );
  if (cacheDirectory !== shared) {
    ensureLinkedWorktree(shared, cacheDirectory);
    if (pinnedRevision) {
      checkoutGitRevision(cacheDirectory, pinnedRevision, refresh);
    } else if (refresh) {
      runGit(cacheDirectory, "reset", "--hard", gitHeadRevision(shared));
    }
    if (sparseSubdir) {
      applySparseCheckout(cacheDirectory, sparseSubdir);
    }
    return { cacheDirectory, revision: gitHeadRevision(cacheDirectory) };
  }
  return { cacheDirectory: shared, revision: gitHeadRevision(shared) };
}

function ensureSharedGitRepository(
  projectRoot: string,
  cloneUrl: string,
  shared: string,
  refresh: boolean,
  pinnedRevision?: string,
): void {
  if (gitDirectory(shared)) {
    if (pinnedRevision) {
      checkoutGitRevision(shared, pinnedRevision, refresh);
    } else if (refresh) {
      refreshGitWorktree(shared);
    }
    if (refresh) {
      refreshGlobalFromProject(cloneUrl, shared);
    }
    return;
  }

  if (existsSync(shared)) {
    rmSync(shared, { recursive: true, force: true });
  }
  mkdirSync(join(shared, ".."), { recursive: true });

  const seeded = seedGitCacheFromGlobal(cloneUrl, shared, projectRoot);
  if (seeded) {
    runGit(shared, "remote", "set-url", "origin", cloneUrl);
  } else {
    runGit(projectRoot, "clone", "--depth", "1", cloneUrl, shared);
    mirrorGitCacheToGlobal(cloneUrl, shared);
  }

  if (pinnedRevision) {
    checkoutGitRevision(shared, pinnedRevision, true);
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
  runGit(shared, "worktree", "add", "--detach", checkout);
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

function seedGitCacheFromGlobal(
  cloneUrl: string,
  destination: string,
  workingDirectory: string,
): boolean {
  const globalCache = globalGitCacheDirectory(cloneUrl);
  if (!globalCache || !globalGitCacheReady(globalCache)) {
    return false;
  }
  runGit(
    workingDirectory,
    "clone",
    "--depth",
    "1",
    "--quiet",
    globalCache,
    destination,
  );
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

function applySparseCheckout(repository: string, subdir: string): void {
  runGit(repository, "sparse-checkout", "init", "--cone");
  runGit(repository, "sparse-checkout", "set", subdir);
}

function checkoutGitRevision(
  repository: string,
  revision: string,
  refresh: boolean,
): void {
  if (refresh) {
    runGit(repository, "fetch", "--depth", "1", "origin", revision);
  }
  runGit(repository, "checkout", "--force", revision);
}

function refreshGitWorktree(repository: string): void {
  runGit(repository, "fetch", "--depth", "1", "origin");
  runGit(repository, "reset", "--hard", "origin/HEAD");
}

function gitHeadRevision(repository: string): string {
  const output = runGitCapture(repository, "rev-parse", "HEAD").trim();
  if (output.length === 0) {
    throw PrayError.resolution("git repository has no HEAD revision");
  }
  return output;
}

function runGit(repository: string, ...argumentsList: string[]): void {
  const result = spawnSync("git", ["-C", repository, ...argumentsList], {
    encoding: "utf8",
  });
  if ((result.error as NodeJS.ErrnoException | undefined)?.code === "ENOENT") {
    throw PrayError.unsupported("git is required for git sources");
  }
  if (result.status !== 0) {
    throw PrayError.resolution(
      commandError(
        `git ${argumentsList.join(" ")}`,
        result.stderr ?? result.stdout ?? "",
      ),
    );
  }
}

function runGitCapture(repository: string, ...argumentsList: string[]): string {
  const result = spawnSync("git", ["-C", repository, ...argumentsList], {
    encoding: "utf8",
  });
  if (result.status !== 0) {
    throw PrayError.resolution(
      commandError(
        `git ${argumentsList.join(" ")}`,
        result.stderr ?? result.stdout ?? "",
      ),
    );
  }
  return result.stdout ?? "";
}

function commandError(program: string, output: string): string {
  const message = output.trim();
  return message.length === 0
    ? `${program} failed`
    : `${program} failed: ${message}`;
}
