import { join } from "node:path";
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

export function cacheKey(text: string): string {
  return sha256Hex(text).slice(0, 16);
}
