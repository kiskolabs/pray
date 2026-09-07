import { randomBytes } from "node:crypto";
import {
  closeSync,
  constants,
  fsyncSync,
  linkSync,
  openSync,
  unlinkSync,
  writeFileSync,
} from "node:fs";
import { hostname } from "node:os";
import { join } from "node:path";
import { PrayError } from "../errors.js";
import { readRegularBytes } from "../render/destination-io.js";

type Owner = { version: number; pid: number; host: string; token: string };
export const token = () => randomBytes(16).toString("hex");
export function syncDirectory(path: string): void {
  if (process.platform === "win32") return;
  const descriptor = openSync(path, constants.O_RDONLY);
  try {
    fsyncSync(descriptor);
  } finally {
    closeSync(descriptor);
  }
}
export function privateFile(path: string): number {
  return openSync(
    path,
    constants.O_WRONLY |
      constants.O_CREAT |
      constants.O_EXCL |
      constants.O_NOFOLLOW,
    0o600,
  );
}
export function acquire(
  directory: string,
  path = join(directory, "owner"),
  depth = 0,
): () => void {
  if (depth > 16)
    throw PrayError.render(
      "too many interrupted lock recoveries; inspect .pray/write-state",
    );
  const owner: Owner = {
    version: 1,
    pid: process.pid,
    host: hostname(),
    token: token(),
  };
  const temporary = join(directory, `${owner.token}.owner`);
  const descriptor = privateFile(temporary);
  try {
    writeFileSync(descriptor, JSON.stringify(owner));
    fsyncSync(descriptor);
  } finally {
    closeSync(descriptor);
  }
  let linked = false;
  try {
    linkSync(temporary, path);
    linked = true;
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code !== "EEXIST") throw error;
  } finally {
    unlinkSync(temporary);
  }
  if (linked) {
    syncDirectory(directory);
    return () => {
      if (readOwner(path).token === owner.token) unlinkSync(path);
    };
  }
  const previous = readOwner(path);
  if (previous.host !== hostname() || alive(previous.pid))
    throw PrayError.render(
      "another pray command owns this project; retry after it finishes",
    );
  // A claim tied to the dead owner's unique token serializes stale-lock removal.
  const release = acquire(
    directory,
    join(directory, `reclaim-${previous.token}`),
    depth + 1,
  );
  try {
    if (readOwner(path).token !== previous.token)
      throw PrayError.render("project ownership changed; retry the command");
    unlinkSync(path);
    return acquire(directory, path, depth + 1);
  } finally {
    release();
  }
}
function alive(pid: number): boolean {
  try {
    process.kill(pid, 0);
    return true;
  } catch (error) {
    return (error as NodeJS.ErrnoException).code !== "ESRCH";
  }
}
function readOwner(path: string): Owner {
  const bytes = readRegularBytes(path, "project write owner");
  if (bytes.length > 4096)
    throw PrayError.render("invalid project write owner");
  let owner: Owner;
  try {
    owner = JSON.parse(bytes.toString("utf8"));
  } catch {
    throw PrayError.render("invalid project write owner");
  }
  if (
    owner.version !== 1 ||
    !Number.isSafeInteger(owner.pid) ||
    owner.pid <= 0 ||
    typeof owner.host !== "string" ||
    !/^[a-f0-9]{32}$/.test(owner.token)
  )
    throw PrayError.render("invalid project write owner");
  return owner;
}
