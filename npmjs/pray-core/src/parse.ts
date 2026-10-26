/**
 * Workers-safe parse and validate surface: text in, values out.
 * Full resolve/render/publish/serve lives on the package root export.
 *
 * @module
 */

export {
  latestConstraintForPackage,
  normalizeVersionConstraint,
  pessimisticConstraintForVersion,
  versionIsGreaterThan,
  versionSatisfies,
} from "./constraint.js";
export type { PrayErrorKind } from "./errors.js";
export { PrayError } from "./errors.js";
export {
  lockfileHash,
  lockfilesEquivalent,
  parseLockfile,
  serializeLockfile,
} from "./lockfile/text.js";
export type {
  LockedPackage,
  LockedTarget,
  Lockfile,
  LockSource,
  ManagedSpanRecord,
} from "./lockfile/types.js";
export { canonicalLockfile } from "./lockfile/types.js";
export { manifestHash, parseManifest } from "./manifest/parse-api.js";
export {
  canonicalManifest,
  defaultRenderPolicy,
  type Manifest,
  type ManifestLocal,
  type ManifestPackage,
  type ManifestPublishRemote,
  type ManifestSource,
  type ManifestTarget,
  type RenderPolicy,
} from "./manifest/types.js";
export { parsePackageSpecText as parsePackageSpec } from "./package-spec/parser.js";
export {
  canonicalPackageSpec,
  type PackageSpec,
} from "./package-spec/types.js";
