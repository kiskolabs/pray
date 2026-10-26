import { mkdirSync, readFileSync, statSync } from "node:fs";
import { dirname, join } from "node:path";
import { validateArchiveMemberPath } from "../archive/path-safety.js";
import { PrayError } from "../errors.js";
import type { PackageSpec } from "../package-spec/types.js";
import {
  MAX_ARCHIVE_ENTRIES,
  MAX_ARCHIVE_ENTRY_BYTES,
  MAX_ARCHIVE_TOTAL_BYTES,
} from "../resource-limits.js";
import { removeProjectFile, writeProjectFile } from "../transaction/index.js";
import { contentPaths } from "./upstream-merge.js";

export function localContentForRefresh(
  root: string,
  spec: PackageSpec,
): Map<string, Buffer> {
  const paths = contentPaths(spec.files);
  if (paths.length === 0) {
    return new Map();
  }
  let missing = 0;
  for (const relative of paths) {
    try {
      if (!statSync(join(root, relative)).isFile()) {
        missing += 1;
      }
    } catch {
      missing += 1;
    }
  }
  if (missing === paths.length) {
    return new Map();
  }
  return contentFileBytes(root, spec);
}

export function contentFileBytes(
  root: string,
  spec: PackageSpec,
): Map<string, Buffer> {
  const paths = contentPaths(spec.files);
  if (paths.length > MAX_ARCHIVE_ENTRIES) {
    throw PrayError.integrity(
      `package content exceeds ${MAX_ARCHIVE_ENTRIES} files`,
    );
  }
  const files = new Map<string, Buffer>();
  let totalBytes = 0;
  for (const relative of paths) {
    validateArchiveMemberPath(relative);
    const path = join(root, relative);
    let size: number;
    try {
      size = statSync(path).size;
    } catch {
      throw PrayError.integrity(`package file missing: ${relative}`);
    }
    if (size > MAX_ARCHIVE_ENTRY_BYTES) {
      throw PrayError.integrity(
        `package file exceeds ${MAX_ARCHIVE_ENTRY_BYTES} bytes: ${relative}`,
      );
    }
    totalBytes += size;
    if (totalBytes > MAX_ARCHIVE_TOTAL_BYTES) {
      throw PrayError.integrity(
        `package content exceeds ${MAX_ARCHIVE_TOTAL_BYTES} bytes`,
      );
    }
    files.set(relative, readFileSync(path));
  }
  return files;
}

export function writeContentFiles(
  root: string,
  oldContent: Map<string, Buffer>,
  merged: Map<string, Buffer>,
): void {
  for (const path of [...oldContent.keys(), ...merged.keys()]) {
    validateArchiveMemberPath(path);
  }
  for (const path of oldContent.keys()) {
    if (!merged.has(path)) {
      removeProjectFile(join(root, path));
    }
  }
  for (const [relative, bytes] of merged) {
    const destination = join(root, relative);
    mkdirSync(dirname(destination), { recursive: true });
    writeProjectFile(destination, bytes);
  }
}
