/**
 * Cloudflare Workers and browser parse surface. Re-exports `pray-core/parse`.
 * Full CLI dispatch (`runCli`) stays on `./cli` (Node, Deno, Bun).
 *
 * @module
 */

export type {
  LockedPackage,
  LockedTarget,
  Lockfile,
  LockSource,
  ManagedSpanRecord,
  Manifest,
  ManifestLocal,
  ManifestPackage,
  ManifestPublishRemote,
  ManifestSource,
  ManifestTarget,
  PackageSpec,
  PrayErrorKind,
  RenderPolicy,
} from "pray-core/parse";
export {
  canonicalLockfile,
  canonicalManifest,
  canonicalPackageSpec,
  defaultRenderPolicy,
  latestConstraintForPackage,
  lockfileHash,
  lockfilesEquivalent,
  manifestHash,
  normalizeVersionConstraint,
  PrayError,
  parseLockfile,
  parseManifest,
  parsePackageSpec,
  pessimisticConstraintForVersion,
  serializeLockfile,
  versionIsGreaterThan,
  versionSatisfies,
} from "pray-core/parse";
