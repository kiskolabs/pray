export interface LockSource {
  name: string;
  kind: string;
  url: string;
  revision?: string;
  host_key_fingerprint?: string;
}

export interface LockedPackage {
  name: string;
  version: string;
  source?: string;
  path: string;
  tree_hash: string;
  artifact_hash: string;
  artifact: string;
  exports: string[];
  dependencies: string[];
  signer_fingerprint?: string;
}

export interface LockedTarget {
  name: string;
  outputs: string[];
}

export interface ManagedSpanRecord {
  id: string;
  target: string;
  open_line: number;
  close_line: number;
  ideal_checksum: string;
  package: string;
  export: string;
  source_checksum: string;
  silenced: boolean;
}

export interface Lockfile {
  prayfile_lock: string;
  spec: string;
  generated_by: string;
  manifest_hash: string;
  environment?: string;
  source: LockSource[];
  package: LockedPackage[];
  target: LockedTarget[];
  managed_span: ManagedSpanRecord[];
}

export function canonicalLockfile(lockfile: Lockfile): Lockfile {
  return {
    ...lockfile,
    source: [...lockfile.source].sort((left, right) =>
      left.name.localeCompare(right.name),
    ),
    package: [...lockfile.package].sort((left, right) =>
      left.name.localeCompare(right.name) ||
      (left.source ?? "").localeCompare(right.source ?? "") ||
      left.version.localeCompare(right.version),
    ),
    target: [...lockfile.target].sort((left, right) =>
      left.name.localeCompare(right.name),
    ),
    managed_span: [...lockfile.managed_span].sort((left, right) =>
      left.target.localeCompare(right.target) ||
      left.open_line - right.open_line ||
      left.id.localeCompare(right.id),
    ),
  };
}

export const PACKAGE_VERSION = "1.0.0";
export const GENERATED_BY = `pray ${PACKAGE_VERSION} (typescript)`;
