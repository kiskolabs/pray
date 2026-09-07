import {
  closeSync,
  constants,
  existsSync,
  fchmodSync,
  fstatSync,
  fsyncSync,
  linkSync,
  lstatSync,
  mkdirSync,
  readdirSync,
  renameSync,
  unlinkSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { PrayError } from "../errors.js";
import { sha256Prefixed } from "../hashing.js";
import { validateDestinationPath } from "../manifest/validate.js";
import {
  destinationKind,
  openRegular,
  readRegularBytes,
} from "../render/destination-io.js";
import { ensureSafeDestinationAncestors } from "../render/path-guard.js";
import { privateFile, syncDirectory, token } from "./owner.js";
import { type JournalRecord, MAX_LOG, readJournal } from "./records.js";

const MAX_SAVED = 64 * 1024 * 1024;
const MAX_FILE = 32 * 1024 * 1024;
export class Journal {
  readonly path: string;
  closed = false;
  saved = 0;
  entries = 0;
  constructor(
    readonly root: string,
    readonly directory: string,
  ) {
    this.path = join(directory, "journal");
  }
  private append(record: JournalRecord): void {
    const bytes = Buffer.from(`${JSON.stringify(record)}\n`);
    const descriptor = openRegular(
      this.path,
      "project recovery journal",
      constants.O_WRONLY | constants.O_APPEND,
    );
    try {
      if (fstatSync(descriptor).size + bytes.length > MAX_LOG)
        throw PrayError.render(
          "project recovery journal exceeds its 96 MiB limit",
        );
      writeFileSync(descriptor, bytes);
      fsyncSync(descriptor);
    } finally {
      closeSync(descriptor);
    }
  }
  replace(
    path: string,
    before: Buffer | undefined,
    after: Buffer | undefined,
  ): void {
    if (this.closed)
      throw PrayError.render("the project write has already finished");
    if (equal(before, after)) return;
    const display = relative(this.root, resolve(path)).replaceAll("\\", "/");
    this.destination(display);
    if ((before?.length ?? 0) > MAX_FILE || (after?.length ?? 0) > MAX_FILE)
      throw PrayError.render("destination exceeds the 32 MiB limit");
    if (
      this.saved + (before?.length ?? 0) + (after?.length ?? 0) > MAX_SAVED ||
      this.entries === 10000
    )
      throw PrayError.render(
        "project write exceeds the 64 MiB or 10000-file recovery limit",
      );
    if (!equal(snapshot(path), before)) throw changed(display);
    const mode = before === undefined ? 0o644 : lstatSync(path).mode & 0o777;
    if (!existsSync(this.path)) {
      const descriptor = privateFile(this.path);
      try {
        fsyncSync(descriptor);
      } finally {
        closeSync(descriptor);
      }
      this.append({ type: "start", version: 1 });
      syncDirectory(this.directory);
    }
    this.append({
      type: "write",
      entry: {
        path: display,
        before: before?.toString("base64") ?? null,
        after_hash: after === undefined ? null : sha256Prefixed(after),
        mode,
      },
    });
    this.saved += (before?.length ?? 0) + (after?.length ?? 0);
    this.entries++;
    this.install(display, before, after, mode);
  }
  private destination(display: string): string {
    validateDestinationPath(display);
    if (
      display === ".pray/write-state" ||
      display.startsWith(".pray/write-state/")
    )
      throw PrayError.render("destination overlaps project recovery state");
    ensureSafeDestinationAncestors(this.root, display, display);
    return join(this.root, display);
  }
  private install(
    display: string,
    expected: Buffer | undefined,
    bytes: Buffer | undefined,
    mode: number,
  ): void {
    const path = this.destination(display);
    mkdirSync(dirname(path), { recursive: true });
    this.destination(display);
    const temporary = join(this.directory, `${token()}.stage`);
    if (bytes !== undefined) {
      const descriptor = privateFile(temporary);
      try {
        writeFileSync(descriptor, bytes);
        fchmodSync(descriptor, mode & 0o777);
        fsyncSync(descriptor);
      } finally {
        closeSync(descriptor);
      }
    }
    if (!equal(snapshot(path), expected)) throw changed(display);
    this.destination(display);
    if (bytes !== undefined && expected === undefined) {
      linkSync(temporary, path);
      unlinkSync(temporary);
    } else if (bytes !== undefined) renameSync(temporary, path);
    else if (expected !== undefined) unlinkSync(path);
    this.syncParents(path);
  }
  private syncParents(path: string): void {
    for (let parent = dirname(path); ; parent = dirname(parent)) {
      try {
        syncDirectory(parent);
      } catch (error) {
        if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
      }
      if (parent === this.root) break;
    }
  }
  recover(): void {
    this.cleanStages();
    if (!existsSync(this.path)) {
      return;
    }
    const { entries, undone, committed } = readJournal(this.path);
    if (!committed) {
      for (let index = entries.length - 1; index >= 0; index--) {
        if (undone.has(index)) continue;
        const entry = entries[index]!;
        const before =
          entry.before === null
            ? undefined
            : Buffer.from(entry.before, "base64");
        if (
          before &&
          (before.length > MAX_FILE ||
            before.toString("base64") !== entry.before)
        )
          throw PrayError.render("invalid recovery bytes");
        const current = snapshot(this.destination(entry.path));
        if (!equal(current, before)) {
          if (
            current !== undefined &&
            sha256Prefixed(current) !== entry.after_hash
          )
            throw changed(entry.path);
          this.install(entry.path, current, before, entry.mode);
        }
        // A prior recovery may have stopped before its directory sync completed.
        this.syncParents(this.destination(entry.path));
        this.append({ type: "undone", index });
      }
    }
    unlinkSync(this.path);
    syncDirectory(this.directory);
    this.saved = 0;
    this.entries = 0;
    this.cleanStages();
  }
  commit(): void {
    if (existsSync(this.path)) {
      this.append({ type: "commit" });
      unlinkSync(this.path);
      syncDirectory(this.directory);
    }
    this.cleanStages();
  }
  private cleanStages(): void {
    for (const name of readdirSync(this.directory))
      if (/^[a-f0-9]{32}\.stage$/.test(name))
        unlinkSync(join(this.directory, name));
  }
}
export function snapshot(path: string): Buffer | undefined {
  return destinationKind(path) === "missing"
    ? undefined
    : readRegularBytes(path, path);
}
function equal(left: Buffer | undefined, right: Buffer | undefined): boolean {
  return left === undefined
    ? right === undefined
    : right !== undefined && left.equals(right);
}
function changed(display: string): PrayError {
  return PrayError.render(
    `\`${display}\` changed during the interrupted write. Inspect your changes and move the file aside, then retry; recovery data remains in .pray/write-state`,
  );
}
