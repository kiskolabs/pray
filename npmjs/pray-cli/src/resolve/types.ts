import type { LocalPosition } from "../domain/types.js";
import type { Manifest, ManifestPackage } from "../manifest/types.js";
import type { PackageSpec } from "../package-spec/types.js";

export interface ResolvedLocalFile {
  path: string;
  manifestPath: string;
  content: string;
  position: LocalPosition;
  optional: boolean;
}

export interface ResolvedPackage {
  declaration: ManifestPackage;
  root: string;
  spec: PackageSpec;
  treeHash: string;
  artifactHash: string;
  artifact: string;
  selectedExports: string[];
  sourceChecksum: string;
  exportBodies: Map<string, string>;
  skillFiles: Map<string, string[]>;
  signerFingerprint?: string;
  registryLatestVersion?: string;
}

export interface ResolvedProject {
  manifestPath: string;
  projectRoot: string;
  manifest: Manifest;
  manifestHash: string;
  packages: ResolvedPackage[];
  localFiles: ResolvedLocalFile[];
  sourceRevisions: Map<string, string>;
  sourceHostKeys: Map<string, string>;
}
