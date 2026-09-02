import { existsSync } from "node:fs";
import { PrayError } from "../../errors.js";
import { buildLockfile, readLockfile } from "../../lockfile/index.js";
import {
  defaultLockfilePath,
  defaultManifestPath,
} from "../../lockfile/paths.js";
import { renderProject } from "../../render/project.js";
import { defaultResolveOptions } from "../../resolve/context.js";
import { resolveProject } from "../../resolve/project.js";
import { packageSourceSummary } from "../../tree/index.js";
import { previewRemoteUpdates } from "./update.js";
import {
  printConstraintBlockedPackages,
  printUpdateSummary,
} from "./update-report.js";

export async function runList(): Promise<void> {
  const project = await resolveProject(defaultManifestPath());
  const lines = ["Package list"];
  for (const packageEntry of project.packages) {
    lines.push(
      `${packageEntry.declaration.name} ${packageEntry.spec.version} source=${packageSourceSummary(packageEntry)} exports=${packageEntry.selectedExports.join(", ")}`,
    );
  }
  process.stdout.write(`${lines.join("\n")}\n`);
}

export function parseOutdatedArguments(argumentsList: string[]): {
  remote: boolean;
} {
  let remote = false;
  for (const argument of argumentsList) {
    if (argument === "--remote") {
      remote = true;
      continue;
    }
    throw PrayError.unsupported(`unknown outdated flag: ${argument}`);
  }
  return { remote };
}

export async function runOutdated(argumentsList: string[] = []): Promise<void> {
  const { remote } = parseOutdatedArguments(argumentsList);
  if (remote) {
    await previewRemoteUpdates(undefined, false);
    return;
  }

  const lockfilePath = defaultLockfilePath(process.cwd());
  const previous = existsSync(lockfilePath)
    ? readLockfile(lockfilePath)
    : undefined;
  const project = await resolveProject(defaultManifestPath(), {
    ...defaultResolveOptions(),
    refreshSourceRevisions: true,
    ignoreLockedVersions: true,
  });
  const rendered = renderProject(project);
  const latest = buildLockfile({
    manifestHash: project.manifestHash,
    projectRoot: project.projectRoot,
    manifestSources: project.manifest.sources,
    manifestTargets: project.manifest.targets,
    rendered,
    packages: project.packages,
    sourceRevisions: project.sourceRevisions,
    sourceHostKeys: project.sourceHostKeys,
    project,
  });
  let reported = printUpdateSummary(
    previous,
    latest,
    undefined,
    project,
    "Outdated packages",
  );
  reported =
    printConstraintBlockedPackages(project, "Outdated packages", !reported) ||
    reported;
  if (!reported) {
    process.stdout.write("Outdated packages\n");
    process.stdout.write("All packages up to date\n");
  }
}

export async function runExplain(name: string | undefined): Promise<void> {
  if (!name) {
    throw PrayError.resolution("explain requires a package name");
  }
  const project = await resolveProject(defaultManifestPath());
  const packageEntry = project.packages.find(
    (entry) => entry.declaration.name === name,
  );
  if (!packageEntry) {
    throw PrayError.resolution(`package ${name} not found`);
  }
  const lockfilePath = defaultLockfilePath(project.projectRoot);
  const lockfile = existsSync(lockfilePath)
    ? readLockfile(lockfilePath)
    : undefined;
  const lockfilePackage = lockfile?.package.find(
    (entry) => entry.name === name,
  );
  const lines = [
    "Package explanation",
    `name: ${packageEntry.declaration.name}`,
    `constraint: ${packageEntry.declaration.constraint}`,
    `resolved version: ${packageEntry.spec.version}`,
  ];
  if (packageEntry.registryLatestVersion) {
    lines.push(`registry latest: ${packageEntry.registryLatestVersion}`);
  }
  lines.push(`source: ${packageSourceSummary(packageEntry)}`);
  lines.push(`exports: ${packageEntry.selectedExports.join(", ")}`);
  lines.push(
    `dependencies: ${packageEntry.spec.dependencies.map((dependency) => dependency.name).join(", ")}`,
  );
  lines.push(`tree hash: ${packageEntry.treeHash}`);
  if (lockfilePackage) {
    lines.push(`lockfile version: ${lockfilePackage.version}`);
    lines.push(`lockfile path: ${lockfilePackage.path}`);
  }
  process.stdout.write(`${lines.join("\n")}\n`);
}
