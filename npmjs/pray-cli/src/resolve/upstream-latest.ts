import { readFileSync } from "node:fs";
import { latestConstraintForPackage, versionSatisfies } from "../constraint.js";
import { PrayError } from "../errors.js";
import { prepareGitSources } from "../git/sources.js";
import type { Lockfile } from "../lockfile/types.js";
import type { ManifestPackage } from "../manifest/types.js";
import { findPrayspecFile } from "../package-spec/discovery.js";
import { parsePackageSpec } from "../package-spec/index.js";
import { renderPackageSpec } from "../package-spec/render.js";
import { writeProjectFile } from "../transaction/index.js";
import type { ResolveOptions } from "./context.js";
import { impliedSourceName } from "./package-root.js";
import { resolvePackage } from "./project.js";
import { sourceMap } from "./source-map.js";
import type { ResolvedPackage, ResolvedProject } from "./types.js";

export interface PathUpstreamLatestConstraint {
  packageName: string;
  upstreamName: string;
  currentConstraint: string;
  latestVersion: string;
  newConstraint: string;
  packageRoot: string;
}

export async function planPathUpstreamLatestConstraints(
  project: ResolvedProject,
  previous: Lockfile | undefined,
  selected: string | undefined,
  options: ResolveOptions,
): Promise<PathUpstreamLatestConstraint[]> {
  const gitSources = prepareGitSources(
    project.projectRoot,
    project.manifest.sources,
    previous,
    options.refreshSourceRevisions,
  );
  const sources = sourceMap(project.manifest.sources);
  const plans: PathUpstreamLatestConstraint[] = [];
  for (const packageEntry of project.packages) {
    if (selected && packageEntry.declaration.name !== selected) {
      continue;
    }
    const plan = await planOnePathUpstream(
      project,
      packageEntry,
      sources,
      gitSources,
      previous,
      options,
    );
    if (plan) {
      plans.push(plan);
    }
  }
  return plans;
}

export function applyPathUpstreamLatestConstraints(
  plans: readonly PathUpstreamLatestConstraint[],
): void {
  for (const plan of plans) {
    const specPath = findPrayspecFile(plan.packageRoot);
    const spec = parsePackageSpec(readFileSync(specPath, "utf8"));
    if (!spec.upstream) {
      throw PrayError.resolution(
        `package ${plan.packageName} has no upstream pin`,
      );
    }
    spec.upstream = {
      ...spec.upstream,
      constraint: plan.newConstraint,
    };
    writeProjectFile(specPath, renderPackageSpec(spec));
  }
}

async function planOnePathUpstream(
  project: ResolvedProject,
  packageEntry: ResolvedPackage,
  sources: Map<string, import("../manifest/types.js").ManifestSource>,
  gitSources: ReturnType<typeof prepareGitSources>,
  lockfile: Lockfile | undefined,
  options: ResolveOptions,
): Promise<PathUpstreamLatestConstraint | undefined> {
  const upstream = packageEntry.spec.upstream;
  if (!upstream || !packageEntry.declaration.path) {
    return undefined;
  }
  const source = impliedSourceName(
    namedPackage(upstream.name, "*", undefined),
    sources,
  );
  const resolved = await resolvePackage(
    project.projectRoot,
    sources,
    gitSources,
    namedPackage(upstream.name, "*", source),
    lockfile,
    options,
  );
  if (versionSatisfies(resolved.spec.version, upstream.constraint)) {
    return undefined;
  }
  const newConstraint = latestSpecUpstreamConstraint(
    upstream.constraint,
    resolved.spec.version,
  );
  if (!versionSatisfies(resolved.spec.version, newConstraint)) {
    throw PrayError.resolution(
      `derived constraint ${newConstraint} does not admit latest ${resolved.spec.version} for ${packageEntry.declaration.name}`,
    );
  }
  return {
    packageName: packageEntry.declaration.name,
    upstreamName: upstream.name,
    currentConstraint: upstream.constraint,
    latestVersion: resolved.spec.version,
    newConstraint,
    packageRoot: packageEntry.root,
  };
}

export function latestSpecUpstreamConstraint(
  current: string,
  latestVersion: string,
): string {
  const derived = latestConstraintForPackage(current, latestVersion);
  if (derived.startsWith("=")) {
    return `= ${latestVersion}`;
  }
  return derived;
}

function namedPackage(
  name: string,
  constraint: string,
  source: string | undefined,
): ManifestPackage {
  return {
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
}
