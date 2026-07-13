import { PrayError } from "../../errors.js";
import { buildLockfile, writeLockfile } from "../../lockfile/index.js";
import {
  defaultLockfilePath,
  defaultManifestPath,
} from "../../lockfile/paths.js";
import {
  addPackageToManifest,
  removePackageFromManifest,
} from "../../manifest/edit.js";
import { renderProject, writeRenderedTargets } from "../../render/project.js";
import { resolveProject } from "../../resolve/project.js";
import { materializeProject } from "../materialize.js";

export async function runAdd(argumentsList: string[]): Promise<void> {
  const name = argumentsList[0];
  if (!name) {
    throw PrayError.unsupported("add requires a package name");
  }
  let constraint: string | undefined;
  let path: string | undefined;
  for (let index = 1; index < argumentsList.length; index += 1) {
    const argument = argumentsList[index];
    if (argument === undefined) {
      continue;
    }
    if (argument === "--path") {
      path = argumentsList[index + 1];
      index += 1;
    } else if (!constraint) {
      constraint = argument;
    }
  }
  addPackageToManifest(defaultManifestPath(), name, { constraint, path });
}

export async function runRemove(name: string | undefined): Promise<void> {
  if (!name) {
    throw PrayError.manifest("remove requires a package name");
  }
  removePackageFromManifest(defaultManifestPath(), name);
  await materializeProject();
}

export async function runUnlock(name: string | undefined): Promise<void> {
  if (!name) {
    throw PrayError.manifest("unlock requires a package name");
  }
  const manifestPath = defaultManifestPath();
  const project = await resolveProject(manifestPath);
  if (!project.manifest.packages.some((entry) => entry.name === name)) {
    throw PrayError.manifest(`package ${name} not found`);
  }
  const lockfilePath = defaultLockfilePath(project.projectRoot);
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
  writeLockfile(lockfilePath, updatedLockfile);
  writeRenderedTargets(project, rendered);
  process.stdout.write(`Unlocked ${name}\n`);
}
