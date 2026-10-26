/** Locked named source (git or registry) recorded in `Prayfile.lock`. */
export interface LockSource {
  /** Source name as declared in the Prayfile. */
  name: string;
  /** Source kind (`git`, `registry`, and related). */
  kind: string;
  /** Source URL. */
  url: string;
  /** Pinned revision when applicable. */
  revision?: string;
  /** Host key fingerprint for git sources when recorded. */
  host_key_fingerprint?: string;
}

/** Locked package entry in `Prayfile.lock`. */
export interface LockedPackage {
  /** Package name. */
  name: string;
  /** Selected version. */
  version: string;
  /** Named source that resolved this package, when any. */
  source?: string;
  /** Path to the resolved package root relative to the project. */
  path: string;
  /** Content-addressed tree hash of the package. */
  tree_hash: string;
  /** Hash of the package artifact. */
  artifact_hash: string;
  /** Artifact locator (for example `path:` or registry path). */
  artifact: string;
  /** Selected export names. */
  exports: string[];
  /** Direct dependency package names. */
  dependencies: string[];
  /** Signer fingerprint when the artifact was signed. */
  signer_fingerprint?: string;
  /** Upstream package identity when this entry embeds another package. */
  upstream?: LockedUpstream;
}

export interface LockedUpstream {
  name: string;
  version: string;
  source?: string;
  tree_hash: string;
  artifact_hash: string;
}

/** Locked destination target and its output paths. */
export interface LockedTarget {
  /** Target name from the Prayfile. */
  name: string;
  /** Output file paths rendered for this target. */
  outputs: string[];
}

export const LOCAL_EMBED_PACKAGE = "local";

/** Managed marker span recorded for a rendered destination. */
export interface ManagedSpanRecord {
  /** Stable span identifier used in pray markers. */
  id: string;
  /** Destination path relative to the project root. */
  target: string;
  /** 1-based open marker line. */
  open_line: number;
  /** 1-based close marker line. */
  close_line: number;
  /** Checksum of ideal managed content between markers. */
  ideal_checksum: string;
  /** Package that owns this span. */
  package: string;
  /** Export name that produced this span. */
  export: string;
  /** Checksum of the source export body. */
  source_checksum: string;
  /** True when the user silenced drift for this span. */
  silenced: boolean;
}

export interface ProvisionedFileRecord {
  path: string;
  content_hash: string;
  package: string;
  export: string;
}

/** Parsed `Prayfile.lock` document. */
export interface Lockfile {
  /** Lockfile format major version. */
  prayfile_lock: string;
  /** Lockfile spec revision. */
  spec: string;
  /** Tool identity that wrote the lockfile. */
  generated_by: string;
  /** Hash of the manifest used for this lock. */
  manifest_hash: string;
  /** Active environment label when resolution was environment-scoped. */
  environment?: string;
  /** Locked sources. */
  source: LockSource[];
  /** Locked packages. */
  package: LockedPackage[];
  /** Locked destination targets. */
  target: LockedTarget[];
  /** Managed spans in rendered destinations. */
  managed_span: ManagedSpanRecord[];
  /** Provisioned files written beside compose destinations. */
  provisioned: ProvisionedFileRecord[];
}

export function canonicalLockfile(lockfile: Lockfile): Lockfile {
  return {
    ...lockfile,
    source: [...lockfile.source].sort((left, right) =>
      left.name.localeCompare(right.name),
    ),
    package: [...lockfile.package].sort(
      (left, right) =>
        left.name.localeCompare(right.name) ||
        (left.source ?? "").localeCompare(right.source ?? "") ||
        left.version.localeCompare(right.version),
    ),
    target: [...lockfile.target].sort((left, right) =>
      left.name.localeCompare(right.name),
    ),
    managed_span: [...lockfile.managed_span].sort(
      (left, right) =>
        left.target.localeCompare(right.target) ||
        left.open_line - right.open_line ||
        left.id.localeCompare(right.id),
    ),
    provisioned: [...(lockfile.provisioned ?? [])].sort(
      (left, right) =>
        left.path.localeCompare(right.path) ||
        left.package.localeCompare(right.package),
    ),
  };
}

/** Published package version string for this TypeScript surface. */
export const PACKAGE_VERSION = "1.23.0";
export const GENERATED_BY = `pray ${PACKAGE_VERSION} (typescript)`;
