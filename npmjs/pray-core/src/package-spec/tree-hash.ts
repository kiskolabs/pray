import { PrayError } from "../errors.js";
import { sha256Prefixed } from "../hashing.js";
import { readSourceFile } from "../source-path.js";
import type { PackageSpec } from "./types.js";
import { validateRelativePaths } from "./validate-paths.js";

export function treeHashFromFileBytes(fileBytes: Map<string, Buffer>): string {
  // Sort UTF-8 path bytes the same way as the prior Buffer.compare path.
  const paths = [...fileBytes.keys()].sort((left, right) =>
    Buffer.compare(Buffer.from(left, "utf8"), Buffer.from(right, "utf8")),
  );
  let serialized = "";
  for (const path of paths) {
    const bytes = fileBytes.get(path);
    if (bytes === undefined) {
      throw PrayError.integrity(`package file missing: ${path}`);
    }
    serialized += `file\0regular\0${path}\0${sha256Prefixed(bytes)}\n`;
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
