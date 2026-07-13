import type { Lockfile } from "../lockfile/types.js";

export interface ResolveOptions {
  offline: boolean;
  refreshSourceRevisions: boolean;
  ignoreLockedVersions: boolean;
  unlockedPackages: Set<string>;
}

export const defaultResolveOptions = (): ResolveOptions => ({
  offline: false,
  refreshSourceRevisions: false,
  ignoreLockedVersions: false,
  unlockedPackages: new Set(),
});

export function lockfilePreferredVersion(
  lockfile: Lockfile | undefined,
  packageName: string,
  options: ResolveOptions,
): string | undefined {
  if (!lockfile || options.ignoreLockedVersions) {
    return undefined;
  }
  if (options.unlockedPackages.has(packageName)) {
    return undefined;
  }
  return lockfile.package.find((entry) => entry.name === packageName)?.version;
}
