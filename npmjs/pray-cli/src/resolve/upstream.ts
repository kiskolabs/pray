import { PrayError } from "../errors.js";
import type { LockedUpstream, Lockfile } from "../lockfile/types.js";
import type { ManifestPackage, ManifestSource } from "../manifest/types.js";
import type { PackageSpec } from "../package-spec/types.js";
import type { ResolveOptions } from "./context.js";
import { impliedSourceName } from "./package-root.js";
import type { ResolvedPackage } from "./types.js";

type ResolveNamed = (declaration: ManifestPackage) => Promise<ResolvedPackage>;

interface UpstreamRefreshCandidate {
  declaration: Pick<ManifestPackage, "name" | "path">;
  spec: Pick<PackageSpec, "upstream">;
}

export function assertPathUpstreamRefreshSupported(
  packages: readonly UpstreamRefreshCandidate[],
  selectedPackage?: string,
): void {
  const unsupported = packages.find(
    (packageEntry) =>
      packageEntry.declaration.path &&
      packageEntry.spec.upstream &&
      (!selectedPackage || packageEntry.declaration.name === selectedPackage),
  );
  if (unsupported) {
    throw PrayError.unsupported(
      `this installation cannot refresh upstream package ${unsupported.declaration.name}; install pray with Cargo and retry`,
    );
  }
}

export async function lockPathUpstream(
  declaration: ManifestPackage,
  spec: PackageSpec,
  sources: Map<string, ManifestSource>,
  lockfile: Lockfile | undefined,
  options: ResolveOptions,
  resolveNamed: ResolveNamed,
): Promise<LockedUpstream | undefined> {
  const upstream = spec.upstream;
  if (!upstream) {
    return undefined;
  }
  if (spec.name === upstream.name) {
    throw PrayError.resolution(
      `package ${spec.name} cannot use itself as upstream`,
    );
  }
  if (!declaration.path) {
    return undefined;
  }
  const refresh =
    options.ignoreLockedVersions ||
    options.unlockedPackages.has(declaration.name);
  const locked = lockfile?.package.find(
    (entry) => entry.name === declaration.name,
  )?.upstream;
  if (!refresh && locked && locked.name !== upstream.name) {
    throw PrayError.integrity(
      `locked upstream name mismatch for ${declaration.name}: expected ${locked.name}, found ${upstream.name}`,
    );
  }
  const name = !refresh && locked ? locked.name : upstream.name;
  const constraint =
    refresh || !locked ? upstream.constraint : `= ${locked.version}`;
  const source =
    !refresh && locked
      ? locked.source
      : impliedSourceName(
          {
            name,
            constraint,
            exports: [],
            targets: [],
            features: [],
            groups: [],
            optional: false,
          },
          sources,
        );
  const resolved = await resolveNamed({
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
  });
  const resolvedUpstream: LockedUpstream = {
    name,
    version: resolved.spec.version,
    ...(source ? { source } : {}),
    tree_hash: resolved.treeHash,
    artifact_hash: resolved.artifactHash,
  };
  if (!refresh && locked) {
    ensureLockedUpstreamMatches(locked, resolvedUpstream);
  }
  return resolvedUpstream;
}

export function ensureLockedUpstreamMatches(
  locked: LockedUpstream,
  resolved: LockedUpstream,
): void {
  if (locked.name !== resolved.name || locked.version !== resolved.version) {
    throw PrayError.integrity("locked upstream identity mismatch");
  }
  if (locked.source !== resolved.source) {
    throw PrayError.integrity("locked upstream source mismatch");
  }
  if (locked.tree_hash !== resolved.tree_hash) {
    throw PrayError.integrity("locked upstream tree hash mismatch");
  }
  if (locked.artifact_hash !== resolved.artifact_hash) {
    throw PrayError.integrity("locked upstream artifact hash mismatch");
  }
}
