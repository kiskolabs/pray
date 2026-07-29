import { existsSync, readFileSync, writeFileSync } from "node:fs";
import {
  latestConstraintForPackage,
  versionSatisfies,
} from "../../constraint.js";
import { PrayError } from "../../errors.js";
import { readLockfile } from "../../lockfile/index.js";
import { replacePackageDeclaration } from "../../manifest/package-declaration.js";
import { defaultResolveOptions } from "../../resolve/context.js";
import {
  lockfilePath,
  manifestPath,
  resolveCurrentProject,
} from "../invocation.js";
import {
  printUpdateJsonReport,
  updateWithManifestConstraints,
} from "./update-core.js";

export async function updateLatestCommand(
  packageName: string | undefined,
  json: boolean,
): Promise<void> {
  const path = manifestPath();
  let manifestText = readFileSync(path, "utf8");
  const project = await resolveCurrentProject({
    ...defaultResolveOptions(),
    refreshSourceRevisions: true,
    ignoreLockedVersions: true,
  });
  if (
    packageName &&
    !project.manifest.packages.some((entry) => entry.name === packageName)
  ) {
    throw PrayError.manifest(`package ${packageName} not found`);
  }

  const manifestUpdates: Array<{
    name: string;
    from_constraint: string;
    to_constraint: string;
    registry_latest_version: string;
  }> = [];

  for (const resolved of project.packages) {
    if (packageName && resolved.declaration.name !== packageName) {
      continue;
    }
    const registryLatest = resolved.registryLatestVersion;
    if (!registryLatest) {
      continue;
    }
    if (versionSatisfies(registryLatest, resolved.declaration.constraint)) {
      continue;
    }
    const newConstraint = latestConstraintForPackage(
      resolved.declaration.constraint,
      registryLatest,
    );
    if (!versionSatisfies(registryLatest, newConstraint)) {
      throw PrayError.resolution(
        `derived constraint ${newConstraint} does not admit registry latest ${registryLatest} for ${resolved.declaration.name}`,
      );
    }
    manifestUpdates.push({
      name: resolved.declaration.name,
      from_constraint: resolved.declaration.constraint,
      to_constraint: newConstraint,
      registry_latest_version: registryLatest,
    });
    manifestText = replacePackageDeclaration(manifestText, {
      ...resolved.declaration,
      constraint: newConstraint,
    });
  }

  if (manifestUpdates.length === 0) {
    if (json) {
      const current = existsSync(lockfilePath())
        ? readLockfile(lockfilePath())
        : undefined;
      printUpdateJsonReport([], current, current, packageName, project);
      return;
    }
    process.stdout.write(
      "All package constraints already allow registry latest versions\n",
    );
  } else {
    if (!json) {
      for (const update of manifestUpdates) {
        process.stdout.write(
          `Prayfile: ${update.name} constraint ${update.from_constraint} -> ${update.to_constraint} (registry latest ${update.registry_latest_version})\n`,
        );
      }
    }
    writeFileSync(path, manifestText, "utf8");
  }

  await updateWithManifestConstraints(packageName, json, manifestUpdates);
}
