import {
  closeSync,
  constants,
  fstatSync,
  lstatSync,
  openSync,
  readSync,
  writeSync,
} from "node:fs";
import { PrayError } from "../errors.js";
import { replaceProjectFile } from "../transaction/hooks.js";

export type DestinationKind = "missing" | "regular" | "symlink" | "other";

export function createBytes(
  path: string,
  display: string,
  bytes: Buffer,
): void {
  if (replaceProjectFile(path, undefined, bytes)) return;
  const descriptor = openPath(
    path,
    display,
    constants.O_WRONLY | constants.O_CREAT | constants.O_EXCL,
  );
  try {
    writeAll(descriptor, bytes);
  } finally {
    closeSync(descriptor);
  }
}

export function readRegularBytes(path: string, display: string): Buffer {
  const descriptor = openRegular(path, display, constants.O_RDONLY);
  try {
    return readDestinationBytes(descriptor, display);
  } finally {
    closeSync(descriptor);
  }
}

export const MAX_DESTINATION_BYTES = 32 * 1024 * 1024;

export function readDestinationBytes(
  descriptor: number,
  display: string,
): Buffer {
  const fail = () =>
    PrayError.render(
      `refusing to read \`${display}\`; destination exceeds the 32 MiB limit`,
    );
  if (fstatSync(descriptor).size > MAX_DESTINATION_BYTES) throw fail();
  let capacity = Math.min(fstatSync(descriptor).size + 1, 64 * 1024);
  const chunks: Buffer[] = [];
  let size = 0;
  while (true) {
    const chunk = Buffer.allocUnsafe(
      Math.min(capacity, MAX_DESTINATION_BYTES + 1 - size),
    );
    const count = readSync(descriptor, chunk, 0, chunk.length, null);
    if (count === 0) break;
    size += count;
    if (size > MAX_DESTINATION_BYTES) throw fail();
    chunks.push(chunk.subarray(0, count));
    if (count === capacity) capacity = 64 * 1024;
  }
  return Buffer.concat(chunks, size);
}

export function openRegular(
  path: string,
  display: string,
  flags: number,
): number {
  const descriptor = openPath(path, display, flags);
  if (!fstatSync(descriptor).isFile()) {
    closeSync(descriptor);
    throw PrayError.render(
      `refusing to write \`${display}\`; destination is not a regular file`,
    );
  }
  return descriptor;
}

export function writeAll(descriptor: number, bytes: Buffer): void {
  let offset = 0;
  while (offset < bytes.length) {
    offset += writeSync(
      descriptor,
      bytes,
      offset,
      bytes.length - offset,
      offset,
    );
  }
}

export function destinationKind(path: string): DestinationKind {
  try {
    const metadata = lstatSync(path);
    if (metadata.isSymbolicLink()) return "symlink";
    if (metadata.isFile()) return "regular";
    return "other";
  } catch (error) {
    if (isCode(error, "ENOENT")) return "missing";
    throw error;
  }
}

function openPath(path: string, display: string, flags: number): number {
  try {
    return openSync(path, flags | constants.O_NOFOLLOW, 0o666);
  } catch (error) {
    if (isCode(error, "ELOOP")) {
      throw PrayError.render(
        `refusing to write \`${display}\` because it is a symbolic link`,
      );
    }
    throw error;
  }
}

function isCode(error: unknown, code: string): boolean {
  return (
    typeof error === "object" &&
    error !== null &&
    "code" in error &&
    (error as { code: string }).code === code
  );
}
