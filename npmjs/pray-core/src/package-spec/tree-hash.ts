import { PrayError } from "../errors.js";
import { sha256Prefixed } from "../hashing.js";
import { readSourceFile } from "../source-path.js";
import type { PackageSpec } from "./types.js";
import { validateRelativePaths } from "./validate-paths.js";

export function treeHashFromFileBytes(fileBytes: Map<string, Buffer>): string {
  const entries = [...fileBytes.entries()]
    .map(
      ([path, bytes]) =>
        [Buffer.from(path, "utf8"), path, sha256Prefixed(bytes)] as const,
    )
    .sort(([left], [right]) => Buffer.compare(left, right));

  let serialized = "";
  for (const [, path, hash] of entries) {
    serialized += `file\0regular\0${path}\0${hash}\n`;
  }
  return sha256Prefixed(serialized);
}

export function treeHashForRoot(root: string, spec: PackageSpec): string {
  validateRelativePaths(spec);
  const fileBytes = new Map<string, Buffer>();
  for (const file of spec.files) {
    try {
      const bytes = readSourceFile(root, file, "package file");
      fileBytes.set(file, bytes);
    } catch (error) {
      if (error instanceof PrayError) throw error;
      throw PrayError.integrity(`package file missing: ${file}`);
    }
  }
  return treeHashFromFileBytes(fileBytes);
}
