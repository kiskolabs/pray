import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, rmSync } from "node:fs";
import { homedir } from "node:os";
import { isAbsolute, join } from "node:path";
import { PrayError } from "../errors.js";
import { sha256Hex } from "../hashing.js";
import type { Lockfile } from "../lockfile/types.js";
import type { ManifestSource } from "../manifest/types.js";

export interface GitSourceCheckout {
  cacheDirectory: string;
  revision: string;
  subdir?: string;
}

export function prepareGitSources(
  projectRoot: string,
  sources: ManifestSource[],
  lockfile: Lockfile | undefined,
  refresh = false,
): Map<string, GitSourceCheckout> {
  const checkouts = new Map<string, GitSourceCheckout>();
  for (const source of sources) {
    if (source.kind !== "git") {
      continue;
    }
    const cloneUrl = source.url.replace(/^git\+/, "");
    if (isLocalFilesystemSource(cloneUrl) && !localGitRepoPath(cloneUrl)) {
      const sourceRoot = localGitSourceRoot(cloneUrl);
      if (sourceRoot) {
        checkouts.set(source.name, {
          cacheDirectory: sourceRoot,
          revision: "",
          subdir: source.subdir,
        });
      }
      continue;
    }
    const pinnedRevision = refresh
      ? undefined
      : pinnedRevisionForSource(lockfile, source);
    const { cacheDirectory, revision } = ensureGitRepository(
      projectRoot,
      cloneUrl,
      refresh,
      pinnedRevision,
      source.subdir,
    );
    checkouts.set(source.name, {
      cacheDirectory,
      revision,
      subdir: source.subdir,
    });
  }
  return checkouts;
}

export function resolveDistributionRoot(
  repositoryRoot: string,
  subdir?: string,
): string {
  if (subdir) {
    const path = join(repositoryRoot, subdir);
    if (localDistributionRoot(path)) {
      return path;
    }
    throw PrayError.resolution(
      `no pray distribution root at subdir ${path} in git source ${repositoryRoot}`,
    );
  }
  const discovered = discoverDistributionRoot(repositoryRoot);
  if (discovered) {
    return discovered;
  }
  throw PrayError.resolution(
    `no pray distribution root in git source ${repositoryRoot}. ` +
      "Expected v1/packages at the repository root or under prayers/.",
  );
}

export function discoverDistributionRoot(path: string): string | undefined {
  if (localDistributionRoot(path)) {
    return path;
  }
  const prayersRoot = join(path, "prayers");
  if (localDistributionRoot(prayersRoot)) {
    return prayersRoot;
  }
  return undefined;
}

export function localDistributionRoot(path: string): boolean {
  return existsSync(join(path, "v1", "packages"));
}

export function localGitSourceRoot(cloneUrl: string): string | undefined {
  const path = cloneUrl.startsWith("file://")
    ? cloneUrl.slice("file://".length)
    : cloneUrl;
  if (!existsSync(path)) {
    return undefined;
  }
  return discoverDistributionRoot(path);
}

export function gitSourceCacheDirectory(
  projectRoot: string,
  cloneUrl: string,
): string {
  return join(projectRoot, ".pray", "cache", "git", cacheKey(cloneUrl));
}

function ensureGitRepository(
  projectRoot: string,
  cloneUrl: string,
  refresh: boolean,
  pinnedRevision?: string,
  sparseSubdir?: string,
): { cacheDirectory: string; revision: string } {
  const cacheDirectory = gitSourceCacheDirectory(projectRoot, cloneUrl);
  if (existsSync(join(cacheDirectory, ".git"))) {
    if (refresh) {
      refreshGlobalGitCache(cloneUrl);
    }
    if (pinnedRevision) {
      checkoutGitRevision(cacheDirectory, pinnedRevision, refresh);
    } else if (refresh) {
      refreshGitWorktree(cacheDirectory);
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

  if (!seedGitCacheFromGlobal(cloneUrl, cacheDirectory, projectRoot)) {
    runGit(projectRoot, "clone", "--depth", "1", cloneUrl, cacheDirectory);
    mirrorGitCacheToGlobal(cloneUrl, cacheDirectory);
  } else {
    runGit(cacheDirectory, "remote", "set-url", "origin", cloneUrl);
  }

  if (pinnedRevision) {
    checkoutGitRevision(cacheDirectory, pinnedRevision, true);
  }
  if (sparseSubdir) {
    applySparseCheckout(cacheDirectory, sparseSubdir);
  }

  return { cacheDirectory, revision: gitHeadRevision(cacheDirectory) };
}

function pinnedRevisionForSource(
  lockfile: Lockfile | undefined,
  source: ManifestSource,
): string | undefined {
  const locked = lockfile?.source.find(
    (entry) => entry.name === source.name && entry.kind === "git",
  );
  if (locked?.revision) {
    return locked.revision;
  }
  if (source.kind === "git") {
    return source.rev ?? source.tag;
  }
  return undefined;
}

function isLocalFilesystemSource(cloneUrl: string): boolean {
  return cloneUrl.startsWith("file://") || isAbsolute(cloneUrl);
}

function localGitRepoPath(cloneUrl: string): string | undefined {
  const path = cloneUrl.startsWith("file://")
    ? cloneUrl.slice("file://".length)
    : cloneUrl;
  return existsSync(join(path, ".git")) ? path : undefined;
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

function refreshGlobalGitCache(cloneUrl: string): void {
  const globalCache = globalGitCacheDirectory(cloneUrl);
  if (!globalCache || !globalGitCacheReady(globalCache)) {
    return;
  }
  runGit(globalCache, "fetch", "--depth", "1", "origin");
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
