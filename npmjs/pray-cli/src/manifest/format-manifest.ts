import { PrayError } from "../errors.js";
import type { ResolvedProject } from "../resolve/types.js";
import {
  bindLocalEntry,
  bindPackageEntry,
  exportKindMatchesRole,
  newDestinationTarget,
  packageBound,
  packageRoles,
} from "./destination.js";
import { serializeRecommended, targetHasExtras } from "./format-serialize.js";
import { parseManifestText } from "./parser.js";
import type {
  ExportRole,
  Manifest,
  ManifestLocal,
  ManifestPackage,
  ManifestTarget,
} from "./types.js";
import { canonicalManifest, manifestToJson } from "./types.js";

export interface PackageFormatHint {
  roles: ExportRole[];
  filePath?: string;
  /** Export names that must stay explicit after migration when a role is ambiguous. */
  exports?: string[];
}

export function usesDestinationDsl(manifest: Manifest): boolean {
  return (
    manifest.targets.some(
      (target) =>
        (target.scoped ?? false) || (target.mode ?? "legacy") !== "legacy",
    ) ||
    manifest.packages.some((entry) => packageBound(entry) || entry.file) ||
    manifest.local.some((entry) => entry.bound ?? false)
  );
}

function hasMigratableLegacyTargets(manifest: Manifest): boolean {
  return manifest.targets.some(
    (target) =>
      !(target.scoped ?? false) &&
      (target.mode ?? "legacy") === "legacy" &&
      (target.outputs.length > 0 || target.skills.length > 0),
  );
}

export function classifyFormatHints(
  project: ResolvedProject,
): Map<string, PackageFormatHint> {
  const hints = new Map<string, PackageFormatHint>();
  for (const packageEntry of project.packages) {
    const roles: ExportRole[] = [];
    let filePath = packageEntry.declaration.file;
    for (const exportName of packageEntry.selectedExports) {
      const exportEntry = packageEntry.spec.exports.get(exportName);
      if (!exportEntry) {
        continue;
      }
      for (const role of ["fragment", "folder", "file"] as ExportRole[]) {
        if (
          exportKindMatchesRole(exportEntry.kind, role) &&
          !roles.includes(role)
        ) {
          roles.push(role);
        }
      }
      if (!filePath && exportEntry.kind === "file") {
        filePath = exportEntry.defaultPath ?? exportName;
      }
    }
    const exports = ambiguousExportsForRoles(
      packageEntry.selectedExports,
      packageEntry.spec.exports,
      roles,
    );
    hints.set(packageEntry.declaration.name, { roles, filePath, exports });
  }
  return hints;
}

function ambiguousExportsForRoles(
  selectedExports: string[],
  exports: Map<string, { kind: string }>,
  roles: ExportRole[],
): string[] {
  const ambiguous: string[] = [];
  for (const role of roles) {
    const matching = selectedExports.filter((exportName) => {
      const exportEntry = exports.get(exportName);
      return (
        exportEntry !== undefined &&
        exportKindMatchesRole(exportEntry.kind, role)
      );
    });
    if (matching.length > 1) {
      for (const exportName of matching) {
        if (!ambiguous.includes(exportName)) {
          ambiguous.push(exportName);
        }
      }
    }
  }
  return ambiguous;
}

export function recommendManifest(
  manifest: Manifest,
  hints: Map<string, PackageFormatHint>,
): Manifest {
  // Top-level file: bindings alone must not skip migration of legacy target blocks.
  const recommended = hasMigratableLegacyTargets(manifest)
    ? migrateLegacyManifest(manifest, hints)
    : {
        ...manifest,
        packages: manifest.packages.map((entry) => ({
          ...entry,
          exports: [...entry.exports],
          targets: [...entry.targets],
          features: [...entry.features],
          groups: [...entry.groups],
          roles: [...(entry.roles ?? [])],
        })),
        local: manifest.local.map((entry) => ({ ...entry })),
        targets: manifest.targets.map((target) => ({
          ...target,
          outputs: [...target.outputs],
          skills: [...target.skills],
          commands: [...target.commands],
          rules: [...target.rules],
          entries: [...(target.entries ?? [])],
        })),
      };
  omitContextResolvedExports(recommended);
  omitDefaultSources(recommended);
  return recommended;
}

function omitContextResolvedExports(manifest: Manifest): void {
  for (const packageEntry of manifest.packages) {
    if (packageBound(packageEntry) && packageEntry.exports.length <= 1) {
      packageEntry.exports = [];
    }
  }
}

function packageNamespace(name: string): string | undefined {
  const separator = name.indexOf("/");
  return separator === -1 ? undefined : name.slice(0, separator);
}

function omitDefaultSources(manifest: Manifest): void {
  const soleSource =
    manifest.sources.length === 1 ? manifest.sources[0]?.name : undefined;
  const sourceNames = new Set(manifest.sources.map((source) => source.name));
  for (const packageEntry of manifest.packages) {
    const source = packageEntry.source;
    if (!source) {
      continue;
    }
    const matchesSole = soleSource === source;
    const matchesNamespace =
      packageNamespace(packageEntry.name) === source && sourceNames.has(source);
    if (matchesSole || matchesNamespace) {
      packageEntry.source = undefined;
    }
  }
}

