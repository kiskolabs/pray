import { isAbsolute, win32 } from "node:path";
import { PrayError } from "../errors.js";
import type { Manifest } from "./types.js";

export function validateManifestPaths(manifest: Manifest): void {
  for (const target of manifest.targets) {
    for (const path of [
      ...target.outputs,
      ...target.skills,
      ...target.commands,
      ...target.rules,
    ]) {
      validateProjectRelativePath(path);
    }
  }
  for (const packageEntry of manifest.packages) {
    if (packageEntry.path) validateProjectRelativePath(packageEntry.path);
    if (packageEntry.file) validateProjectRelativePath(packageEntry.file);
  }
  for (const local of manifest.local) validateProjectRelativePath(local.path);
}

export function validateProjectRelativePath(value: string): string {
  const path = value.trim();
  if (path.length === 0 || isAbsolute(path) || win32.isAbsolute(path)) {
    throw PrayError.manifest(
      `project path must be repository-relative: ${value}`,
    );
  }
  const parts = path.replaceAll("\\", "/").split("/");
  if (parts.includes("..") || path.includes("\0")) {
    throw PrayError.manifest(`project path escapes repository root: ${value}`);
  }
  if (parts.every((part) => part.length === 0 || part === ".")) {
    throw PrayError.manifest(
      `project path must be repository-relative: ${value}`,
    );
  }
  return path;
}
