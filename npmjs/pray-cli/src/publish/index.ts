import { join } from "node:path";
import { buildPackageArchiveBytes } from "../archive/praypkg.js";
import { allowsTorrent, readDistributionSettings } from "../distribution.js";
import { sha256Prefixed } from "../hashing.js";
import { httpPut, joinUrl } from "../http/client.js";
import { requireReleaseVersion } from "../package-spec/index.js";
import { deriveRegistryDerivedMetadataFromRoot } from "../registry/derived-metadata.js";
import { registryArtifactSignature } from "../registry/index.js";
import type {
  RegistryPackageMetadata,
  RegistryPackageVersion,
} from "../registry/types.js";
import type { ResolvedPackage, ResolvedProject } from "../resolve/types.js";
import { storedPackageArtifact, storedPublishMatches } from "./integrity.js";
import {
  loadRegistryIndex,
  loadRegistryPackageMetadata,
  metadataToHash,
  registryArtifactPath,
  registryMetadataPath,
  writeOutputBytes,
  writeRegistryIndex,
  writeRegistryPackageMetadata,
} from "./registry-store.js";
import {
  emptyPublishStats,
  type PublishStats,
  reportPublished,
  reportUnchanged,
} from "./report.js";
import {
  fetchDistributionSettings,
  torrentManifestBytes,
  torrentManifestPath,
  writeTorrentManifest,
} from "./torrent-manifest.js";

export {
  initDistributionRoot,
  registryArtifactPath,
} from "./registry-store.js";

export async function publishToRoot(
  project: ResolvedProject,
  root: string,
  signer = "local",
  signerFingerprint?: string,
  destName = root,
  target = root,
): Promise<PublishStats> {
  const distributionRoot = root;
  const index = loadRegistryIndex(distributionRoot);
  const distribution = readDistributionSettings(distributionRoot);
  const packageNames = new Set(index.packages);
  const stats = emptyPublishStats();

  for (const packageEntry of project.packages) {
    requireReleaseVersion(packageEntry.spec);
    const artifactPath = registryArtifactPath(
      packageEntry.declaration.name,
      packageEntry.spec.version,
    );
    const metadataPath = registryMetadataPath(
      distributionRoot,
      packageEntry.declaration.name,
    );
    const metadata = loadRegistryPackageMetadata(
      metadataPath,
      packageEntry.declaration.name,
    );
    const existing = metadata.versions.find(
      (entry) => entry.version === packageEntry.spec.version,
    );
    const storedArtifact = storedPackageArtifact(
      distributionRoot,
      artifactPath,
      packageEntry,
      existing,
      distribution,
    );
    if (
      existing !== undefined &&
      storedArtifact !== undefined &&
      storedPublishMatches(storedArtifact, packageEntry, existing)
    ) {
      packageNames.add(packageEntry.declaration.name);
      writeRegistryPackageMetadata(metadataPath, metadata);
      reportUnchanged(packageEntry, destName, target);
      stats.unchanged += 1;
      continue;
    }

    const archiveBytes = buildPackageArchiveBytes(packageEntry);
    writeOutputBytes(join(distributionRoot, artifactPath), archiveBytes);
    writeTorrentManifest(
      distributionRoot,
      packageEntry.declaration.name,
      packageEntry.spec.version,
      artifactPath,
      archiveBytes,
      distribution,
    );
    const versionEntry = publishedRegistryPackageVersion(
      packageEntry,
      signer,
      signerFingerprint,
      archiveBytes,
      artifactPath,
    );
    preserveExistingPublishMetadata(metadata, versionEntry, storedArtifact);
    metadata.versions = metadata.versions.filter(
      (entry) => entry.version !== versionEntry.version,
    );
    metadata.versions.push(versionEntry);
    writeRegistryPackageMetadata(metadataPath, metadata);
    packageNames.add(packageEntry.declaration.name);
    reportPublished(packageEntry, destName, target, artifactPath);
    stats.published += 1;
  }

  index.packages = [...packageNames].sort();
  writeRegistryIndex(distributionRoot, index);
  return stats;
}

export async function publishToServer(
  project: ResolvedProject,
  serverUrl: string,
  signer = "local",
  signerFingerprint?: string,
  destName = serverUrl,
  target = serverUrl,
): Promise<PublishStats> {
  const distribution = await fetchDistributionSettings(serverUrl);
  const stats = emptyPublishStats();
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
    if (allowsTorrent(distribution)) {
      await httpPut(
        joinUrl(serverUrl, torrentManifestPath(artifactPath)),
        "application/json",
        torrentManifestBytes(
          packageEntry.declaration.name,
          packageEntry.spec.version,
          artifactPath,
          archiveBytes,
          distribution.bootstrapTrackers,
        ),
      );
    }
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
    reportPublished(packageEntry, destName, target, artifactPath);
    stats.published += 1;
  }
  return stats;
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
    publishedAt: Math.floor(Date.now() / 1000),
    signature: registryArtifactSignature(
      archiveBytes,
      packageEntry.treeHash,
      signer,
    ),
    derivedMetadata: deriveRegistryDerivedMetadataFromRoot(
      packageEntry.root,
      packageEntry.spec,
    ),
  };
}

function preserveExistingPublishMetadata(
  metadata: RegistryPackageMetadata,
  versionEntry: RegistryPackageVersion,
  storedArtifact: Buffer | undefined,
): void {
  const existing = metadata.versions.find(
    (entry) => entry.version === versionEntry.version,
  );
  if (!existing) return;

  versionEntry.yanked = existing.yanked;
  if (storedArtifact !== undefined) {
    versionEntry.publishedAt = existing.publishedAt;
  }
  versionEntry.derivedMetadata ??= existing.derivedMetadata;
}
