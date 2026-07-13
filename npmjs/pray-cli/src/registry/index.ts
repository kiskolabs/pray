import { existsSync, mkdirSync, readFileSync, rmSync } from "node:fs";
import { join } from "node:path";
import semver from "semver";
import { unpackPraypkg } from "../archive/praypkg.js";
import { versionSatisfies } from "../constraint.js";
import { PrayError } from "../errors.js";
import { sha256Hex, sha256Prefixed } from "../hashing.js";
import {
  httpGet,
  httpGetText,
  isLocalSourceUrl,
  joinUrl,
} from "../http/client.js";
import type { ManifestPackage } from "../manifest/types.js";
import {
  findPrayspecFile,
  parsePackageSpec,
  treeHashForRoot,
} from "../package-spec/index.js";
import type {
  RegistryPackageMetadata,
  RegistryPackageResolution,
  RegistryPackageVersion,
} from "./types.js";

export async function resolveRegistryPackageRoot(
  projectRoot: string,
  sourceUrl: string,
  declaration: ManifestPackage,
  options: RegistryResolveOptions = {},
): Promise<RegistryPackageResolution> {
  const metadata = await fetchPackageMetadata(sourceUrl, declaration.name);
  const registryLatestVersion = registryLatestVersionLabel(metadata);
  const selected = selectPackageVersion(
    metadata,
    declaration.constraint,
    options.preferredVersion,
  );
  const cacheDirectory = registryCacheDirectory(
    projectRoot,
    sourceUrl,
    declaration.name,
    selected.version,
    selected.artifactHash,
  );

  if (cacheReady(cacheDirectory, selected)) {
    return {
      root: cacheDirectory,
      signerFingerprint: selected.signerFingerprint,
      registryLatestVersion,
    };
  }

  if (options.offline) {
    throw PrayError.resolution(
      offlinePackageError(declaration.name, selected.version),
    );
  }

  if (existsSync(cacheDirectory)) {
    rmSync(cacheDirectory, { recursive: true, force: true });
  }
  mkdirSync(cacheDirectory, { recursive: true });

  const artifactBytes = await readArtifactBytes(sourceUrl, selected.artifact);
  validateAndUnpack(cacheDirectory, declaration, selected, artifactBytes);

  return {
    root: cacheDirectory,
    signerFingerprint: selected.signerFingerprint,
    registryLatestVersion,
  };
}

export async function resolveLocalRegistryPackageRoot(
  projectRoot: string,
  sourceKey: string,
  sourceRoot: string,
  declaration: ManifestPackage,
  options: RegistryResolveOptions = {},
): Promise<RegistryPackageResolution> {
  const metadataPath = join(sourceRoot, "v1", "packages", `${declaration.name}.json`);
  if (!existsSync(metadataPath)) {
    throw PrayError.resolution(
      `package ${declaration.name} not found in distribution ${sourceRoot}. ` +
        `Missing ${metadataPath}. Check the package name, version constraint \`${declaration.constraint}\`, ` +
        "and that the source publishes registry metadata.",
    );
  }

  const metadata = parseMetadata(readFileSync(metadataPath, "utf8"));
  const registryLatestVersion = registryLatestVersionLabel(metadata);
  const selected = selectPackageVersion(
    metadata,
    declaration.constraint,
    options.preferredVersion,
  );
  const cacheDirectory = registryCacheDirectory(
    projectRoot,
    sourceKey,
    declaration.name,
    selected.version,
    selected.artifactHash,
  );

  if (cacheReady(cacheDirectory, selected)) {
    return {
      root: cacheDirectory,
      signerFingerprint: selected.signerFingerprint,
      registryLatestVersion,
    };
  }

  if (options.offline) {
    throw PrayError.resolution(
      offlinePackageError(declaration.name, selected.version),
    );
  }

  if (existsSync(cacheDirectory)) {
    rmSync(cacheDirectory, { recursive: true, force: true });
  }
  mkdirSync(cacheDirectory, { recursive: true });

  const artifactBytes = readLocalArtifactBytes(sourceRoot, selected.artifact);
  validateAndUnpack(cacheDirectory, declaration, selected, artifactBytes);

  return {
    root: cacheDirectory,
    signerFingerprint: selected.signerFingerprint,
    registryLatestVersion,
  };
}

export interface RegistryResolveOptions {
  preferredVersion?: string;
  offline?: boolean;
}

export async function fetchPackageMetadata(
  sourceUrl: string,
  packageName: string,
): Promise<RegistryPackageMetadata> {
  if (isLocalSourceUrl(sourceUrl)) {
    const metadataPath = join(sourceUrl, "v1", "packages", `${packageName}.json`);
    return parseMetadata(readFileSync(metadataPath, "utf8"));
  }
  return parseMetadata(
    await httpGetText(joinUrl(sourceUrl, `v1/packages/${packageName}.json`)),
  );
}

export function parseMetadata(text: string): RegistryPackageMetadata {
  const data = JSON.parse(text) as {
    name: string;
    versions: Record<string, unknown>[];
  };
  return {
    name: data.name,
    versions: data.versions.map((entry) => versionFromHash(entry)),
  };
}

