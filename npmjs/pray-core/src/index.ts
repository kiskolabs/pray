/**
 * TypeScript library for Prayfile: a package manager for the language placed before inference.
 *
 * Resolve, lock, render, verify, publish, and serve shared instructions, policies,
 * templates, and related input as versioned packages. Consumes path, git, and
 * registry distribution points. CLI dispatch lives in the `pray-cli` package
 * (`pray-cli/cli`). The `pray` binary uses that path.
 *
 * @example
 * ```ts
 * import { parseManifest, resolveProject } from "@kiskolabs/pray-core";
 *
 * const manifest = parseManifest(`prayfile 1\n`);
 * const project = await resolveProject("./Prayfile");
 * ```
 *
 * @module
 */

export { submitConfession } from "./confess/index.js";
export { PrayError } from "./errors.js";
export {
  buildLockfile,
  lockfileHash,
  lockfilesEquivalent,
  parseLockfile,
  readLockfile,
  serializeLockfile,
  writeLockfile,
  writeLockfileIfChanged,
} from "./lockfile/index.js";
export {
  defaultLockfilePath,
  defaultManifestPath,
  projectRootFromManifest,
} from "./lockfile/paths.js";
export type {
  LockedPackage,
  LockedTarget,
  Lockfile,
  LockSource,
  ManagedSpanRecord,
} from "./lockfile/types.js";
export { PACKAGE_VERSION } from "./lockfile/types.js";
export {
  addPackageToManifest,
  removePackageFromManifest,
} from "./manifest/edit.js";
export {
  manifestHash,
  parseManifest,
  readManifestText,
} from "./manifest/index.js";
export type {
  Manifest,
  ManifestLocal,
  ManifestPackage,
  ManifestSource,
  ManifestTarget,
  RenderPolicy,
} from "./manifest/types.js";
export type { MaterializeOptions } from "./materialize.js";
export { materializeProject, printManifest } from "./materialize.js";
export { parsePackageSpec } from "./package-spec/index.js";
export { publishToRoot, publishToServer } from "./publish/index.js";
export type { RegistrySearchHit } from "./registry/search.js";
export {
  searchHttpRegistry,
  searchLocalRegistry,
  searchRegistryIndexNames,
} from "./registry/search.js";
export { renderProject, writeRenderedTargets } from "./render/project.js";
export type { RenderedTarget } from "./render/types.js";
export { defaultResolveOptions, resolveProject } from "./resolve/project.js";
export type {
  ResolvedLocalFile,
  ResolvedPackage,
  ResolvedProject,
} from "./resolve/types.js";
export { runServer } from "./serve/index.js";
export { syncDistributionRoot } from "./sync/index.js";
export { vendorProject } from "./vendor/index.js";
export { inspectLockedDestinations } from "./verify/locked-dest.js";
export type {
  VerificationFinding,
  VerificationReport,
} from "./verify/project.js";
export {
  driftProject,
  inspectProject,
  verifyProject,
} from "./verify/project.js";