export function formatRecommended(
  manifest: Manifest,
  hints: Map<string, PackageFormatHint>,
): string {
  const recommended = recommendManifest(manifest, hints);
  const text = serializeRecommended(recommended);
  const reparsed = parseManifestText(text);
  if (
    JSON.stringify(manifestToJson(reparsed)) !==
    JSON.stringify(manifestToJson(canonicalManifest(recommended)))
  ) {
    throw PrayError.manifest(
      "formatted Prayfile did not round-trip to an equivalent manifest",
    );
  }
  return text;
}

function migrateLegacyManifest(
  manifest: Manifest,
  hints: Map<string, PackageFormatHint>,
): Manifest {
  const next: Manifest = {
    prayfileVersion: manifest.prayfileVersion,
    sources: manifest.sources,
    targets: [],
    packages: manifest.packages.map((entry) => ({
      ...entry,
      exports: [...entry.exports],
      targets: [...entry.targets],
      features: [...entry.features],
      groups: [...entry.groups],
      roles: [...(entry.roles ?? [])],
    })),
    local: manifest.local.map((entry) => ({ ...entry })),
    render: manifest.render,
  };

  applyFormatHints(next.packages, hints);

  const composePaths = uniquePaths(
    manifest.targets.flatMap((target) =>
      target.outputs.map((path) => [path, target.name] as [string, string]),
    ),
  );
  const treePaths = uniquePaths(
    manifest.targets.flatMap((target) =>
      target.skills.map((path) => [path, target.name] as [string, string]),
    ),
  );

  for (const [path, targetNames] of composePaths) {
    const target = newDestinationTarget("compose", path);
    for (const local of localsForCompose(next.local)) {
      bindLocalEntry(target, local.path);
      const entry = next.local.find(
        (candidate) => candidate.path === local.path,
      );
      if (entry) {
        entry.bound = true;
      }
    }
    for (const packageEntry of packagesForRole(
      next.packages,
      "fragment",
      targetNames,
    )) {
      bindPackageEntry(target, packageEntry.name);
      markPackageBound(next.packages, packageEntry.name, "fragment");
    }
    next.targets.push(target);
  }

  for (const [path, targetNames] of treePaths) {
    const target = newDestinationTarget("tree", path);
    for (const packageEntry of packagesForRole(
      next.packages,
      "folder",
      targetNames,
    )) {
      bindPackageEntry(target, packageEntry.name);
      markPackageBound(next.packages, packageEntry.name, "folder");
    }
    next.targets.push(target);
  }

  for (const packageEntry of next.packages) {
    if (packageEntry.file) {
      packageEntry.bound = true;
      const roles = packageRoles(packageEntry);
      if (!roles.includes("file")) {
        roles.push("file");
      }
      packageEntry.roles = roles;
    }
  }
  for (const local of next.local) {
    if (local.bound) {
      local.position = "after";
    }
  }

  for (const target of manifest.targets) {
    if (targetHasExtras(target)) {
      next.targets.push({
        name: target.name,
        outputs: [],
        skills: [],
        commands: target.commands,
        rules: target.rules,
        maxBytes: target.maxBytes,
        mode: "legacy",
        scoped: false,
        entries: [],
      });
    }
  }

  return next;
}

function applyFormatHints(
  packages: ManifestPackage[],
  hints: Map<string, PackageFormatHint>,
): void {
  for (const packageEntry of packages) {
    const hint = hints.get(packageEntry.name);
    if (hint) {
      const roles = packageRoles(packageEntry);
      for (const role of hint.roles) {
        if (!roles.includes(role)) {
          roles.push(role);
        }
      }
      packageEntry.roles = roles;
      if (!packageEntry.file) {
        packageEntry.file = hint.filePath;
      }
      if (
        packageEntry.exports.length === 0 &&
        hint.exports &&
        hint.exports.length > 0
      ) {
        packageEntry.exports = [...hint.exports];
      }
    }
    if (packageEntry.file && !packageRoles(packageEntry).includes("file")) {
      const roles = packageRoles(packageEntry);
      roles.push("file");
      packageEntry.roles = roles;
    }
  }
}

function uniquePaths(
  items: Array<[string, string]>,
): Array<[string, Set<string>]> {
  const map = new Map<string, Set<string>>();
  for (const [path, targetName] of items) {
    const set = map.get(path) ?? new Set<string>();
    set.add(targetName);
    map.set(path, set);
  }
  return [...map.entries()].sort(([left], [right]) =>
    left.localeCompare(right),
  );
}

function localsForCompose(locals: ManifestLocal[]): ManifestLocal[] {
  const before: ManifestLocal[] = [];
  const after: ManifestLocal[] = [];
  for (const local of locals) {
    if (local.bound) {
      continue;
    }
    if (local.position === "before") {
      before.push(local);
    } else {
      after.push(local);
    }
  }
  return [...before, ...after];
}

function packagesForRole(
  packages: ManifestPackage[],
  role: ExportRole,
  targetNames: Set<string>,
): ManifestPackage[] {
  return packages.filter((packageEntry) => {
    if (packageEntry.file) {
      return false;
    }
    if (
      packageEntry.targets.length > 0 &&
      !packageEntry.targets.some((name) => targetNames.has(name))
    ) {
      return false;
    }
    return packageRoles(packageEntry).includes(role);
  });
}

function markPackageBound(
  packages: ManifestPackage[],
  name: string,
  role: ExportRole,
): void {
  const packageEntry = packages.find((entry) => entry.name === name);
  if (!packageEntry) {
    return;
  }
  packageEntry.bound = true;
  const roles = packageRoles(packageEntry);
  if (!roles.includes(role)) {
    roles.push(role);
  }
  packageEntry.roles = roles;
}

export type { ManifestTarget };
