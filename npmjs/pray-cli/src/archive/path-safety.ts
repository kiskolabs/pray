import { isAbsolute, normalize } from "node:path";
import { PrayError } from "../errors.js";

export function validateArchiveMemberPath(path: string): string {
  const cleaned = path.replace(/^\.\//, "");
  if (cleaned.length === 0 || isAbsolute(cleaned) || cleaned.startsWith("/")) {
    throw PrayError.integrity(`package path must be relative: ${path}`);
  }

  for (const part of normalize(cleaned).split(/[/\\]/)) {
    if (part === "" || part === ".") {
      continue;
    }
    if (part === ".." || part.includes("\0")) {
      throw PrayError.integrity(`package path escapes package root: ${path}`);
    }
  }

  return cleaned;
}
