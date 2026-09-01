import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { PrayError } from "../errors.js";
import { sha256Prefixed } from "../hashing.js";
import { httpGet, joinUrl } from "../http/client.js";
import type { ManifestPackage } from "../manifest/types.js";
import { fetchPackageMetadata } from "../registry/index.js";
import {
  installArtifactToCache,
  requireIntegrityFields,
} from "../registry/install.js";
import type { RegistryPackageMetadata } from "../registry/types.js";
import { MAX_FEDERATION_PEERS } from "../resource-limits.js";
import {
  rejectAbsoluteArtifactPath,
  resolveDistributionPath,
  validatePackageName,
  validatePathSegment,
} from "./path-safety.js";

export interface SyncSummary {
  peers: string[];
  packages: string[];
}

export async function syncDistributionRoot(
  root: string,
  peers: string[],
): Promise<SyncSummary> {
  const distributionRoot = root;
  const queue = [...peers];
  const visited = new Set<string>();
  const syncedPackages = new Set<string>();

  while (queue.length > 0) {
    const peer = queue.shift()!;
    if (visited.has(peer)) {
      continue;
    }
    visited.add(peer);
    if (visited.size > MAX_FEDERATION_PEERS) {
      throw PrayError.resolution(
        `federation exceeds ${MAX_FEDERATION_PEERS} peers`,
      );
    }

    const metadata = await fetchPackageIndex(peer);
    for (const rawPackageName of metadata.packages) {
      const packageName = validatePackageName(rawPackageName);
      const packageMetadata = await fetchPackageMetadata(peer, packageName);
      if (packageMetadata.name !== packageName) {
        throw PrayError.integrity(`metadata name mismatch for ${packageName}`);
      }
      const latest = packageMetadata.versions
        .filter((entry) => !entry.yanked)
        .at(-1);
      if (!latest) {
        continue;
      }
      validatePathSegment(latest.version, "package version");
      requireIntegrityFields(packageName, latest);
      rejectAbsoluteArtifactPath(latest.artifact);
      const artifactBytes = await httpGet(joinUrl(peer, latest.artifact));
      if (sha256Prefixed(artifactBytes) !== latest.artifactHash) {
        throw PrayError.integrity(`artifact hash mismatch for ${packageName}`);
      }
      const cacheDirectory = resolveDistributionPath(
        distributionRoot,
        join(
          ".pray",
          "sync-staging",
          packageName.replaceAll("/", "-"),
          latest.version,
        ),
      );
      installArtifactToCache(
        cacheDirectory,
        { name: packageName } as ManifestPackage,
        latest,
        artifactBytes,
      );
      writeRegistryPackageMetadataLocal(
        resolveDistributionPath(
          distributionRoot,
          join("v1", "packages", `${packageName}.json`),
        ),
        packageMetadata,
      );
      const artifactPath = resolveDistributionPath(
        distributionRoot,
        latest.artifact,
      );
      writeRegistryBytes(artifactPath, artifactBytes);
      syncedPackages.add(packageName);
    }
  }

  writeRegistryBytes(
    resolveDistributionPath(distributionRoot, "v1/index.json"),
    Buffer.from(
      `${JSON.stringify(
        {
          spec: "prayfile-distribution-1",
          packages: [...syncedPackages].sort(),
        },
        null,
        2,
      )}\n`,
      "utf8",
    ),
  );

  return { peers: [...visited], packages: [...syncedPackages].sort() };
}

function writeRegistryBytes(path: string, bytes: Buffer): void {
  const parent = join(path, "..");
  mkdirSync(parent, { recursive: true });
  writeFileSync(path, bytes);
}

async function fetchPackageIndex(
  peer: string,
): Promise<{ packages: string[] }> {
  const text = await httpGet(joinUrl(peer, "v1/index.json")).then((buffer) =>
    buffer.toString("utf8"),
  );
  const data = JSON.parse(text) as { packages?: string[] };
  return { packages: data.packages ?? [] };
}

function writeRegistryPackageMetadataLocal(
  path: string,
  metadata: RegistryPackageMetadata,
): void {
  mkdirSync(join(path, ".."), { recursive: true });
  writeFileSync(
    path,
    `${JSON.stringify(
      {
        name: metadata.name,
        versions: metadata.versions.map((entry) => ({
          version: entry.version,
          artifact: entry.artifact,
          artifact_hash: entry.artifactHash,
          tree_hash: entry.treeHash,
          yanked: entry.yanked,
          targets: entry.targets,
          exports: entry.exports,
          signer: entry.signer,
          signer_fingerprint: entry.signerFingerprint,
          published_at: entry.publishedAt,
          signature: entry.signature,
        })),
      },
      null,
      2,
    )}\n`,
    "utf8",
  );
}
