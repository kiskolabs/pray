import { PrayError } from "../errors.js";
import { sha256Prefixed } from "../hashing.js";
import type { RegistryPackageVersion } from "./types.js";

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

export function verifyRegistrySignature(
  packageName: string,
  selected: RegistryPackageVersion,
  artifactBytes: Buffer,
): void {
  if (!selected.signature) {
    return;
  }
  if (!selected.signer || !selected.treeHash) {
    throw PrayError.integrity(
      `package signature metadata incomplete for ${packageName} ${selected.version}`,
    );
  }
  const expected = registryArtifactSignature(
    artifactBytes,
    selected.treeHash,
    selected.signer,
  );
  if (expected !== selected.signature) {
    throw PrayError.integrity(
      `package signature mismatch for ${packageName} ${selected.version}`,
    );
  }
}
