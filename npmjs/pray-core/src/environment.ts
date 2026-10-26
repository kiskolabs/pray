import { PrayError } from "./errors.js";
import type { Manifest, ManifestPackage } from "./manifest/types.js";
import type { ResolvedPackage } from "./resolve/types.js";

export function packageMatchesEnvironment(
  groups: string[],
  environment: string | undefined,
): boolean {
  if (groups.length === 0) {
    return true;
  }
  if (environment === undefined) {
    return false;
  }
  return groups.includes(environment);
}

export function collectGroupNames(manifest: Manifest): Set<string> {
  const names = new Set<string>();
  for (const packageEntry of manifest.packages) {
    for (const group of packageEntry.groups) {
      names.add(group);
    }
  }
  return names;
}

export function validateEnvironment(
  manifest: Manifest,
  environment: string | undefined,
): void {
  if (environment === undefined) {
    return;
  }
  if (environment.length === 0) {
    throw PrayError.resolution("environment name cannot be empty");
  }
  const knownGroups = collectGroupNames(manifest);
  if (knownGroups.size === 0) {
    throw PrayError.resolution(
      `unknown environment ${environment}; Prayfile defines no groups`,
    );
  }
  if (!knownGroups.has(environment)) {
    const names = [...knownGroups].sort();
    throw PrayError.resolution(
      `unknown environment ${environment}; available groups are ${names.join(", ")}`,
    );
  }
}

export function packagesForRender(
  packages: ResolvedPackage[],
  environment: string | undefined,
): ResolvedPackage[] {
  return packages.filter((packageEntry) =>
    packageMatchesEnvironment(packageEntry.declaration.groups, environment),
  );
}

export function shouldRenderPackage(
  declaration: ManifestPackage,
  environment: string | undefined,
): boolean {
  return packageMatchesEnvironment(declaration.groups, environment);
}