export function versionFromHash(
  entry: Record<string, unknown>,
): RegistryPackageVersion {
  return {
    version: String(entry.version),
    artifact: String(entry.artifact),
    artifactHash: entry.artifact_hash ? String(entry.artifact_hash) : undefined,
    treeHash: entry.tree_hash ? String(entry.tree_hash) : undefined,
    yanked: Boolean(entry.yanked),
    targets: Array.isArray(entry.targets)
      ? entry.targets.map((value) => String(value))
      : [],
    exports: Array.isArray(entry.exports)
      ? entry.exports.map((value) => String(value))
      : [],
    signer: entry.signer ? String(entry.signer) : undefined,
    signerFingerprint: entry.signer_fingerprint
      ? String(entry.signer_fingerprint)
      : undefined,
    publishedAt: entry.published_at ? String(entry.published_at) : undefined,
    signature: entry.signature ? String(entry.signature) : undefined,
  };
}

export function selectPackageVersion(
  metadata: RegistryPackageMetadata,
  constraint: string,
  preferredVersion?: string,
): RegistryPackageVersion {
  if (preferredVersion) {
    const preferred = metadata.versions.find(
      (entry) => entry.version === preferredVersion && !entry.yanked,
    );
    if (
      preferred &&
      versionSatisfies(preferred.version, constraint)
    ) {
      return preferred;
    }
  }

  let selected: RegistryPackageVersion | undefined;
  for (const version of metadata.versions) {
    if (version.yanked) {
      continue;
    }
    if (!versionSatisfies(version.version, constraint)) {
      continue;
    }
    if (
      !selected ||
      semver.gt(version.version, selected.version)
    ) {
      selected = version;
    }
  }

  if (!selected) {
    throw PrayError.resolution(
      `no registry version for ${metadata.name} satisfies ${constraint}`,
    );
  }

  return selected;
}

export function registryLatestVersionLabel(
  metadata: RegistryPackageMetadata,
): string | undefined {
  let highest: RegistryPackageVersion | undefined;
  for (const version of metadata.versions) {
    if (version.yanked) {
      continue;
    }
    if (!highest || semver.gt(version.version, highest.version)) {
      highest = version;
    }
  }
  return highest?.version;
}

export function registryCacheDirectory(
  projectRoot: string,
  sourceKey: string,
  packageName: string,
  version: string,
  artifactHash?: string,
): string {
  const identifier = [
    sourceKey,
    packageName,
    version,
    artifactHash ?? "no-artifact-hash",
  ].join(":");
  const digest = sha256Hex(identifier).slice(0, 16);
  return join(
    projectRoot,
    ".pray",
    "cache",
    "registry",
    packageName.replaceAll("/", "-"),
    version,
    digest,
  );
}

export function registryArtifactSignature(
  artifactBytes: Buffer,
  treeHash: string,
  signer: string,
): string {
  const payload = Buffer.concat([
    artifactBytes,
    Buffer.from("\0"),
    Buffer.from(treeHash, "utf8"),
    Buffer.from("\0"),
    Buffer.from(signer, "utf8"),
  ]);
  return sha256Prefixed(payload);
}

function validateAndUnpack(
  cacheDirectory: string,
  declaration: ManifestPackage,
  selected: RegistryPackageVersion,
  artifactBytes: Buffer,
): void {
  if (selected.artifactHash) {
    const artifactHash = sha256Prefixed(artifactBytes);
    if (artifactHash !== selected.artifactHash) {
      throw PrayError.integrity(
        `package artifact hash mismatch for ${declaration.name} ${selected.version}`,
      );
    }
  }

  unpackPraypkg(artifactBytes, cacheDirectory);
  const specPath = findPrayspecFile(cacheDirectory);
  const spec = parsePackageSpec(readFileSync(specPath, "utf8"));
  if (spec.name !== declaration.name) {
    throw PrayError.resolution(
      `package path ${cacheDirectory} declares ${spec.name}, expected ${declaration.name}`,
    );
  }
  if (spec.version !== selected.version) {
    throw PrayError.resolution(
      `package ${declaration.name} version ${spec.version} does not match registry version ${selected.version}`,
    );
  }
  if (selected.treeHash) {
    const treeHash = treeHashForRoot(cacheDirectory, spec);
    if (treeHash !== selected.treeHash) {
      throw PrayError.integrity(
        `package tree hash mismatch for ${declaration.name} ${selected.version}`,
      );
    }
  }
}

function cacheReady(
  cacheDirectory: string,
  selected: RegistryPackageVersion,
): boolean {
  if (!existsSync(cacheDirectory)) {
    return false;
  }
  try {
    const specPath = findPrayspecFile(cacheDirectory);
    const spec = parsePackageSpec(readFileSync(specPath, "utf8"));
    return spec.version === selected.version;
  } catch {
    return false;
  }
}

async function readArtifactBytes(sourceUrl: string, artifact: string): Promise<Buffer> {
  if (isLocalSourceUrl(sourceUrl)) {
    return readLocalArtifactBytes(sourceUrl, artifact);
  }
  if (artifact.startsWith("http://") || artifact.startsWith("https://")) {
    return httpGet(artifact);
  }
  return httpGet(joinUrl(sourceUrl, artifact));
}

function readLocalArtifactBytes(sourceRoot: string, artifact: string): Buffer {
  if (artifact.startsWith("http://") || artifact.startsWith("https://")) {
    throw PrayError.unsupported(
      "remote artifact URLs in local distribution require async fetch",
    );
  }
  if (artifact.startsWith("file://")) {
    return readFileSync(artifact.slice("file://".length));
  }
  const path = join(sourceRoot, artifact);
  if (!existsSync(path)) {
    throw PrayError.resolution(`package artifact missing at ${path}`);
  }
  return readFileSync(path);
}

function offlinePackageError(packageName: string, version: string): string {
  return `package ${packageName} ${version} is not cached and offline mode is enabled`;
}

export { joinUrl } from "../http/client.js";
