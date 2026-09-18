import { spawnSync } from "node:child_process";
import { existsSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { gitSourceCacheDirectory } from "./paths.js";
import { globalGitCacheDirectory, globalGitCacheReady } from "./store.js";

export function gitSourceCachedRepository(
  projectRoot: string,
  cloneUrl: string,
): string | undefined {
  const globalCache = globalGitCacheDirectory(cloneUrl);
  if (globalCache !== undefined && globalGitCacheReady(globalCache)) {
    return globalCache;
  }
  const shared = gitSourceCacheDirectory(projectRoot, cloneUrl);
  if (isGitCheckout(shared)) {
    return shared;
  }
  return findCachedOrigin(projectRoot, cloneUrl);
}

export function isGitCheckout(path: string): boolean {
  return existsSync(join(path, ".git"));
}

function findCachedOrigin(
  projectRoot: string,
  cloneUrl: string,
): string | undefined {
  const cacheRoot = join(projectRoot, ".pray", "cache", "git");
  if (!existsSync(cacheRoot)) {
    return undefined;
  }
  for (const name of readdirSync(cacheRoot)) {
    const path = join(cacheRoot, name);
    if (isGitCheckout(path) && originMatches(path, cloneUrl)) {
      return path;
    }
  }
  return undefined;
}

function originMatches(repository: string, cloneUrl: string): boolean {
  const result = spawnSync(
    "git",
    ["-C", repository, "remote", "get-url", "origin"],
    {
      encoding: "utf8",
    },
  );
  if (result.status !== 0) {
    return false;
  }
  const origin = (result.stdout ?? "").trim().replace(/^git\+/, "");
  return origin === cloneUrl;
}
