import { PrayError } from "../errors.js";
import {
  recordedPackageVersion,
  satisfyPackageConstraint,
} from "../package-spec/index.js";
import { findDependencyCycle } from "./dependency-graph.js";
import type { ResolvedPackage } from "./types.js";

export function rejectDependencyCycles(packages: ResolvedPackage[]): void {
  const edges = new Map<string, string[]>();
  for (const packageEntry of packages) {
    edges.set(
      packageEntry.declaration.name,
      packageEntry.spec.dependencies
        .filter((dependency) => !dependency.optional)
        .map((dependency) => dependency.name),
    );
  }
  const cycle = findDependencyCycle(edges);
  if (!cycle) {
    return;
  }
  throw PrayError.resolution(
    `dependency cycle detected: ${cycle.join(" -> ")}`,
  );
}

export function validateOptionalReferrals(packages: ResolvedPackage[]): void {
  const packagesByName = new Map(
    packages.map((packageEntry) => [
      packageEntry.declaration.name,
      packageEntry,
    ]),
  );
  for (const packageEntry of packages) {
    for (const dependency of packageEntry.spec.dependencies) {
      if (!dependency.optional) continue;
      const target = packagesByName.get(dependency.name);
      if (!target) continue;
      try {
        satisfyPackageConstraint(target.spec, dependency.constraint);
      } catch {
        throw PrayError.resolution(
          `optional referral ${packageEntry.declaration.name} -> ${dependency.name} requires ${dependency.constraint}, but selected version is ${recordedPackageVersion(target.spec)}`,
        );
      }
    }
  }
}
