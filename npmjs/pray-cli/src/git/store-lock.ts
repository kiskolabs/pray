import { mkdirSync, rmSync } from "node:fs";
import { join } from "node:path";
import { PrayError } from "../errors.js";

export function withGitCacheLock<T>(db: string, body: () => T): T {
  const lockDirectory = `${db}.lock.d`;
  mkdirSync(join(lockDirectory, ".."), { recursive: true });
  let acquired = false;
  for (let attempt = 0; attempt < 400; attempt += 1) {
    try {
      mkdirSync(lockDirectory);
      acquired = true;
      break;
    } catch (error) {
      const code = (error as NodeJS.ErrnoException).code;
      if (code !== "EEXIST") {
        throw error;
      }
      const until = Date.now() + 25;
      while (Date.now() < until) {
        // wait for the other process to release the exclusive lock directory
      }
    }
  }
  if (!acquired) {
    throw PrayError.resolution(
      `timed out waiting for git cache lock ${lockDirectory}`,
    );
  }
  try {
    return body();
  } finally {
    try {
      rmSync(lockDirectory, { recursive: true, force: true });
    } catch {
      // best-effort unlock
    }
  }
}
