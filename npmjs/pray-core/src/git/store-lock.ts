import {
  mkdirSync,
  readFileSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { PrayError } from "../errors.js";

const LOCK_ATTEMPTS = 2_400;
const LOCK_RETRY_MS = 25;
const STALE_LOCK_SECS = 600;

export function withGitCacheLock<T>(db: string, body: () => T): T {
  const lockDirectory = `${db}.lock.d`;
  mkdirSync(join(lockDirectory, ".."), { recursive: true });
  let acquired = false;
  for (let attempt = 0; attempt < LOCK_ATTEMPTS; attempt += 1) {
    try {
      mkdirSync(lockDirectory);
      writeFileSync(
        join(lockDirectory, "owner"),
        `${process.pid}\n${Math.floor(Date.now() / 1000)}\n`,
      );
      acquired = true;
      break;
    } catch (error) {
      const code = (error as NodeJS.ErrnoException).code;
      if (code !== "EEXIST") {
        throw error;
      }
      clearStaleLock(lockDirectory);
      const until = Date.now() + LOCK_RETRY_MS;
      while (Date.now() < until) {
        // wait for the other process to release the exclusive lock directory
      }
    }
  }
  if (!acquired) {
    throw PrayError.resolution(
      `timed out waiting for git cache lock ${lockDirectory}. If no other pray process is using that cache, remove the lock directory and retry.`,
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

function clearStaleLock(lockDirectory: string): void {
  if (lockAgeSeconds(lockDirectory) < STALE_LOCK_SECS) {
    return;
  }
  try {
    rmSync(lockDirectory, { recursive: true, force: true });
  } catch {
    // another process may have cleared it
  }
}

function lockAgeSeconds(lockDirectory: string): number {
  try {
    const text = readFileSync(join(lockDirectory, "owner"), "utf8");
    const stamp = Number.parseInt(text.split("\n")[1] ?? "", 10);
    if (Number.isFinite(stamp) && stamp > 0) {
      return Math.floor(Date.now() / 1000) - stamp;
    }
  } catch {
    // fall through to directory mtime
  }
  try {
    return Math.floor((Date.now() - statSync(lockDirectory).mtimeMs) / 1000);
  } catch {
    return 0;
  }
}
