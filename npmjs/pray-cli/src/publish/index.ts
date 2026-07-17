import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { buildPackageArchiveBytes } from "../archive/praypkg.js";
import { PrayError } from "../errors.js";
import { sha256Prefixed } from "../hashing.js";
import { httpPut, joinUrl } from "../http/client.js";
import { parseMetadata, registryArtifactSignature } from "../registry/index.js";
import type {
  RegistryIndex,
  RegistryPackageMetadata,
  RegistryPackageVersion,
} from "../registry/types.js";
import type { ResolvedPackage, ResolvedProject } from "../resolve/types.js";

export async function publishToRoot(
  project: ResolvedProject,
  root: string,
  signer = "local",
  signerFingerprint?: string,
): Promise<void> {
  const distributionRoot = root;
  const index = loadRegistryIndex(distributionRoot);
  const packageNames = new Set(index.packages);

  for (const packageEntry of project.packages) {
    const archiveBytes = buildPackageArchiveBytes(packageEntry);
    const artifactPath = registryArtifactPath(
      packageEntry.declaration.name,
      packageEntry.spec.version,
    );
    writeOutputBytes(join(distributionRoot, artifactPath), archiveBytes);

    const metadataPath = registryMetadataPath(
      distributionRoot,
      packageEntry.declaration.name,
    );
    const metadata = loadRegistryPackageMetadata(
      metadataPath,
      packageEntry.declaration.name,
    );
    const versionEntry = publishedRegistryPackageVersion(
      packageEntry,
      signer,
      signerFingerprint,
      archiveBytes,
      artifactPath,
    );
    metadata.versions = metadata.versions.filter(
      (entry) => entry.version !== versionEntry.version,
    );
    metadata.versions.push(versionEntry);
    writeRegistryPackageMetadata(metadataPath, metadata);
    packageNames.add(packageEntry.declaration.name);
  }

  index.packages = [...packageNames].sort();
  writeRegistryIndex(distributionRoot, index);
}

export async function publishToServer(
  project: ResolvedProject,
  serverUrl: string,
  signer = "local",
  signerFingerprint?: string,
): Promise<void> {
  for (const packageEntry of project.packages) {
    const archiveBytes = buildPackageArchiveBytes(packageEntry);
    const artifactPath = registryArtifactPath(
      packageEntry.declaration.name,
      packageEntry.spec.version,
    );
    await httpPut(
      joinUrl(serverUrl, artifactPath),
      "application/octet-stream",
      archiveBytes,
    );
    const metadata: RegistryPackageMetadata = {
      name: packageEntry.declaration.name,
      versions: [
        publishedRegistryPackageVersion(
          packageEntry,
          signer,
          signerFingerprint,
          archiveBytes,
          artifactPath,
        ),
      ],
    };
    await httpPut(
      joinUrl(serverUrl, `v1/packages/${packageEntry.declaration.name}.json`),
      "application/json",
      JSON.stringify(metadataToHash(metadata), null, 2),
    );
  }
}

export function initDistributionRoot(root: string): void {
  const distributionRoot = root.endsWith("prayers")
    ? root
    : join(root, "prayers");
  const indexPath = join(distributionRoot, "v1", "index.json");
  if (existsSync(indexPath)) {
    throw PrayError.manifest(
      `distribution repo already exists at ${distributionRoot}`,
    );
  }
  mkdirSync(join(distributionRoot, "v1", "packages"), { recursive: true });
  mkdirSync(join(distributionRoot, "v1", "artifacts"), { recursive: true });
  writeRegistryIndex(distributionRoot, {
    spec: "prayfile-distribution-1",
    packages: [],
  });
}

function publishedRegistryPackageVersion(
  packageEntry: ResolvedPackage,
  signer: string,
  signerFingerprint: string | undefined,
  archiveBytes: Buffer,
  artifactPath: string,
): RegistryPackageVersion {
  return {
    version: packageEntry.spec.version,
    artifact: artifactPath,
    artifactHash: sha256Prefixed(archiveBytes),
    treeHash: packageEntry.treeHash,
    yanked: false,
    targets: packageEntry.spec.targets,
    exports: [...packageEntry.spec.exports.keys()],
    signer,
    signerFingerprint,
    publishedAt: new Date().toISOString(),
    signature: registryArtifactSignature(
      archiveBytes,
      packageEntry.treeHash,
      signer,
    ),
  };
}

function loadRegistryIndex(root: string): RegistryIndex {
  const path = join(root, "v1", "index.json");
  if (!existsSync(path)) {
    return { spec: "prayfile-distribution-1", packages: [] };
  }
  const data = JSON.parse(readFileSync(path, "utf8")) as RegistryIndex;
  return { spec: data.spec, packages: data.packages ?? [] };
}

function writeRegistryIndex(root: string, index: RegistryIndex): void {
  const path = join(root, "v1", "index.json");
  mkdirSync(join(path, ".."), { recursive: true });
  writeFileSync(
    path,
    `${JSON.stringify({ spec: index.spec, packages: index.packages }, null, 2)}\n`,
    "utf8",
  );
}

function loadRegistryPackageMetadata(
  path: string,
  packageName: string,
): RegistryPackageMetadata {
  if (!existsSync(path)) {
    return { name: packageName, versions: [] };
  }
  return parseMetadata(readFileSync(path, "utf8"));
}

function writeRegistryPackageMetadata(
  path: string,
  metadata: RegistryPackageMetadata,
): void {
  mkdirSync(join(path, ".."), { recursive: true });
  writeFileSync(
    path,
    `${JSON.stringify(metadataToHash(metadata), null, 2)}\n`,
    "utf8",
  );
}

function metadataToHash(
  metadata: RegistryPackageMetadata,
): Record<string, unknown> {
  return {
    name: metadata.name,
    versions: metadata.versions.map((entry) => versionToHash(entry)),
  };
}

function versionToHash(entry: RegistryPackageVersion): Record<string, unknown> {
  const hash: Record<string, unknown> = {
    version: entry.version,
    artifact: entry.artifact,
    yanked: entry.yanked,
    targets: entry.targets,
    exports: entry.exports,
  };
  if (entry.artifactHash) hash.artifact_hash = entry.artifactHash;
  if (entry.treeHash) hash.tree_hash = entry.treeHash;
  if (entry.signer) hash.signer = entry.signer;
  if (entry.signerFingerprint)
    hash.signer_fingerprint = entry.signerFingerprint;
  if (entry.publishedAt) hash.published_at = entry.publishedAt;
  if (entry.signature) hash.signature = entry.signature;
  return hash;
}

function registryMetadataPath(root: string, packageName: string): string {
  return join(root, "v1", "packages", `${packageName}.json`);
}

export function registryArtifactPath(
  packageName: string,
  version: string,
): string {
  const artifactName = `${packageName.replaceAll("/", "-")}-${version}.praypkg`;
  return `v1/artifacts/${packageName}/${version}/${artifactName}`;
}

function writeOutputBytes(path: string, bytes: Buffer): void {
  mkdirSync(join(path, ".."), { recursive: true });
  writeFileSync(path, bytes);
}
