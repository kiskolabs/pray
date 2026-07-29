import { existsSync } from "node:fs";
import { PrayError } from "../../errors.js";
import { buildLockfile, readLockfile } from "../../lockfile/index.js";
import { renderProject } from "../../render/project.js";
import { defaultResolveOptions } from "../../resolve/context.js";
import {
  lockfilePath,
  resolveCurrentProject,
  resolveCurrentProjectWithGitRefreshFallback,
} from "../invocation.js";

export async function runPlanCommand(argumentsList: string[]): Promise<void> {
  const remote = parsePlanArguments(argumentsList);
  const project = remote
    ? await resolveCurrentProject({
        ...defaultResolveOptions(),
        refreshSourceRevisions: true,
        ignoreLockedVersions: true,
      })
    : await resolveCurrentProjectWithGitRefreshFallback(
        defaultResolveOptions(),
        true,
      );
  const rendered = renderProject(project);
  const nextLockfile = buildLockfile({
    manifestHash: project.manifestHash,
    projectRoot: project.projectRoot,
    manifestSources: project.manifest.sources,
    manifestTargets: project.manifest.targets,
    rendered,
    packages: project.packages,
    sourceRevisions: project.sourceRevisions,
    sourceHostKeys: project.sourceHostKeys,
  });
  const previous = existsSync(lockfilePath())
    ? readLockfile(lockfilePath())
    : undefined;
  process.stdout.write("Plan\n");
  for (const target of rendered) {
    process.stdout.write(`would render ${target.path}\n`);
  }
  if (!previous) {
    return;
  }
  for (const packageEntry of nextLockfile.package) {
    const previousPackage = previous.package.find(
      (entry) => entry.name === packageEntry.name,
    );
    if (previousPackage && previousPackage.version !== packageEntry.version) {
      process.stdout.write(
        `would update ${packageEntry.name} ${previousPackage.version} -> ${packageEntry.version}\n`,
      );
    }
  }
}

function parsePlanArguments(argumentsList: string[]): boolean {
  let remote = false;
  for (const argument of argumentsList) {
    if (argument === "--remote") {
      remote = true;
      continue;
    }
    throw PrayError.unsupported(`unknown plan flag: ${argument}`);
  }
  return remote;
}
