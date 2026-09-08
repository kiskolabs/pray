import { existsSync } from "node:fs";
import { PrayError } from "../errors.js";
import { readLockfile } from "../lockfile/index.js";
import type { Lockfile } from "../lockfile/types.js";

export function resolutionMayBenefitFromGitSourceRefresh(
  error: unknown,
): boolean {
  return (
    error instanceof PrayError &&
    error.kind === "resolution" &&
    catalogMissMessage(error.message)
  );
}

function catalogMissMessage(message: string): boolean {
  return (
    message.includes("no registry version") ||
    message.includes("not found in distribution") ||
    message.includes("not found in git source") ||
    message.includes("v1/packages/")
  );
}

export function annotateMissingGitCatalog(
  error: unknown,
  packageName: string,
  sourceName: string,
  revision: string,
): unknown {
  if (
    !(error instanceof PrayError) ||
    error.kind !== "resolution" ||
    revision.length === 0 ||
    !catalogMissMessage(error.message) ||
    error.message.includes("not found in git source")
  ) {
    return error;
  }
  return PrayError.resolution(
    `package ${packageName} was not found in git source ${sourceName} at revision ${revision}. ` +
      "Run `pray update` to advance the source pin.",
  );
}

export function annotateFailedGitRefresh(
  lockfilePath: string,
  error: unknown,
): unknown {
  if (
    !(error instanceof PrayError) ||
    error.kind !== "resolution" ||
    !catalogMissMessage(error.message)
  ) {
    return error;
  }
  const pins = lockedGitRevisionGuidance(lockfilePath);
  if (pins === undefined) {
    return error;
  }
  const packageName =
    packageNameFromCatalogMiss(error.message) ?? "the declared package";
  return PrayError.resolution(
    `package ${packageName} was not found in locked git source (${pins}). ` +
      "Run `pray update` to advance the source pin.",
  );
}

function lockedGitRevisionGuidance(lockfilePath: string): string | undefined {
  if (!existsSync(lockfilePath)) {
    return undefined;
  }
  let lockfile: Lockfile;
  try {
    lockfile = readLockfile(lockfilePath);
  } catch {
    return undefined;
  }
  const pins = lockfile.source
    .filter((source) => source.kind === "git" && source.revision)
    .map((source) => `${source.name} at ${source.revision}`);
  return pins.length === 0 ? undefined : pins.join(", ");
}

function packageNameFromCatalogMiss(message: string): string | undefined {
  if (!message.startsWith("package ")) {
    return undefined;
  }
  const rest = message.slice("package ".length);
  const ending =
    rest.indexOf(" was not found") === -1
      ? rest.indexOf(" not found")
      : rest.indexOf(" was not found");
  if (ending === -1) {
    return undefined;
  }
  const name = rest.slice(0, ending).trim();
  return name.length === 0 ? undefined : name;
}
