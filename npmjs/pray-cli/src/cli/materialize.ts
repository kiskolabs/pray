import { existsSync } from "node:fs";
import { resolve } from "node:path";
import { PrayError } from "../errors.js";
import {
  buildLockfile,
  lockfilesEquivalent,
  readLockfile,
  writeLockfileIfChanged,
} from "../lockfile/index.js";
import { defaultLockfilePath, defaultManifestPath } from "../lockfile/paths.js";
import { manifestToJson } from "../manifest/types.js";
import { renderProject, writeRenderedTargets } from "../render/project.js";
import { defaultResolveOptions } from "../resolve/context.js";
import { resolveProject } from "../resolve/project.js";
import { applyPathUpstreamRefreshes } from "../resolve/upstream-refresh.js";
import { runTransaction } from "../transaction/index.js";
import { localSummaryLines } from "./apply-report.js";
import {
  projectRoot,
  resolveCurrentProjectWithGitRefreshFallback,
} from "./invocation.js";

export interface MaterializeOptions {
  manifestPath?: string;
  frozen?: boolean;
  locked?: boolean;
  offline?: boolean;
  refreshSourceRevisions?: boolean;
}

export async function materializeProject(
  options: MaterializeOptions = {},
): Promise<void> {
  return runTransaction(projectRoot(), () => materializeInTransaction(options));
}

async function materializeInTransaction(
  options: MaterializeOptions,
): Promise<void> {
  const manifestPath = resolve(options.manifestPath ?? defaultManifestPath());
  if (!existsSync(manifestPath)) {
    throw PrayError.manifest(
      `missing ${manifestPath}; run pray init to create one`,
    );
  }

  const resolveOptions = {
    ...defaultResolveOptions(),
    offline: options.offline ?? false,
    refreshSourceRevisions: options.refreshSourceRevisions ?? false,
  };
  let project = await resolveCurrentProjectWithGitRefreshFallback(
    resolveOptions,
    !options.locked && !options.frozen,
  );
  const lockfilePath = defaultLockfilePath(project.projectRoot);
  const previousLockfile = existsSync(lockfilePath)
    ? readLockfile(lockfilePath)
    : undefined;
  if (!options.locked && !options.frozen) {
    if (
      await applyPathUpstreamRefreshes(
        project,
        previousLockfile,
        undefined,
        resolveOptions,
      )
    ) {
      project = await resolveCurrentProjectWithGitRefreshFallback(
        resolveOptions,
        true,
      );
    }
  }
  const rendered = renderProject(project);
  const nextLockfile = buildLockfile({
    manifestHash: project.manifestHash,
    ...(project.environment ? { environment: project.environment } : {}),
    projectRoot: project.projectRoot,
    manifestSources: project.manifest.sources,
    manifestTargets: project.manifest.targets,
    rendered,
    packages: project.packages,
    sourceRevisions: project.sourceRevisions,
    sourceHostKeys: project.sourceHostKeys,
    project,
  });

  if (options.frozen || options.locked) {
    const existing = existsSync(lockfilePath)
      ? readLockfile(lockfilePath)
      : undefined;
    if (options.frozen && existing) {
      for (const target of rendered) {
        const outputPath = resolve(project.projectRoot, target.path);
        if (!existsSync(outputPath)) {
          throw PrayError.verify(
            `Rendered file ${target.path} is missing under --frozen`,
          );
        }
      }
    }
    if (options.locked && existing) {
      if (!lockfilesEquivalent(existing, nextLockfile)) {
        throw PrayError.resolution(
          "Prayfile.lock would change under --locked. Run without --locked to update.",
        );
      }
    }
  }

  writeRenderedTargets(project, rendered, previousLockfile);
  writeLockfileIfChanged(lockfilePath, nextLockfile);
  if (!options.frozen && !options.locked) {
    for (const line of localSummaryLines(previousLockfile, project)) {
      process.stdout.write(`${line}\n`);
    }
  }
}

export async function printManifest(
  manifestPath = defaultManifestPath(),
): Promise<void> {
  const project = await resolveProject(manifestPath, defaultResolveOptions());
  process.stdout.write(
    `${JSON.stringify(manifestToJson(project.manifest), null, 2)}\n`,
  );
}
