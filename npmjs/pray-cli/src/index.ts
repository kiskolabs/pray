export { runCli } from "./cli/main.js";
export type { MaterializeOptions } from "./cli/materialize.js";
export { materializeProject } from "./cli/materialize.js";
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
export { parsePackageSpec } from "./package-spec/index.js";
export { publishToRoot, publishToServer } from "./publish/index.js";
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
export { runTrustCommand } from "./trust/index.js";
export { vendorProject } from "./vendor/index.js";
export type {
  VerificationFinding,
  VerificationReport,
} from "./verify/project.js";
export {
  driftProject,
  inspectProject,
  verifyProject,
} from "./verify/project.js";
