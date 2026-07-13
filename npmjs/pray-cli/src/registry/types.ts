export interface RegistryPackageVersion {
  version: string;
  artifact: string;
  artifactHash?: string;
  treeHash?: string;
  yanked: boolean;
  targets: string[];
  exports: string[];
  signer?: string;
  signerFingerprint?: string;
  publishedAt?: string;
  signature?: string;
}

export interface RegistryPackageMetadata {
  name: string;
  versions: RegistryPackageVersion[];
}

export interface RegistryPackageResolution {
  root: string;
  signerFingerprint?: string;
  registryLatestVersion?: string;
}

export interface RegistryIndex {
  spec: string;
  packages: string[];
}
