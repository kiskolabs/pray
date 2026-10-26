import { isAbsolute } from "node:path";
import { PrayError } from "../errors.js";

export function validateArchiveMemberPath(path: string): string {
  const text = path.replaceAll("\\", "/");
  if (text.length === 0 || isAbsolute(text) || text.startsWith("/")) {
    throw PrayError.integrity(`package path must be relative: ${path}`);
  }

  const parts: string[] = [];
  for (const part of text.split("/")) {
    if (part === "" || part === ".") {
      continue;
    }
    if (part === ".." || part.includes("\0")) {
      throw PrayError.integrity(`package path escapes package root: ${path}`);
    }
    parts.push(part);
  }
  if (parts.length === 0) {
    throw PrayError.integrity(`package path must be relative: ${path}`);
  }

  return parts.join("/");
}
