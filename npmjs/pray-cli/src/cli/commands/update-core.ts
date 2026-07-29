import { existsSync } from "node:fs";
import { PrayError } from "../../errors.js";
import {
  buildLockfile,
  readLockfile,
  writeLockfile,
  writeLockfileIfChanged,
} from "../../lockfile/index.js";
import type { Lockfile } from "../../lockfile/types.js";
import { renderProject, writeRenderedTargets } from "../../render/project.js";
import { defaultResolveOptions } from "../../resolve/context.js";
import type { ResolvedProject } from "../../resolve/types.js";
import { lockfilePath, resolveCurrentProject } from "../invocation.js";
import {
  buildUpdateSummary,
  constraintBlockedPackagesJson,
  mergeSelectedPackageUpdate,
  printConstraintBlockedPackages,
  printUpdateSummary,
} from "./update-report.js";

export async function previewRemoteUpdates(
  selectedPackage: string | undefined,
  json: boolean,
): Promise<void> {
  if (json) {
    throw PrayError.unsupported("--json is not supported with --dry-run");
  }
  const previous = existsSync(lockfilePath())
    ? readLockfile(lockfilePath())
    : undefined;
  const project = await resolveCurrentProject({
    ...defaultResolveOptions(),
    refreshSourceRevisions: true,
    ignoreLockedVersions: true,
  });
  const rendered = renderProject(project);
  const updated = buildLockfile({
    manifestHash: project.manifestHash,
    projectRoot: project.projectRoot,
    manifestSources: project.manifest.sources,
    manifestTargets: project.manifest.targets,
    rendered,
    packages: project.packages,
    sourceRevisions: project.sourceRevisions,
    sourceHostKeys: project.sourceHostKeys,
  });
  if (
    printUpdateSummary(
      previous,
      updated,
      selectedPackage,
      project,
      "Remote update preview",
    )
  ) {
    printConstraintBlockedPackages(project, "Remote update preview", false);
    return;
  }
  if (printConstraintBlockedPackages(project, "Outdated packages", true)) {
    return;
  }
  process.stdout.write("Outdated packages\n");
  process.stdout.write("All packages up to date\n");
}

export async function updateWithManifestConstraints(
  packageName: string | undefined,
  json: boolean,
  manifestConstraintUpdates: Array<{
    name: string;
    from_constraint: string;
    to_constraint: string;
    registry_latest_version: string;
  }>,
): Promise<void> {
  const projectCheck = await resolveCurrentProject();
  if (
    packageName &&
    !projectCheck.manifest.packages.some((entry) => entry.name === packageName)
  ) {
    throw PrayError.manifest(`package ${packageName} not found`);
  }

  const previous = existsSync(lockfilePath())
    ? readLockfile(lockfilePath())
    : undefined;
  const project = await resolveCurrentProject({
    ...defaultResolveOptions(),
    refreshSourceRevisions: true,
    ignoreLockedVersions: packageName === undefined,
    unlockedPackages: packageName ? new Set([packageName]) : new Set(),
  });
  const rendered = renderProject(project);
  const updatedLockfile = buildLockfile({
    manifestHash: project.manifestHash,
    projectRoot: project.projectRoot,
    manifestSources: project.manifest.sources,
    manifestTargets: project.manifest.targets,
    rendered,
    packages: project.packages,
    sourceRevisions: project.sourceRevisions,
    sourceHostKeys: project.sourceHostKeys,
  });
  const merged =
    previous && packageName
      ? mergeSelectedPackageUpdate(previous, updatedLockfile, packageName)
      : updatedLockfile;
  if (packageName) {
    writeLockfile(lockfilePath(), merged);
  } else {
    writeLockfileIfChanged(lockfilePath(), merged);
  }
  writeRenderedTargets(project, rendered);

  if (json) {
    printUpdateJsonReport(
      manifestConstraintUpdates,
      previous,
      merged,
      packageName,
      project,
    );
    return;
  }
  const updateReported = printUpdateSummary(
    previous,
    merged,
    packageName,
    project,
    "Update summary",
  );
  printConstraintBlockedPackages(project, "Update summary", !updateReported);
}

export function printUpdateJsonReport(
  manifestConstraintUpdates: Array<{
    name: string;
    from_constraint: string;
    to_constraint: string;
    registry_latest_version: string;
  }>,
  previous: Lockfile | undefined,
  updated: Lockfile | undefined,
  selectedPackage: string | undefined,
  project: ResolvedProject,
): void {
  const summary = buildUpdateSummary(
    previous,
    updated ?? {
      prayfile_lock: "1",
      spec: "prayfile-1",
      generated_by: "",
      manifest_hash: "",
      source: [],
      package: [],
      target: [],
      managed_span: [],
    },
    selectedPackage,
    project,
  );
  const constraintBlocked = constraintBlockedPackagesJson(project);
  const status =
    manifestConstraintUpdates.length === 0 &&
    summary.updatedPackages.length === 0 &&
    constraintBlocked.length === 0
      ? "up_to_date"
      : "updated";
  process.stdout.write(
    `${JSON.stringify(
      {
        status,
        manifest_constraint_updates: manifestConstraintUpdates,
        updated_packages: summary.updatedPackages,
        constraint_blocked_packages: constraintBlocked,
      },
      null,
      2,
    )}\n`,
  );
}
