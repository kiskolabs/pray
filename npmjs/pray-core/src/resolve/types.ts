import type { LocalPosition } from "../domain/types.js";
import type { Manifest, ManifestPackage } from "../manifest/types.js";
import type { PackageSpec } from "../package-spec/types.js";

/** Resolved local file embed ready for render. */
export interface ResolvedLocalFile {
  /** Relative path of the local file. */
  path: string;
  /** Absolute path used to load the file. */
  manifestPath: string;
  /** File contents. */
  content: string;
  /** Checksum of the source content. */
  sourceChecksum: string;
  /** Placement position in compose destinations. */
  position: LocalPosition;
  /** Whether the file may be missing. */
  optional: boolean;
}

/** Resolved package with loaded exports and integrity hashes. */
export interface ResolvedPackage {
  /** Manifest package declaration. */
  declaration: ManifestPackage;
  /** Absolute package root on disk. */
  root: string;
  /** Parsed prayspec. */
  spec: PackageSpec;
  /** Content-addressed tree hash. */
  treeHash: string;
  /** Artifact hash. */
  artifactHash: string;
  /** Artifact locator. */
  artifact: string;
  /** Selected export names. */
  selectedExports: string[];
  /** Aggregate source checksum for selected exports. */
  sourceChecksum: string;
  /** Export name to body map. */
  exportBodies: Map<string, string>;
  /** Skill export name to file path list. */
  skillFiles: Map<string, string[]>;
  /** Signer fingerprint when known. */
  signerFingerprint?: string;
  /** Latest registry version label when queried. */
  registryLatestVersion?: string;
  /** Upstream package identity when embedded. */
  upstream?: import("../lockfile/types.js").LockedUpstream;
}

/** Fully resolved project graph for render, lock, and verify. */
export interface ResolvedProject {
  /** Absolute Prayfile path. */
  manifestPath: string;
  /** Absolute project root. */
  projectRoot: string;
  /** Parsed manifest. */
  manifest: Manifest;
  /** Manifest content hash. */
  manifestHash: string;
  /** Resolved packages in dependency order. */
  packages: ResolvedPackage[];
  /** Resolved local embeds. */
  localFiles: ResolvedLocalFile[];
  /** Source name to pinned revision. */
  sourceRevisions: Map<string, string>;
  /** Source name to host key fingerprint. */
  sourceHostKeys: Map<string, string>;
  /** Active environment label when set. */
  environment?: string;
}
