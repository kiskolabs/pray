import { existsSync, realpathSync } from "node:fs";
import { resolve } from "node:path";
import { PrayError } from "../errors.js";
import {
  buildLockfile,
  lockfilesEquivalent,
  readLockfile,
  writeLockfileIfChanged,
} from "../lockfile/index.js";
import {
  defaultLockfilePath,
  defaultManifestPath,
  projectRootFromManifest,
} from "../lockfile/paths.js";
import { manifestToJson } from "../manifest/types.js";
import { ensureRenderedTargetsCurrent } from "../render/check.js";
import { renderProject, writeRenderedTargets } from "../render/project.js";
import { defaultResolveOptions } from "../resolve/context.js";
import { resolveProject } from "../resolve/project.js";
import { applyPathUpstreamRefreshes } from "../resolve/upstream-refresh.js";
import { runTransaction } from "../transaction/index.js";
import { verifyProject } from "../verify/project.js";
import { localSummaryLines } from "./apply-report.js";
import {
  projectRoot,
  resolveCurrentProjectWithGitRefreshFallback,
} from "./invocation.js";

/** Options for {@link materializeProject}: resolve, render, and write lockfile. */
export interface MaterializeOptions {
  /** Absolute or relative path to the Prayfile. */
  manifestPath?: string;
  /** Require an existing lockfile and fail when resolution would change it. */
  frozen?: boolean;
  /** Prefer locked versions without refreshing remote sources. */
  locked?: boolean;
  /** Resolve without network access. */
  offline?: boolean;
  /** Refresh pinned git source revisions during resolve. */
  refreshSourceRevisions?: boolean;
}

/** Resolve the project, render destinations, and update `Prayfile.lock` when needed. */
export async function materializeProject(
  options: MaterializeOptions = {},
): Promise<void> {
  const transactionRoot = options.manifestPath
    ? realpathSync(projectRootFromManifest(resolve(options.manifestPath)))
    : projectRoot();
  if (
    (options.locked || options.frozen) &&
    !existsSync(defaultLockfilePath(transactionRoot))
  ) {
    throw PrayError.resolution(
      `Prayfile.lock is required under ${options.frozen ? "--frozen" : "--locked"}. Run pray install to create it.`,
    );
  }
  if (options.frozen) return materializeWithOptions(options);
  return runTransaction(transactionRoot, () => materializeWithOptions(options));
}

async function materializeWithOptions(
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

  if (options.locked || options.frozen) {
    const flag = options.frozen ? "--frozen" : "--locked";
    if (
      !previousLockfile ||
      !lockfilesEquivalent(previousLockfile, nextLockfile)
    ) {
      throw PrayError.resolution(
        `Prayfile.lock would change under ${flag}. Run pray install to update it.`,
      );
    }
    if (options.frozen) {
      verifyProject(project, previousLockfile, true);
      ensureRenderedTargetsCurrent(project, rendered);
      return;
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
