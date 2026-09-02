import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { PrayError } from "../errors.js";
import { normalizeLineEndings } from "../hashing.js";
import {
  exportKindMatchesRole,
  packageRoles,
} from "../manifest/destination.js";
import type { ExportRole, ManifestPackage } from "../manifest/types.js";
import type { PackageSpec } from "../package-spec/types.js";

export function selectExports(
  declaration: ManifestPackage,
  spec: PackageSpec,
): string[] {
  if (declaration.exports.length > 0) {
    for (const exportName of declaration.exports) {
      if (!spec.exports.has(exportName)) {
        throw PrayError.resolution(
          `package ${declaration.name} does not export ${exportName}`,
        );
      }
    }
    return [...declaration.exports];
  }

  const roles = packageRoles(declaration);
  if (roles.length === 0 && !declaration.file) {
    return [...spec.exports.keys()].sort();
  }

  const effectiveRoles = [...roles];
  if (declaration.file && !effectiveRoles.includes("file")) {
    effectiveRoles.push("file");
  }

  const selected: string[] = [];
  for (const role of effectiveRoles) {
    const compatible =
      role === "fragment"
        ? fragmentRoleExports(spec)
        : compatibleExportNames(spec, role);
    if (compatible.length === 1) {
      const name = compatible[0]!;
      if (!selected.includes(name)) {
        selected.push(name);
      }
    } else if (compatible.length === 0) {
      throw PrayError.resolution(
        `package ${declaration.name} has no export compatible with ${role}`,
      );
    } else {
      throw PrayError.resolution(
        `package ${declaration.name} has multiple exports compatible with ${role}; set export: "name"`,
      );
    }
  }
  return selected;
}

export function loadExportBodies(
  root: string,
  spec: PackageSpec,
  selectedExports: string[],
): Map<string, string> {
  const exportBodies = new Map<string, string>();
  for (const exportName of selectedExports) {
    const entry = spec.exports.get(exportName);
    if (!entry) {
      throw PrayError.resolution(
        `package ${spec.name} is missing export ${exportName}`,
      );
    }
    if (entry.kind !== "fragment" && entry.kind !== "file") {
      continue;
    }
    const filePath = join(root, entry.path);
    if (!existsSync(filePath)) {
      throw PrayError.integrity(
        `package file missing for export ${exportName}: ${entry.path}`,
      );
    }
    const decoded = utf8Text(readFileSync(filePath));
    if (decoded === undefined) {
      if (entry.kind === "fragment") {
        throw PrayError.integrity(
          `package file is not valid utf-8 for export ${exportName}`,
        );
      }
      continue;
    }
    exportBodies.set(exportName, normalizeLineEndings(decoded));
  }
  return exportBodies;
}

function fragmentRoleExports(spec: PackageSpec): string[] {
  const fragments = [...spec.exports.entries()]
    .filter(([, exportEntry]) => exportEntry.kind === "fragment")
    .map(([name]) => name);
  if (fragments.length > 0) {
    return fragments;
  }
  return [...spec.exports.entries()]
    .filter(([, exportEntry]) => exportEntry.kind === "file")
    .map(([name]) => name);
}

function compatibleExportNames(spec: PackageSpec, role: ExportRole): string[] {
  return [...spec.exports.entries()]
    .filter(([, exportEntry]) => exportKindMatchesRole(exportEntry.kind, role))
    .map(([name]) => name);
}

function utf8Text(bytes: Buffer): string | undefined {
  try {
    return new TextDecoder("utf-8", { fatal: true }).decode(bytes);
  } catch {
    return undefined;
  }
}
