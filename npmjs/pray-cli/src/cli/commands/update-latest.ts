import { existsSync, readFileSync } from "node:fs";
import {
  latestConstraintForPackage,
  versionSatisfies,
} from "../../constraint.js";
import { PrayError } from "../../errors.js";
import { readLockfile } from "../../lockfile/index.js";
import { parseManifest } from "../../manifest/index.js";
import { replacePackageDeclaration } from "../../manifest/package-declaration.js";
import { resolveProject } from "../../resolve/project.js";
import {
  applyPathUpstreamLatestConstraints,
  type PathUpstreamLatestConstraint,
  planPathUpstreamLatestConstraints,
} from "../../resolve/upstream-latest.js";
import { applyPathUpstreamRefreshes } from "../../resolve/upstream-refresh.js";
import {
  lockfilePath,
  manifestPath,
  resolveCurrentProject,
} from "../invocation.js";
import { updateResolveOptions, writeUpdate } from "./update-core.js";

export async function updateLatestCommand(
  packageName: string | undefined,
  json: boolean,
  dryRun = false,
): Promise<void> {
  const path = manifestPath();
  const originalText = readFileSync(path, "utf8");
  let manifestText = originalText;
  const options = updateResolveOptions(packageName);
  const project = await resolveCurrentProject(options);
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

  const previous = existsSync(lockfilePath())
    ? readLockfile(lockfilePath())
    : undefined;
  const upstreamPlans = await planPathUpstreamLatestConstraints(
    project,
    previous,
    packageName,
    options,
  );
  const upstreamConstraintUpdates = upstreamPlans.map((plan) => ({
    name: plan.packageName,
    from_constraint: plan.currentConstraint,
    to_constraint: plan.newConstraint,
    latest_version: plan.latestVersion,
  }));

  if (!json) {
    if (manifestUpdates.length === 0 && upstreamPlans.length === 0) {
      process.stdout.write(
        "All package constraints already allow latest versions\n",
      );
    } else {
      for (const update of manifestUpdates) {
        process.stdout.write(
          `Prayfile: ${update.name} constraint ${update.from_constraint} -> ${update.to_constraint} (registry latest ${update.registry_latest_version})\n`,
        );
      }
      printLatestUpstreamConstraints(upstreamPlans);
    }
  }

  let candidate = project;
  if (manifestUpdates.length > 0 || upstreamPlans.length > 0) {
    if (!dryRun) {
      applyPathUpstreamLatestConstraints(upstreamPlans);
    }
    candidate = await resolveProject(
      path,
      options,
      parseManifest(manifestText),
    );
    if (
      !dryRun &&
      (await applyPathUpstreamRefreshes(
        candidate,
        previous,
        packageName,
        options,
      ))
    ) {
      candidate = await resolveProject(
        path,
        options,
        parseManifest(manifestText),
      );
    }
  } else if (
    !dryRun &&
    (await applyPathUpstreamRefreshes(project, previous, packageName, options))
  ) {
    candidate = await resolveProject(
      path,
      options,
      parseManifest(manifestText),
    );
  }
  await writeUpdate(
    candidate,
    packageName,
    json,
    manifestUpdates,
    manifestText === originalText ? undefined : manifestText,
    dryRun,
    upstreamConstraintUpdates,
  );
}

function printLatestUpstreamConstraints(
  plans: readonly PathUpstreamLatestConstraint[],
): void {
  for (const plan of plans) {
    process.stdout.write(
      `${plan.packageName} upstream ${plan.currentConstraint} -> ${plan.newConstraint} (latest ${plan.latestVersion})\n`,
    );
  }
}
