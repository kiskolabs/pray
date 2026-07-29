import { versionIsGreaterThan } from "../../constraint.js";
import type { Lockfile } from "../../lockfile/types.js";
import type { ResolvedProject } from "../../resolve/types.js";

export interface UpdatePackageEntry {
  name: string;
  from_version?: string;
  to_version: string;
}

export interface UpdateSummaryReport {
  lines: string[];
  updatedPackages: UpdatePackageEntry[];
}

export function buildUpdateSummary(
  previous: Lockfile | undefined,
  updated: Lockfile,
  selectedPackage: string | undefined,
  _project: ResolvedProject,
): UpdateSummaryReport {
  const previousByName = new Map(
    (previous?.package ?? []).map((entry) => [entry.name, entry]),
  );
  const lines: string[] = [];
  const updatedPackages: UpdatePackageEntry[] = [];

  if (previous) {
    for (const source of updated.source) {
      const previousRevision = previous.source.find(
        (entry) => entry.name === source.name,
      )?.revision;
      if (previousRevision !== source.revision) {
        lines.push(
          `Updated source ${source.name} revision ${previousRevision ?? "none"} -> ${source.revision ?? "none"}`,
        );
      }
    }
  }

  for (const packageEntry of updated.package) {
    if (selectedPackage && packageEntry.name !== selectedPackage) {
      continue;
    }
    const previousPackage = previousByName.get(packageEntry.name);
    if (!previousPackage) {
      lines.push(
        `Updated package ${packageEntry.name} (new) -> ${packageEntry.version}`,
      );
      updatedPackages.push({
        name: packageEntry.name,
        to_version: packageEntry.version,
      });
      continue;
    }
    if (previousPackage.version === packageEntry.version) {
      continue;
    }
    lines.push(
      `Updated package ${packageEntry.name} ${previousPackage.version} -> ${packageEntry.version}`,
    );
    updatedPackages.push({
      name: packageEntry.name,
      from_version: previousPackage.version,
      to_version: packageEntry.version,
    });
  }

  return { lines, updatedPackages };
}

export function constraintBlockedPackageLines(
  project: ResolvedProject,
): string[] {
  const lines: string[] = [];
  for (const packageEntry of project.packages) {
    const latest = packageEntry.registryLatestVersion;
    if (!latest || latest === packageEntry.spec.version) {
      continue;
    }
    if (!versionIsGreaterThan(latest, packageEntry.spec.version)) {
      continue;
    }
    lines.push(
      `Available package ${packageEntry.declaration.name} ${packageEntry.spec.version} -> ${latest} (blocked by ${packageEntry.declaration.constraint})`,
    );
  }
  return lines.sort();
}

export function constraintBlockedPackagesJson(project: ResolvedProject) {
  return project.packages
    .flatMap((packageEntry) => {
      const latest = packageEntry.registryLatestVersion;
      if (!latest || latest === packageEntry.spec.version) {
        return [];
      }
      if (!versionIsGreaterThan(latest, packageEntry.spec.version)) {
        return [];
      }
      return [
        {
          name: packageEntry.declaration.name,
          resolved_version: packageEntry.spec.version,
          registry_latest_version: latest,
          constraint: packageEntry.declaration.constraint,
        },
      ];
    })
    .sort((left, right) => left.name.localeCompare(right.name));
}

export function printUpdateSummary(
  previous: Lockfile | undefined,
  updated: Lockfile,
  selectedPackage: string | undefined,
  project: ResolvedProject,
  title: string,
): boolean {
  const report = buildUpdateSummary(
    previous,
    updated,
    selectedPackage,
    project,
  );
  if (report.lines.length === 0) {
    return false;
  }
  process.stdout.write(`${title}\n`);
  for (const line of report.lines) {
    process.stdout.write(`${line}\n`);
  }
  return true;
}

export function printConstraintBlockedPackages(
  project: ResolvedProject,
  title: string,
  printTitle: boolean,
): boolean {
  const lines = constraintBlockedPackageLines(project);
  if (lines.length === 0) {
    return false;
  }
  if (printTitle) {
    process.stdout.write(`${title}\n`);
  }
  for (const line of lines) {
    process.stdout.write(`${line}\n`);
  }
  return true;
}

export function mergeSelectedPackageUpdate(
  previous: Lockfile,
  updated: Lockfile,
  selectedPackage: string,
): Lockfile {
  return {
    ...updated,
    package: updated.package.map((packageEntry) => {
      if (packageEntry.name === selectedPackage) {
        return packageEntry;
      }
      const previousPackage = previous.package.find(
        (entry) => entry.name === packageEntry.name,
      );
      if (!previousPackage) {
        return packageEntry;
      }
      return { ...packageEntry, version: previousPackage.version };
    }),
  };
}
