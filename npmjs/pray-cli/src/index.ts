export { PrayError } from "./errors.js";
export { parseManifest, manifestHash, readManifestText } from "./manifest/index.js";
export { addPackageToManifest, removePackageFromManifest } from "./manifest/edit.js";
export { parsePackageSpec } from "./package-spec/index.js";
export { resolveProject, defaultResolveOptions } from "./resolve/project.js";
export { renderProject, writeRenderedTargets } from "./render/project.js";
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
export {
  inspectProject,
  verifyProject,
  driftProject,
} from "./verify/project.js";
export { materializeProject } from "./cli/materialize.js";
export { publishToRoot, publishToServer } from "./publish/index.js";
export { runServer } from "./serve/index.js";
export { runTrustCommand } from "./trust/index.js";
export { submitConfession } from "./confess/index.js";
export { syncDistributionRoot } from "./sync/index.js";
export { vendorProject } from "./vendor/index.js";
export { runCli } from "./cli/main.js";
export { PACKAGE_VERSION } from "./lockfile/types.js";

export type {
  Manifest,
  ManifestLocal,
  ManifestPackage,
  ManifestSource,
  ManifestTarget,
  RenderPolicy,
} from "./manifest/types.js";
export type {
  Lockfile,
  LockedPackage,
  LockedTarget,
  LockSource,
  ManagedSpanRecord,
} from "./lockfile/types.js";
export type {
  ResolvedLocalFile,
  ResolvedPackage,
  ResolvedProject,
} from "./resolve/types.js";
export type { RenderedTarget } from "./render/types.js";
export type {
  VerificationFinding,
  VerificationReport,
} from "./verify/project.js";
export type { MaterializeOptions } from "./cli/materialize.js";
