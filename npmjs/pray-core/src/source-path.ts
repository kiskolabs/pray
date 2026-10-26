import { lstatSync, readFileSync } from "node:fs";
import { join, parse, resolve } from "node:path";
import { PrayError } from "./errors.js";

export function declaredSourceDirectory(
  root: string,
  relativePath: string,
  label: string,
): string {
  rejectSourceSymlinks(root, relativePath, label);
  return resolve(root, relativePath);
}

export function rejectSourceSymlinks(
  root: string,
  relativePath: string,
  label: string,
): void {
  const pathRoot = parse(relativePath).root;
  let current = pathRoot || root;
  const pathToWalk = pathRoot
    ? relativePath.slice(pathRoot.length)
    : relativePath;
  for (const component of pathToWalk.split(/[\\/]/)) {
    if (component.length === 0 || component === ".") continue;
    current = join(current, component);
    try {
      if (lstatSync(current).isSymbolicLink()) {
        throw PrayError.integrity(
          `${label} uses a symbolic link: ${relativePath}`,
        );
      }
    } catch (error) {
      if (
        error instanceof Error &&
        "code" in error &&
        error.code === "ENOENT"
      ) {
        return;
      }
      throw error;
    }
  }
}

export function readSourceFile(
  root: string,
  relativePath: string,
  label: string,
): Buffer {
  rejectSourceSymlinks(root, relativePath, label);
  return readFileSync(join(root, relativePath));
}
