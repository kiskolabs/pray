import { existsSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { basename, join } from "node:path";
import { unpackPraypkg } from "../archive/praypkg.js";
import type { RegistryDistributionSettings } from "../distribution.js";
import { sha256Prefixed } from "../hashing.js";
import { findPrayspecFile } from "../package-spec/index.js";
import { registryArtifactSignature } from "../registry/index.js";
import type { RegistryPackageVersion } from "../registry/types.js";
import type { ResolvedPackage } from "../resolve/types.js";
import { torrentDescriptorPresent } from "./torrent-manifest.js";

export function storedPackageArtifact(
  root: string,
  artifactPath: string,
  packageEntry: ResolvedPackage,
  existing: RegistryPackageVersion | undefined,
  distribution: RegistryDistributionSettings,
): Buffer | undefined {
  if (
    existing?.artifact !== artifactPath ||
    existing.treeHash !== packageEntry.treeHash
  ) {
    return undefined;
  }
  const storedPath = join(root, artifactPath);
  if (!existsSync(storedPath)) return undefined;

  const artifactBytes = readFileSync(storedPath);
  return existing.artifactHash === sha256Prefixed(artifactBytes) &&
    storedPrayspecMatches(artifactBytes, packageEntry.root) &&
    torrentDescriptorPresent(root, artifactPath, distribution)
    ? artifactBytes
    : undefined;
}

// A stored row is current when its own recorded signature still authenticates the
// stored artifact. The check deliberately ignores who is publishing now: a row signed
// by another publisher is still a valid attestation of identical bytes, and rewriting it
// would restate authorship without republishing anything. A row carrying no signature is
// not current, so publish repairs it.
export function storedPublishMatches(
  artifactBytes: Buffer,
  packageEntry: ResolvedPackage,
  existing: RegistryPackageVersion,
): boolean {
  if (existing.signer === undefined || existing.signature === undefined) {
    return false;
  }
  return (
    existing.signature ===
    registryArtifactSignature(
      artifactBytes,
      packageEntry.treeHash,
      existing.signer,
    )
  );
}

function storedPrayspecMatches(
  artifactBytes: Buffer,
  packageRoot: string,
): boolean {
  const directory = mkdtempSync(join(tmpdir(), "pray-publish-check-"));
  try {
    unpackPraypkg(artifactBytes, directory);
    const storedPath = findPrayspecFile(directory);
    const currentPath = findPrayspecFile(packageRoot);
    return (
      basename(storedPath) === basename(currentPath) &&
      readFileSync(storedPath).equals(readFileSync(currentPath))
    );
  } catch {
    return false;
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
}
