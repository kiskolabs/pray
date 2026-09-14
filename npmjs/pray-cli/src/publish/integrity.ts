import { existsSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { basename, join } from "node:path";
import { unpackPraypkg } from "../archive/praypkg.js";
import { sha256Prefixed } from "../hashing.js";
import { findPrayspecFile } from "../package-spec/index.js";
import { registryArtifactSignature } from "../registry/index.js";
import type { RegistryPackageVersion } from "../registry/types.js";
import type { ResolvedPackage } from "../resolve/types.js";

export function storedPackageArtifact(
  root: string,
  artifactPath: string,
  packageEntry: ResolvedPackage,
  existing: RegistryPackageVersion | undefined,
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
    storedPrayspecMatches(artifactBytes, packageEntry.root)
    ? artifactBytes
    : undefined;
}

export function storedPublishMatches(
  artifactBytes: Buffer,
  packageEntry: ResolvedPackage,
  signer: string,
  signerFingerprint: string | undefined,
  existing: RegistryPackageVersion,
): boolean {
  return (
    existing.signer === signer &&
    existing.signerFingerprint === signerFingerprint &&
    existing.signature ===
      registryArtifactSignature(artifactBytes, packageEntry.treeHash, signer)
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
