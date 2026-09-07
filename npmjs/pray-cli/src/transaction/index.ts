import {
  closeSync,
  constants,
  fchmodSync,
  mkdirSync,
  openSync,
  writeFileSync,
} from "node:fs";
import { join, resolve } from "node:path";
import { PrayError } from "../errors.js";
import { ensureSafeDestinationAncestors } from "../render/path-guard.js";
import { activeTransaction as active } from "./hooks.js";
import { Journal, snapshot } from "./journal.js";
import { acquire, syncDirectory } from "./owner.js";

export function runTransaction<T>(root: string, operation: () => T): T {
  root = resolve(root);
  const current = active.getStore();
  if (current?.root === root) return operation();
  if (current)
    throw PrayError.render(
      "a command cannot write two projects in one transaction",
    );
  ensureSafeDestinationAncestors(
    root,
    ".pray/write-state/owner",
    "project write state",
  );
  const directory = join(root, ".pray/write-state");
  mkdirSync(directory, { recursive: true, mode: 0o700 });
  ensureSafeDestinationAncestors(
    root,
    ".pray/write-state/owner",
    "project write state",
  );
  const handle = openSync(directory, constants.O_RDONLY | constants.O_NOFOLLOW);
  try {
    fchmodSync(handle, 0o700);
  } finally {
    closeSync(handle);
  }
  for (const path of [directory, join(root, ".pray"), root])
    syncDirectory(path);
  const release = acquire(directory);
  const journal = new Journal(root, directory);
  try {
    journal.recover();
  } catch (error) {
    release();
    throw error;
  }
  const failed = (error: unknown): never => {
    try {
      journal.recover();
    } catch (recovery) {
      throw PrayError.render(`${error}\nRecovery stopped: ${recovery}`);
    } finally {
      journal.closed = true;
      release();
    }
    throw error;
  };
  const committed = <V>(value: V): V => {
    try {
      journal.commit();
      return value;
    } finally {
      journal.closed = true;
      release();
    }
  };
  return active.run(journal, () => {
    let result: T;
    try {
      result = operation();
    } catch (error) {
      return failed(error);
    }
    if (result instanceof Promise) return result.then(committed, failed) as T;
    return committed(result);
  });
}
export function writeProjectFile(path: string, bytes: string | Buffer): void {
  const journal = active.getStore();
  if (!journal) {
    writeFileSync(path, bytes);
    return;
  }
  journal.replace(path, snapshot(path), Buffer.from(bytes));
}
