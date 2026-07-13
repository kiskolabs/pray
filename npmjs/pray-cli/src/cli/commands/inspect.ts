import { existsSync } from "node:fs";
import { PrayError } from "../../errors.js";
import { buildLockfile, lockfilesEquivalent, readLockfile } from "../../lockfile/index.js";
import {
  defaultLockfilePath,
  defaultManifestPath,
} from "../../lockfile/paths.js";
import { renderProject } from "../../render/project.js";
import { resolveProject } from "../../resolve/project.js";
import { packageSourceSummary } from "../../tree/index.js";

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

export async function runOutdated(): Promise<void> {
  const lockfilePath = defaultLockfilePath(process.cwd());
  const previous = existsSync(lockfilePath) ? readLockfile(lockfilePath) : undefined;
  const project = await resolveProject(defaultManifestPath());
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
  });
  if (previous && lockfilesEquivalent(previous, latest)) {
    process.stdout.write("All packages up to date\n");
    return;
  }
  process.stdout.write("Outdated packages\n");
  for (const packageEntry of project.packages) {
    if (
      packageEntry.registryLatestVersion &&
      packageEntry.registryLatestVersion !== packageEntry.spec.version
    ) {
      process.stdout.write(
        `${packageEntry.declaration.name}: ${packageEntry.spec.version} -> ${packageEntry.registryLatestVersion}\n`,
      );
    }
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
  const lockfile = existsSync(lockfilePath) ? readLockfile(lockfilePath) : undefined;
  const lockfilePackage = lockfile?.package.find((entry) => entry.name === name);
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
