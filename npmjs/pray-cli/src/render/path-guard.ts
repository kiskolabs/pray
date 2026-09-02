import { lstatSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { PrayError } from "../errors.js";

export function ensureSafeDestinationAncestors(
  projectRoot: string,
  relativePath: string,
  display: string,
): void {
  const parent = dirname(relativePath.replaceAll("\\", "/"));
  if (parent === ".") return;
  let current = projectRoot;
  for (const component of parent.split("/")) {
    if (!component || component === ".") continue;
    current = resolve(current, component);
    try {
      const metadata = lstatSync(current);
      if (metadata.isSymbolicLink()) {
        throw PrayError.render(
          `refusing to write \`${display}\` because a destination parent is a symbolic link`,
        );
      }
      if (!metadata.isDirectory()) {
        throw PrayError.render(
          `refusing to write \`${display}\`; a destination parent is not a directory`,
        );
      }
    } catch (error) {
      if (!isNotFound(error)) throw error;
    }
  }
}

function isNotFound(error: unknown): boolean {
  return (
    typeof error === "object" &&
    error !== null &&
    "code" in error &&
    (error as { code: string }).code === "ENOENT"
  );
}
