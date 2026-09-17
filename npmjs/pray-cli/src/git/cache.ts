import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, rmSync } from "node:fs";
import { homedir } from "node:os";
import { join } from "node:path";
import { PrayError } from "../errors.js";
import { sha256Hex } from "../hashing.js";

export function gitSourceCacheDirectory(
  projectRoot: string,
  cloneUrl: string,
  subdir?: string,
): string {
  const identity =
    subdir !== undefined && subdir.length > 0
      ? `${cloneUrl}\n${subdir}`
      : cloneUrl;
  return join(projectRoot, ".pray", "cache", "git", cacheKey(identity));
}

export function ensureGitRepository(
  projectRoot: string,
  cloneUrl: string,
  refresh: boolean,
  pinnedRevision?: string,
  sparseSubdir?: string,
): { cacheDirectory: string; revision: string } {
  const cacheDirectory = gitSourceCacheDirectory(
    projectRoot,
    cloneUrl,
    sparseSubdir,
  );
  if (existsSync(join(cacheDirectory, ".git"))) {
    if (pinnedRevision) {
      checkoutGitRevision(cacheDirectory, pinnedRevision, refresh);
    } else if (refresh) {
      refreshGitWorktree(cacheDirectory);
    }
    if (refresh) {
      refreshGlobalFromProject(cloneUrl, cacheDirectory);
    }
    if (sparseSubdir) {
      applySparseCheckout(cacheDirectory, sparseSubdir);
    }
    return { cacheDirectory, revision: gitHeadRevision(cacheDirectory) };
  }

  if (existsSync(cacheDirectory)) {
    rmSync(cacheDirectory, { recursive: true, force: true });
  }
  mkdirSync(join(cacheDirectory, ".."), { recursive: true });

  const seeded = seedGitCacheFromGlobal(cloneUrl, cacheDirectory, projectRoot);
  if (seeded) {
    runGit(cacheDirectory, "remote", "set-url", "origin", cloneUrl);
  } else {
    runGit(projectRoot, "clone", "--depth", "1", cloneUrl, cacheDirectory);
    mirrorGitCacheToGlobal(cloneUrl, cacheDirectory);
  }

  if (pinnedRevision) {
    checkoutGitRevision(cacheDirectory, pinnedRevision, true);
  } else if (refresh && seeded) {
    refreshGitWorktree(cacheDirectory);
  }
  if (refresh && seeded) {
    refreshGlobalFromProject(cloneUrl, cacheDirectory);
  }
  if (sparseSubdir) {
    applySparseCheckout(cacheDirectory, sparseSubdir);
  }

  return { cacheDirectory, revision: gitHeadRevision(cacheDirectory) };
}

function cacheKey(text: string): string {
  return sha256Hex(text).slice(0, 16);
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
