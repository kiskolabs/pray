import { isAbsolute, relative, resolve, win32 } from "node:path";
import { PrayError } from "../errors.js";

export function resolveDistributionPath(root: string, value: string): string {
  const path = value.trim().replaceAll("\\", "/");
  if (
    path.length === 0 ||
    isAbsolute(path) ||
    win32.isAbsolute(value) ||
    path.includes("\0") ||
    path.split("/").includes("..")
  ) {
    throw PrayError.integrity(`unsafe distribution path: ${value}`);
  }
  const absoluteRoot = resolve(root);
  const candidate = resolve(absoluteRoot, path);
  const fromRoot = relative(absoluteRoot, candidate);
  if (fromRoot.startsWith("..") || isAbsolute(fromRoot)) {
    throw PrayError.integrity(`distribution path escapes root: ${value}`);
  }
  return candidate;
}

export function validatePackageName(value: string): string {
  const parts = value.split("/");
  if (
    parts.length === 0 ||
    parts.some((part) => part.length === 0 || part === "." || part === "..") ||
    value.includes("\\") ||
    value.includes("\0")
  ) {
    throw PrayError.integrity(`invalid package name: ${value}`);
  }
  return value;
}

export function rejectAbsoluteArtifactPath(value: string): string {
  const path = value.trim();
  if (/^[a-z][a-z0-9+.-]*:/i.test(path)) {
    throw PrayError.integrity(
      `remote artifact path must be relative: ${value}`,
    );
  }
  return path;
}

export function validatePathSegment(value: string, label: string): string {
  if (
    value.length === 0 ||
    value === "." ||
    value === ".." ||
    value.includes("/") ||
    value.includes("\\") ||
    value.includes("\0")
  ) {
    throw PrayError.integrity(`invalid ${label}: ${value}`);
  }
  return value;
}
