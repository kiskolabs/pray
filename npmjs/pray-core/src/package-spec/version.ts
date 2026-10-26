import { normalizeVersionConstraint, versionSatisfies } from "../constraint.js";
import { PrayError } from "../errors.js";
import type { PackageSpec } from "./types.js";

export const LOCAL_PACKAGE_VERSION = "local";

export function packageHasReleaseVersion(spec: PackageSpec): boolean {
  return spec.version !== "" && spec.version !== LOCAL_PACKAGE_VERSION;
}

export function recordedPackageVersion(spec: PackageSpec): string {
  return packageHasReleaseVersion(spec) ? spec.version : LOCAL_PACKAGE_VERSION;
}

export function requireReleaseVersion(spec: PackageSpec): void {
  if (packageHasReleaseVersion(spec)) {
    return;
  }
  throw PrayError.manifest(
    `package ${spec.name} needs a version before it can be packaged`,
  );
}

export function satisfyPackageConstraint(
  spec: PackageSpec,
  constraint: string,
): void {
  if (!packageHasReleaseVersion(spec)) {
    const normalized = normalizeVersionConstraint(constraint);
    if (normalized.length === 0 || normalized === "*") {
      return;
    }
    throw PrayError.resolution(
      `package ${spec.name} has no version; add spec.version or omit the constraint`,
    );
  }
  if (!versionSatisfies(spec.version, constraint)) {
    throw PrayError.resolution(
      `package ${spec.name} version ${spec.version} does not satisfy constraint ${constraint}`,
    );
  }
}
