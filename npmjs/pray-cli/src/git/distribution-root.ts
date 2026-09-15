import { existsSync } from "node:fs";
import { join } from "node:path";
import { PrayError } from "../errors.js";

export function resolveDistributionRoot(
  repositoryRoot: string,
  subdir?: string,
): string {
  if (subdir) {
    const path = join(repositoryRoot, subdir);
    if (localDistributionRoot(path)) {
      return path;
    }
    throw PrayError.resolution(
      `no pray distribution root at subdir ${path} in git source ${repositoryRoot}`,
    );
  }
  const discovered = discoverDistributionRoot(repositoryRoot);
  if (discovered) {
    return discovered;
  }
  throw PrayError.resolution(
    `no pray distribution root in git source ${repositoryRoot}. ` +
      "Expected v1/packages at the repository root or under prayers/.",
  );
}

export function discoverDistributionRoot(path: string): string | undefined {
  if (localDistributionRoot(path)) {
    return path;
  }
  const prayersRoot = join(path, "prayers");
  if (localDistributionRoot(prayersRoot)) {
    return prayersRoot;
  }
  return undefined;
}

export function localDistributionRoot(path: string): boolean {
  return existsSync(join(path, "v1", "packages"));
}
