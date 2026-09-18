import { existsSync, mkdirSync, rmSync } from "node:fs";
import { homedir } from "node:os";
import { join } from "node:path";
import { PrayError } from "../errors.js";
import { cloneBareGitDb } from "./clone.js";
import { cacheKey } from "./paths.js";
import { runGit, runGitCapture, tryRunGit } from "./run.js";

export function globalCacheRoot(): string | undefined {
  if (process.env.PRAY_CACHE) {
    return process.env.PRAY_CACHE;
  }
  if (process.env.PRAY_HOME) {
    return join(process.env.PRAY_HOME, "cache");
  }
  return join(homedir(), ".cache", "pray");
}

export function globalGitCacheDirectory(cloneUrl: string): string | undefined {
  const root = globalCacheRoot();
  return root ? join(root, "git", cacheKey(cloneUrl)) : undefined;
}

export function globalGitCacheReady(globalCache: string): boolean {
  return (
    existsSync(join(globalCache, "HEAD")) ||
    existsSync(join(globalCache, ".git"))
  );
}

export function offlineGitSourceUncached(cloneUrl: string): never {
  throw PrayError.resolution(
    `git source ${cloneUrl} is not cached locally and offline mode is enabled`,
  );
}

export function ensureGlobalGitDb(
  cloneUrl: string,
  pinnedRevision: string | undefined,
  refresh: boolean,
  offline: boolean,
  workingDirectory: string,
): { db: string; revision: string } {
  const db = globalGitCacheDirectory(cloneUrl);
  if (db === undefined) {
    throw PrayError.resolution("git object cache is not configured");
  }
  if (!globalGitCacheReady(db)) {
    if (offline) {
      offlineGitSourceUncached(cloneUrl);
    }
    if (existsSync(db)) {
      rmSync(db, { recursive: true, force: true });
    }
    mkdirSync(join(db, ".."), { recursive: true });
    cloneBareGitDb(workingDirectory, cloneUrl, db, false);
    ensureGitRemoteOrigin(db, cloneUrl);
  }
  if (refresh && !offline) {
    fetchOriginTip(db, cloneUrl);
  }
  if (pinnedRevision) {
    ensureRevisionInDb(db, cloneUrl, pinnedRevision, !offline);
    return { db, revision: pinnedRevision };
  }
  return { db, revision: gitHeadRevision(db) };
}

function ensureRevisionInDb(
  db: string,
  cloneUrl: string,
  revision: string,
  allowFetch: boolean,
): void {
  if (tryRunGit(db, "cat-file", "-e", revision)) {
    return;
  }
  if (!allowFetch) {
    throw PrayError.resolution(
      `git source ${JSON.stringify(db)} is locked to revision ${revision}, but that commit is not available locally and offline mode is enabled`,
    );
  }
  ensureGitRemoteOrigin(db, cloneUrl);
  if (existsSync(join(db, "shallow"))) {
    fetchUnshallow(db);
  }
  if (tryRunGit(db, "cat-file", "-e", revision)) {
    return;
  }
  fetchRevision(db, revision);
  if (!tryRunGit(db, "cat-file", "-e", revision)) {
    throw PrayError.resolution(
      `git source ${JSON.stringify(db)} is locked to revision ${revision}, but that commit could not be fetched`,
    );
  }
}

function ensureGitRemoteOrigin(repository: string, cloneUrl: string): void {
  if (tryRunGit(repository, "remote", "get-url", "origin")) {
    runGit(repository, "remote", "set-url", "origin", cloneUrl);
  } else {
    runGit(repository, "remote", "add", "origin", cloneUrl);
  }
}

function fetchOriginTip(db: string, cloneUrl: string): void {
  ensureGitRemoteOrigin(db, cloneUrl);
  if (!tryRunGit(db, "fetch", "--depth", "1", "--filter=blob:none", "origin")) {
    runGit(db, "fetch", "--depth", "1", "origin");
  }
  runGit(db, "update-ref", "HEAD", "FETCH_HEAD");
}

function fetchUnshallow(db: string): void {
  if (tryRunGit(db, "fetch", "--unshallow", "--filter=blob:none", "origin")) {
    return;
  }
  runGit(db, "fetch", "--unshallow", "origin");
}

function fetchRevision(db: string, revision: string): void {
  if (tryRunGit(db, "fetch", "--filter=blob:none", "origin", revision)) {
    return;
  }
  runGit(db, "fetch", "origin", revision);
}

function gitHeadRevision(repository: string): string {
  const output = runGitCapture(repository, "rev-parse", "HEAD").trim();
  if (output.length === 0) {
    throw PrayError.resolution("git repository has no HEAD revision");
  }
  return output;
}
