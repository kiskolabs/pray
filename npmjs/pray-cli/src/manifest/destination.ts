import { PrayError } from "../errors.js";
import { type LiteralValue, literalAsBool } from "../literal/types.js";
import type {
  DestinationEntry,
  DestinationMode,
  ExportRole,
  Manifest,
  ManifestLocal,
  ManifestPackage,
  ManifestTarget,
} from "./types.js";

export function targetMode(target: ManifestTarget): DestinationMode {
  return target.mode ?? "legacy";
}

export function targetScoped(target: ManifestTarget): boolean {
  return target.scoped ?? false;
}

export function targetEntries(target: ManifestTarget): DestinationEntry[] {
  return target.entries ?? [];
}

export function packageRoles(packageEntry: ManifestPackage): ExportRole[] {
  return packageEntry.roles ?? [];
}

export function packageBound(packageEntry: ManifestPackage): boolean {
  return packageEntry.bound ?? false;
}

export function localBound(local: ManifestLocal): boolean {
  return local.bound ?? false;
}

export function isLocalPathForm(value: string): boolean {
  return (
    value.startsWith(".") ||
    value.startsWith("/") ||
    value.endsWith(".md") ||
    value.endsWith(".txt") ||
    value.endsWith(".markdown") ||
    !value.includes("/")
  );
}

export function destinationTargetName(
  mode: DestinationMode,
  path: string,
): string {
  const prefix =
    mode === "compose" ? "compose" : mode === "tree" ? "tree" : "legacy";
  return `${prefix}:${path}`;
}

export function newDestinationTarget(
  mode: DestinationMode,
  path: string,
): ManifestTarget {
  const target: ManifestTarget = {
    name: destinationTargetName(mode, path),
    outputs: [],
    skills: [],
    commands: [],
    rules: [],
    mode,
    scoped: true,
    entries: [],
  };
  if (mode === "compose") {
    target.outputs.push(path);
  } else if (mode === "tree") {
    target.skills.push(path);
  }
  return target;
}

export function upsertPackage(
  manifest: Manifest,
  packageEntry: ManifestPackage,
): void {
  const existing = manifest.packages.find(
    (candidate) => candidate.name === packageEntry.name,
  );
  if (!existing) {
    manifest.packages.push(packageEntry);
    return;
  }
  if (
    existing.constraint !== packageEntry.constraint &&
    existing.constraint !== "*" &&
    packageEntry.constraint !== "*"
  ) {
    throw PrayError.manifest(
      `package ${packageEntry.name} declared with conflicting constraints (${existing.constraint} vs ${packageEntry.constraint})`,
    );
  }
  if (existing.constraint === "*" && packageEntry.constraint !== "*") {
    existing.constraint = packageEntry.constraint;
  }
  if (!existing.source) {
    existing.source = packageEntry.source;
  } else if (packageEntry.source && existing.source !== packageEntry.source) {
    throw PrayError.manifest(
      `package ${packageEntry.name} declared with conflicting sources`,
    );
  }
  for (const exportName of packageEntry.exports) {
    if (!existing.exports.includes(exportName)) {
      existing.exports.push(exportName);
    }
  }
  const existingRoles = packageRoles(existing);
  for (const role of packageRoles(packageEntry)) {
    if (!existingRoles.includes(role)) {
      existingRoles.push(role);
    }
  }
  existing.roles = existingRoles;
  if (packageEntry.file) {
    if (existing.file && existing.file !== packageEntry.file) {
      throw PrayError.manifest(
        `package ${packageEntry.name} declared with conflicting file: destinations`,
      );
    }
    existing.file = packageEntry.file;
  }
  existing.bound = packageBound(existing) || packageBound(packageEntry);
  existing.optional = existing.optional || packageEntry.optional;
  if (!existing.path) {
    existing.path = packageEntry.path;
  }
  if (!existing.git) {
    existing.git = packageEntry.git;
  }
  if (!existing.tag) {
    existing.tag = packageEntry.tag;
  }
  if (!existing.rev) {
    existing.rev = packageEntry.rev;
  }
  if (!existing.tarball) {
    existing.tarball = packageEntry.tarball;
  }
  if (!existing.oci) {
    existing.oci = packageEntry.oci;
  }
  for (const group of packageEntry.groups) {
    if (!existing.groups.includes(group)) {
      existing.groups.push(group);
    }
  }
}

export function upsertLocal(manifest: Manifest, local: ManifestLocal): void {
  const existing = manifest.local.find(
    (candidate) => candidate.path === local.path,
  );
  if (!existing) {
    manifest.local.push(local);
    return;
  }
  existing.bound = localBound(existing) || localBound(local);
  existing.optional = existing.optional || local.optional;
  if (existing.position === "after" && local.position !== "after") {
    existing.position = local.position;
  }
}

export function bindPackageEntry(
  target: ManifestTarget,
  packageName: string,
): void {
  const entries = targetEntries(target);
  const exists = entries.some(
    (entry) => entry.kind === "package" && entry.name === packageName,
  );
  if (!exists) {
    entries.push({ kind: "package", name: packageName });
  }
  target.entries = entries;
}

export function bindLocalEntry(target: ManifestTarget, path: string): void {
  const entries = targetEntries(target);
  const exists = entries.some(
    (entry) => entry.kind === "local" && entry.path === path,
  );
  if (!exists) {
    entries.push({ kind: "local", path });
  }
  target.entries = entries;
}

export function roleForDestination(
  mode: DestinationMode,
): ExportRole | undefined {
  if (mode === "compose") {
    return "fragment";
  }
  if (mode === "tree") {
    return "folder";
  }
  return undefined;
}

export function packageBoundToCompose(
  packageEntry: ManifestPackage,
  target: ManifestTarget,
): boolean {
  if (targetScoped(target) && targetMode(target) === "compose") {
    return targetEntries(target).some(
      (entry) => entry.kind === "package" && entry.name === packageEntry.name,
    );
  }
  if (packageBound(packageEntry) || packageEntry.file) {
    return false;
  }
  return true;
}

export function packageBoundToTree(
  packageEntry: ManifestPackage,
  target: ManifestTarget,
): boolean {
  if (targetScoped(target) && targetMode(target) === "tree") {
    return targetEntries(target).some(
      (entry) => entry.kind === "package" && entry.name === packageEntry.name,
    );
  }
  if (packageBound(packageEntry) || packageEntry.file) {
    return false;
  }
  return true;
}

export function exportKindMatchesRole(kind: string, role: ExportRole): boolean {
  if (role === "fragment") {
    return kind === "fragment";
  }
  if (role === "folder") {
    return kind === "folder" || kind === "skill";
  }
  return kind === "file";
}

export function destinationHeaderKeyword(
  mode: DestinationMode,
  keywords: Map<string, LiteralValue>,
): boolean | undefined {
  const label =
    mode === "compose" ? "compose" : mode === "tree" ? "tree" : "destination";
  for (const key of keywords.keys()) {
    if (key !== "header") {
      throw PrayError.parse("manifest", `${label} does not accept ${key}`);
    }
  }
  if (!keywords.has("header")) {
    return undefined;
  }
  if (mode !== "compose") {
    throw PrayError.parse("manifest", `${label} does not accept header`);
  }
  const value = literalAsBool(keywords.get("header")!);
  if (value === undefined) {
    throw PrayError.parse("manifest", "header must be true or false");
  }
  return value;
}
