import { readFileSync } from "node:fs";
import { join } from "node:path";
import { PrayError } from "../errors.js";
import { sha256Prefixed } from "../hashing.js";
import type { PackageSpec } from "./types.js";

export function treeHashFromFileBytes(fileBytes: Map<string, Buffer>): string {
  // Sort by UTF-8 bytes to match the other implementations. Locale collation
  // orders "exports/rules.md" before "README.md" and would hash a package
  // differently here than where it was published.
  const entries = [...fileBytes.entries()]
    .map(([path, bytes]) => [path, sha256Prefixed(bytes)] as const)
    .sort(([left], [right]) =>
      Buffer.compare(Buffer.from(left, "utf8"), Buffer.from(right, "utf8")),
    );

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
