import { readFileSync } from "node:fs";
import { join } from "node:path";
import { PrayError } from "../errors.js";
import { sha256Prefixed } from "../hashing.js";
import type { PackageSpec } from "./types.js";

export function treeHashFromFileBytes(fileBytes: Map<string, Buffer>): string {
  const entries = [...fileBytes.entries()]
    .map(([path, bytes]) => [path, sha256Prefixed(bytes)] as const)
    .sort(([left], [right]) => left.localeCompare(right));

  let serialized = "";
  for (const [path, hash] of entries) {
    serialized += `file\0regular\0${path}\0${hash}\n`;
  }
  return sha256Prefixed(serialized);
}

export function treeHashForRoot(root: string, spec: PackageSpec): string {
  const fileBytes = new Map<string, Buffer>();
  for (const file of spec.files) {
    const path = join(root, file);
    try {
      const bytes = readFileSync(path);
      fileBytes.set(file, bytes);
    } catch {
      throw PrayError.integrity(`package file missing: ${file}`);
    }
  }
  return treeHashFromFileBytes(fileBytes);
}
