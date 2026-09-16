import { extname } from "node:path";
import { PrayError } from "../errors.js";

export function isIdentityPath(path: string): boolean {
  return extname(path) === ".prayspec";
}

export function contentPaths(files: readonly string[]): string[] {
  return files.filter((path) => !isIdentityPath(path));
}

export function isCleanReplica(
  oldContent: Map<string, Buffer>,
  localContent: Map<string, Buffer>,
): boolean {
  if (oldContent.size !== localContent.size) {
    return false;
  }
  for (const [path, bytes] of oldContent.entries()) {
    const local = localContent.get(path);
    if (local === undefined || !local.equals(bytes)) {
      return false;
    }
  }
  return true;
}

export function tryMergeContentFiles(
  oldContent: Map<string, Buffer>,
  newContent: Map<string, Buffer>,
  localContent: Map<string, Buffer>,
): Map<string, Buffer> | string[] {
  if (isCleanReplica(oldContent, localContent)) {
    return new Map(newContent);
  }
  const paths = new Set([
    ...oldContent.keys(),
    ...newContent.keys(),
    ...localContent.keys(),
  ]);
  const merged = new Map<string, Buffer>();
  const conflicts: string[] = [];
  for (const path of [...paths].sort()) {
    const oldBytes = oldContent.get(path);
    const newBytes = newContent.get(path);
    const localBytes = localContent.get(path);
    if (
      newBytes !== undefined &&
      localBytes !== undefined &&
      localBytes.equals(newBytes)
    ) {
      merged.set(path, newBytes);
    } else if (
      oldBytes !== undefined &&
      newBytes !== undefined &&
      localBytes !== undefined &&
      localBytes.equals(oldBytes)
    ) {
      merged.set(path, newBytes);
    } else if (
      oldBytes !== undefined &&
      newBytes !== undefined &&
      localBytes !== undefined &&
      oldBytes.equals(newBytes)
    ) {
      merged.set(path, localBytes);
    } else if (
      oldBytes !== undefined &&
      newBytes === undefined &&
      localBytes !== undefined &&
      localBytes.equals(oldBytes)
    ) {
    } else if (
      oldBytes !== undefined &&
      newBytes !== undefined &&
      localBytes === undefined &&
      oldBytes.equals(newBytes)
    ) {
    } else if (
      oldBytes === undefined &&
      newBytes !== undefined &&
      localBytes === undefined
    ) {
      merged.set(path, newBytes);
    } else if (
      oldBytes === undefined &&
      newBytes === undefined &&
      localBytes !== undefined
    ) {
      merged.set(path, localBytes);
    } else if (
      oldBytes !== undefined &&
      newBytes !== undefined &&
      localBytes === undefined
    ) {
      merged.set(path, newBytes);
    } else {
      conflicts.push(path);
    }
  }
  return conflicts.length === 0 ? merged : conflicts;
}

export function mergeContentFiles(
  oldContent: Map<string, Buffer>,
  newContent: Map<string, Buffer>,
  localContent: Map<string, Buffer>,
): Map<string, Buffer> {
  const merged = tryMergeContentFiles(oldContent, newContent, localContent);
  if (merged instanceof Map) {
    return merged;
  }
  throw PrayError.resolution(`upstream merge conflict in ${merged.join(", ")}`);
}

export function upstreamMergeConflictMessage(
  fork: string,
  upstreamName: string,
  oldVersion: string,
  newVersion: string,
  paths: readonly string[],
): string {
  return `upstream merge conflict in ${fork} while refreshing ${upstreamName} ${oldVersion} to ${newVersion}: ${paths.join(", ")}`;
}

export type OverlayFileChange = "changed" | "local_only" | "missing";

export function overlayFileChanges(
  localContent: Map<string, Buffer>,
  upstreamContent: Map<string, Buffer>,
): Array<[string, OverlayFileChange]> {
  const paths = new Set([...localContent.keys(), ...upstreamContent.keys()]);
  const changes: Array<[string, OverlayFileChange]> = [];
  for (const path of [...paths].sort()) {
    const local = localContent.get(path);
    const upstream = upstreamContent.get(path);
    if (
      local !== undefined &&
      upstream !== undefined &&
      !local.equals(upstream)
    ) {
      changes.push([path, "changed"]);
    } else if (local !== undefined && upstream === undefined) {
      changes.push([path, "local_only"]);
    } else if (local === undefined && upstream !== undefined) {
      changes.push([path, "missing"]);
    }
  }
  return changes;
}

export function overlayDriftLine(
  fork: string,
  upstreamName: string,
  upstreamVersion: string,
  path: string,
  change: OverlayFileChange,
): string {
  if (change === "changed") {
    return `${fork} ${path} differs from ${upstreamName} ${upstreamVersion}`;
  }
  if (change === "local_only") {
    return `${fork} ${path} local`;
  }
  return `${fork} ${path} missing from fork`;
}

export function nextUpstreamConstraint(
  current: string,
  newVersion: string,
): string {
  const trimmed = current.trim();
  if (
    trimmed === "*" ||
    trimmed.startsWith("~>") ||
    trimmed.startsWith("^") ||
    trimmed.startsWith(">=") ||
    trimmed.startsWith(">") ||
    trimmed.startsWith("<=") ||
    trimmed.startsWith("<")
  ) {
    return current;
  }
  return `= ${newVersion}`;
}
