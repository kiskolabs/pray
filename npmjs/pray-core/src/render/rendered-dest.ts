import { closeSync, constants, ftruncateSync, writeSync } from "node:fs";
import { TextDecoder } from "node:util";
import { PrayError } from "../errors.js";
import { replaceProjectFile } from "../transaction/hooks.js";
import {
  createBytes,
  destinationKind,
  openRegular,
  readDestinationBytes,
  readRegularBytes,
} from "./destination-io.js";
import { patchRenderedContent } from "./patch.js";

export function layoutRenderedContent(
  path: string,
  display: string,
  fresh: string,
): string {
  const kind = destinationKind(path);
  if (kind === "missing") return fresh;
  if (kind === "symlink") {
    throw PrayError.render(
      `refusing to write \`${display}\` because it is a symbolic link`,
    );
  }
  if (kind !== "regular") {
    throw PrayError.render(
      `refusing to write \`${display}\`; destination is not a regular file`,
    );
  }
  return patchRenderedContent(
    decodeUtf8(readRegularBytes(path, display)),
    fresh,
  );
}

export function writeRenderedContent(
  path: string,
  display: string,
  fresh: string,
): void {
  if (destinationKind(path) === "missing") {
    createBytes(path, display, Buffer.from(fresh, "utf8"));
    return;
  }
  const descriptor = openRegular(path, display, constants.O_RDWR);
  try {
    const existing = decodeUtf8(readDestinationBytes(descriptor, display));
    const bytes = Buffer.from(patchRenderedContent(existing, fresh), "utf8");
    if (replaceProjectFile(path, Buffer.from(existing, "utf8"), bytes)) return;
    ftruncateSync(descriptor, 0);
    writeAll(descriptor, bytes);
  } finally {
    closeSync(descriptor);
  }
}

function decodeUtf8(bytes: Buffer): string {
  try {
    return new TextDecoder("utf-8", { fatal: true }).decode(bytes);
  } catch {
    throw PrayError.render("rendered destination is not valid UTF-8");
  }
}

function writeAll(descriptor: number, bytes: Buffer): void {
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
