import { PrayError } from "../errors.js";
import { prepareGitSources } from "../git/sources.js";
import type { LockedUpstream, Lockfile } from "../lockfile/types.js";
import type { ManifestPackage, ManifestSource } from "../manifest/types.js";
import type { ResolveOptions } from "./context.js";
import { resolvePackage } from "./project.js";
import { sourceMap } from "./source-map.js";
import type { ResolvedPackage, ResolvedProject } from "./types.js";
import { ensureLockedUpstreamMatches } from "./upstream.js";
import { contentFileBytes, localContentForRefresh } from "./upstream-io.js";
import {
  tryMergeContentFiles,
  upstreamMergeConflictMessage,
} from "./upstream-merge.js";

export type PathUpstreamConflict = {
  packageName: string;
  upstreamName: string;
  oldVersion: string;
  newVersion: string;
  paths: string[];
};

export async function planPathUpstreamConflicts(
  project: ResolvedProject,
  previous: Lockfile | undefined,
  selected: string | undefined,
  options: ResolveOptions,
): Promise<PathUpstreamConflict[]> {
  const gitSources = prepareGitSources(
    project.projectRoot,
    project.manifest.sources,
    previous,
    options.refreshSourceRevisions,
    options.offline,
  );
  const sources = sourceMap(project.manifest.sources);
  const conflicts: PathUpstreamConflict[] = [];
  for (const packageEntry of project.packages) {
    if (selected && packageEntry.declaration.name !== selected) {
      continue;
    }
    const conflict = await planOnePathUpstreamConflict(
      project,
      packageEntry,
      previous,
      sources,
      gitSources,
      options,
    );
    if (conflict) {
      conflicts.push(conflict);
    }
  }
  return conflicts;
}

export async function reportPathUpstreamWouldConflicts(
  project: ResolvedProject,
  previous: Lockfile | undefined,
  selected: string | undefined,
  options: ResolveOptions,
): Promise<void> {
  const conflicts = await planPathUpstreamConflicts(
    project,
    previous,
    selected,
    options,
  );
  if (conflicts.length === 0) {
    return;
  }
  const messages: string[] = [];
  for (const conflict of conflicts) {
    process.stdout.write(
      `would-conflict ${conflict.packageName}: ${conflict.paths.join(", ")}\n`,
    );
    messages.push(
      upstreamMergeConflictMessage(
        conflict.packageName,
        conflict.upstreamName,
        conflict.oldVersion,
        conflict.newVersion,
        conflict.paths,
      ),
    );
  }
  throw PrayError.resolution(messages.join("\n"));
}

async function planOnePathUpstreamConflict(
  project: ResolvedProject,
  packageEntry: ResolvedPackage,
  previous: Lockfile | undefined,
  sources: Map<string, ManifestSource>,
  gitSources: ReturnType<typeof prepareGitSources>,
  options: ResolveOptions,
): Promise<PathUpstreamConflict | undefined> {
  const newUpstream = packageEntry.upstream;
  if (!newUpstream || !packageEntry.declaration.path) {
    return undefined;
  }
  const localContent = localContentForRefresh(
    packageEntry.root,
    packageEntry.spec,
  );
  if (localContent.size === 0) {
    return undefined;
  }
  const oldUpstream = previous?.package.find(
    (entry) => entry.name === packageEntry.declaration.name,
  )?.upstream;
  if (!oldUpstream) {
    return undefined;
  }
  if (
    oldUpstream.version === newUpstream.version &&
    oldUpstream.tree_hash === newUpstream.tree_hash
  ) {
    return undefined;
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
  if (!Array.isArray(merged)) {
    return undefined;
  }
  return {
    packageName: packageEntry.declaration.name,
    upstreamName: oldUpstream.name,
    oldVersion: oldUpstream.version,
    newVersion: newUpstream.version,
    paths: merged,
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
