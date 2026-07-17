import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { unpackPraypkg } from "../archive/praypkg.js";
import { PrayError } from "../errors.js";
import { sha256Prefixed } from "../hashing.js";
import { httpGet, joinUrl } from "../http/client.js";
import { fetchPackageMetadata } from "../registry/index.js";
import type { RegistryPackageMetadata } from "../registry/types.js";

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

    const metadata = await fetchPackageIndex(peer);
    for (const packageName of metadata.packages) {
      const packageMetadata = await fetchPackageMetadata(peer, packageName);
      const latest = packageMetadata.versions
        .filter((entry) => !entry.yanked)
        .at(-1);
      if (!latest) {
        continue;
      }
      const artifactBytes = await httpGet(joinUrl(peer, latest.artifact));
      if (
        latest.artifactHash &&
        sha256Prefixed(artifactBytes) !== latest.artifactHash
      ) {
        throw PrayError.integrity(`artifact hash mismatch for ${packageName}`);
      }
      const cacheDirectory = join(
        distributionRoot,
        ".pray",
        "sync-staging",
        packageName.replaceAll("/", "-"),
        latest.version,
      );
      mkdirSync(cacheDirectory, { recursive: true });
      unpackPraypkg(artifactBytes, cacheDirectory);
      writeRegistryPackageMetadataLocal(
        join(distributionRoot, "v1", "packages", `${packageName}.json`),
        packageMetadata,
      );
      mkdirSync(join(distributionRoot, latest.artifact, ".."), {
        recursive: true,
      });
      writeFileSync(join(distributionRoot, latest.artifact), artifactBytes);
      syncedPackages.add(packageName);
    }
  }

  writeFileSync(
    join(distributionRoot, "v1", "index.json"),
    `${JSON.stringify(
      {
        spec: "prayfile-distribution-1",
        packages: [...syncedPackages].sort(),
      },
      null,
      2,
    )}\n`,
    "utf8",
  );

  return { peers: [...visited], packages: [...syncedPackages].sort() };
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

export async function loginCommand(): Promise<never> {
  throw PrayError.unsupported(
    "login requires passkey or SSH agent authentication and is not implemented yet in pray-cli typescript",
  );
}
