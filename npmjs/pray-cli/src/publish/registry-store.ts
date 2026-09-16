import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { writeDistributionSettings } from "../distribution.js";
import { PrayError } from "../errors.js";
import { parseMetadata } from "../registry/index.js";
import type {
  RegistryIndex,
  RegistryPackageMetadata,
  RegistryPackageVersion,
} from "../registry/types.js";

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
  writeDistributionSettings(distributionRoot);
}

export function loadRegistryIndex(root: string): RegistryIndex {
  const path = join(root, "v1", "index.json");
  if (!existsSync(path)) {
    return { spec: "prayfile-distribution-1", packages: [] };
  }
  const data = JSON.parse(readFileSync(path, "utf8")) as RegistryIndex;
  return { spec: data.spec, packages: data.packages ?? [] };
}

export function writeRegistryIndex(root: string, index: RegistryIndex): void {
  const path = join(root, "v1", "index.json");
  mkdirSync(join(path, ".."), { recursive: true });
  writeFileSync(
    path,
    `${JSON.stringify({ spec: index.spec, packages: index.packages }, null, 2)}\n`,
    "utf8",
  );
}

export function loadRegistryPackageMetadata(
  path: string,
  packageName: string,
): RegistryPackageMetadata {
  if (!existsSync(path)) {
    return { name: packageName, versions: [] };
  }
  return parseMetadata(readFileSync(path, "utf8"));
}

export function writeRegistryPackageMetadata(
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

export function metadataToHash(
  metadata: RegistryPackageMetadata,
): Record<string, unknown> {
  return {
    name: metadata.name,
    versions: metadata.versions.map((entry) => versionToHash(entry)),
  };
}

export function registryMetadataPath(
  root: string,
  packageName: string,
): string {
  return join(root, "v1", "packages", `${packageName}.json`);
}

export function registryArtifactPath(
  packageName: string,
  version: string,
): string {
  const artifactName = `${packageName.replaceAll("/", "-")}-${version}.praypkg`;
  return `v1/artifacts/${packageName}/${version}/${artifactName}`;
}

export function writeOutputBytes(path: string, bytes: Buffer): void {
  mkdirSync(join(path, ".."), { recursive: true });
  writeFileSync(path, bytes);
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
  if (entry.publishedAt !== undefined) hash.published_at = entry.publishedAt;
  if (entry.signature) hash.signature = entry.signature;
  return hash;
}
