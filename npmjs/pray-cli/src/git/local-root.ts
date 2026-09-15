import { existsSync } from "node:fs";
import { isAbsolute, join, resolve } from "node:path";
import { discoverDistributionRoot } from "./distribution-root.js";

export function cloneUrlFilesystemPath(
  projectRoot: string,
  cloneUrl: string,
): string {
  const path = cloneUrl.startsWith("file://")
    ? cloneUrl.slice("file://".length)
    : cloneUrl;
  return isAbsolute(path) ? path : resolve(projectRoot, path);
}

export function localGitSourceRoot(
  projectRoot: string,
  cloneUrl: string,
): string | undefined {
  const path = cloneUrlFilesystemPath(projectRoot, cloneUrl);
  if (!existsSync(path)) {
    return undefined;
  }
  return discoverDistributionRoot(path);
}

export function localGitRepoPath(
  projectRoot: string,
  cloneUrl: string,
): string | undefined {
  const path = cloneUrlFilesystemPath(projectRoot, cloneUrl);
  return existsSync(join(path, ".git")) ? path : undefined;
}

export function isLocalFilesystemSource(cloneUrl: string): boolean {
  return cloneUrl.startsWith("file://") || isAbsolute(cloneUrl);
}
