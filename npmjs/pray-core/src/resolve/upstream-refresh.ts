import { PrayError } from "../errors.js";
import { prepareGitSources } from "../git/sources.js";
import type { LockedUpstream, Lockfile } from "../lockfile/types.js";
import type { ManifestPackage, ManifestSource } from "../manifest/types.js";
import { findPrayspecFile } from "../package-spec/discovery.js";
import { renderPackageSpec } from "../package-spec/render.js";
import type { PackageSpec } from "../package-spec/types.js";
import { writeProjectFile } from "../transaction/index.js";
import type { ResolveOptions } from "./context.js";
import { resolvePackage } from "./project.js";
import { sourceMap } from "./source-map.js";
import type { ResolvedPackage, ResolvedProject } from "./types.js";
import { ensureLockedUpstreamMatches } from "./upstream.js";
import {
  contentFileBytes,
  localContentForRefresh,
  writeContentFiles,
} from "./upstream-io.js";
import {
  isCleanReplica,
  nextUpstreamConstraint,
  tryMergeContentFiles,
  upstreamMergeConflictMessage,
} from "./upstream-merge.js";

export async function applyPathUpstreamRefreshes(
  project: ResolvedProject,
  previous: Lockfile | undefined,
  selected: string | undefined,
  options: ResolveOptions,
): Promise<boolean> {
  const gitSources = prepareGitSources(
    project.projectRoot,
    project.manifest.sources,
    previous,
    options.refreshSourceRevisions,
    options.offline,
  );
  const sources = sourceMap(project.manifest.sources);
  let changed = false;
  for (const packageEntry of project.packages) {
    if (selected && packageEntry.declaration.name !== selected) {
      continue;
    }
    if (
      await applyOnePathUpstream(
        project,
        packageEntry,
        previous,
        sources,
        gitSources,
        options,
      )
    ) {
      changed = true;
    }
  }
  return changed;
}

async function applyOnePathUpstream(
  project: ResolvedProject,
  packageEntry: ResolvedPackage,
  previous: Lockfile | undefined,
  sources: Map<string, ManifestSource>,
  gitSources: ReturnType<typeof prepareGitSources>,
  options: ResolveOptions,
): Promise<boolean> {
  const newUpstream = packageEntry.upstream;
  if (!newUpstream || !packageEntry.declaration.path) {
    return false;
  }
  const localContent = localContentForRefresh(
    packageEntry.root,
    packageEntry.spec,
  );
  if (localContent.size === 0) {
    return materializeEmptyPathFork(
      project,
      packageEntry,
      previous,
      sources,
      gitSources,
      options,
      newUpstream,
    );
  }
  const oldUpstream = previous?.package.find(
    (entry) => entry.name === packageEntry.declaration.name,
  )?.upstream;
  if (!oldUpstream) {
    return false;
  }
  if (
    oldUpstream.version === newUpstream.version &&
    oldUpstream.tree_hash === newUpstream.tree_hash
  ) {
    return false;
  }
  const oldPackage = await resolveNamed(
    project.projectRoot,
    sources,
    gitSources,
    previous,
    options,
    oldUpstream.name,
    `= ${oldUpstream.version}`,
    oldUpstream.source,
  );
  const resolvedOld: LockedUpstream = {
    name: oldUpstream.name,
    version: oldPackage.spec.version,
    ...(oldUpstream.source ? { source: oldUpstream.source } : {}),
    tree_hash: oldPackage.treeHash,
    artifact_hash: oldPackage.artifactHash,
  };
  ensureLockedUpstreamMatches(oldUpstream, resolvedOld);
  const newPackage = await resolveNamed(
    project.projectRoot,
    sources,
    gitSources,
    previous,
    options,
    newUpstream.name,
    `= ${newUpstream.version}`,
    newUpstream.source,
  );
  const oldContent = contentFileBytes(oldPackage.root, oldPackage.spec);
  const newContent = contentFileBytes(newPackage.root, newPackage.spec);
  const merged = tryMergeContentFiles(oldContent, newContent, localContent);
  if (Array.isArray(merged)) {
    throw PrayError.resolution(
      upstreamMergeConflictMessage(
        packageEntry.declaration.name,
        oldUpstream.name,
        oldUpstream.version,
        newUpstream.version,
        merged,
      ),
    );
  }
  writeContentFiles(packageEntry.root, oldContent, merged);
  writeRefreshedSpec(
    packageEntry,
    newPackage,
    isCleanReplica(oldContent, localContent),
    [...merged.keys()],
  );
  return true;
}

async function materializeEmptyPathFork(
  project: ResolvedProject,
  packageEntry: ResolvedPackage,
  previous: Lockfile | undefined,
  sources: Map<string, ManifestSource>,
  gitSources: ReturnType<typeof prepareGitSources>,
  options: ResolveOptions,
  newUpstream: LockedUpstream,
): Promise<boolean> {
  const newPackage = await resolveNamed(
    project.projectRoot,
    sources,
    gitSources,
    previous,
    options,
    newUpstream.name,
    `= ${newUpstream.version}`,
    newUpstream.source,
  );
  const newContent = contentFileBytes(newPackage.root, newPackage.spec);
  writeContentFiles(packageEntry.root, new Map(), newContent);
  writeRefreshedSpec(packageEntry, newPackage, true, [...newContent.keys()]);
  return true;
}

function writeRefreshedSpec(
  packageEntry: ResolvedPackage,
  newPackage: ResolvedPackage,
  cleanReplica: boolean,
  mergedContentPaths: readonly string[],
): void {
  const specPath = findPrayspecFile(packageEntry.root);
  const updated = forkSpecAfterRefresh(
    packageEntry.spec,
    newPackage.spec,
    cleanReplica,
    mergedContentPaths,
  );
  writeProjectFile(specPath, renderPackageSpec(updated));
}

function forkSpecAfterRefresh(
  local: PackageSpec,
  newUpstream: PackageSpec,
  cleanReplica: boolean,
  mergedContentPaths: readonly string[],
): PackageSpec {
  return {
    ...local,
    files: [...mergedContentPaths],
    exports: cleanReplica ? new Map(newUpstream.exports) : local.exports,
    templates: cleanReplica ? new Map(newUpstream.templates) : local.templates,
    upstream: local.upstream
      ? {
          name: newUpstream.name,
          constraint: nextUpstreamConstraint(
            local.upstream.constraint,
            newUpstream.version,
          ),
        }
      : local.upstream,
  };
}

function resolveNamed(
  projectRoot: string,
  sources: Map<string, ManifestSource>,
  gitSources: ReturnType<typeof prepareGitSources>,
  lockfile: Lockfile | undefined,
  options: ResolveOptions,
  name: string,
  constraint: string,
  source: string | undefined,
): Promise<ResolvedPackage> {
  const declaration: ManifestPackage = {
    name,
    constraint,
    ...(source ? { source } : {}),
    exports: [],
    targets: [],
    features: [],
    groups: [],
    optional: false,
    roles: [],
    bound: false,
  };
  return resolvePackage(
    projectRoot,
    sources,
    gitSources,
    declaration,
    lockfile,
    options,
  );
}
