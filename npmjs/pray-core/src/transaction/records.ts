import {
  closeSync,
  constants,
  fstatSync,
  fsyncSync,
  ftruncateSync,
  readSync,
} from "node:fs";
import { PrayError } from "../errors.js";
import { validateDestinationPath } from "../manifest/validate.js";
import { openRegular } from "../render/destination-io.js";
export type Entry = {
  path: string;
  before: string | null;
  after_hash: string | null;
  mode: number;
};
export type JournalRecord =
  | { type: "start"; version: number }
  | { type: "write"; entry: Entry }
  | { type: "undone"; index: number }
  | { type: "commit" };
export const MAX_LOG = 96 * 1024 * 1024;
export function readJournal(path: string): {
  entries: Entry[];
  undone: Set<number>;
  committed: boolean;
} {
  const descriptor = openRegular(
    path,
    "project recovery journal",
    constants.O_RDWR,
  );
  let bytes: Buffer;
  try {
    const size = fstatSync(descriptor).size;
    if (size > MAX_LOG)
      throw PrayError.render(
        "project recovery journal exceeds its 96 MiB limit",
      );
    bytes = Buffer.alloc(size);
    let offset = 0;
    while (offset < size) {
      const count = readSync(descriptor, bytes, offset, size - offset, null);
      if (!count) break;
      offset += count;
    }
    bytes = bytes.subarray(0, offset);
    const complete = bytes.lastIndexOf(10) + 1;
    bytes = bytes.subarray(0, complete);
    // A partial final record precedes its destination mutation and can be discarded.
    ftruncateSync(descriptor, complete);
    fsyncSync(descriptor);
  } finally {
    closeSync(descriptor);
  }
  const entries: Entry[] = [];
  const undone = new Set<number>();
  let committed = false;
  for (const [index, line] of bytes
    .toString("utf8")
    .split("\n")
    .filter(Boolean)
    .entries()) {
    let record: JournalRecord;
    try {
      record = JSON.parse(line);
    } catch {
      throw PrayError.render("invalid project recovery journal");
    }
    if (record.type === "start" && record.version === 1 && index === 0)
      continue;
    if (record.type === "write" && index > 0 && !committed && !undone.size) {
      const entry = record.entry;
      if (
        entries.length === 10000 ||
        typeof entry?.path !== "string" ||
        !Number.isInteger(entry.mode) ||
        entry.mode < 0 ||
        entry.mode > 0o777 ||
        !(entry.before === null || typeof entry.before === "string") ||
        !(
          entry.after_hash === null ||
          /^sha256:[a-f0-9]{64}$/.test(entry.after_hash)
        )
      )
        throw PrayError.render("invalid project recovery entry");
      validateDestinationPath(entry.path);
      entries.push(entry);
      continue;
    }
    if (
      record.type === "undone" &&
      !committed &&
      Number.isInteger(record.index) &&
      record.index >= 0 &&
      record.index < entries.length
    ) {
      undone.add(record.index);
      continue;
    }
    if (record.type === "commit" && index > 0 && !undone.size && !committed) {
      committed = true;
      continue;
    }
    throw PrayError.render("invalid project recovery journal state");
  }
  return { entries, undone, committed };
}
