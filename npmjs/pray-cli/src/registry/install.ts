import {
  existsSync,
  mkdirSync,
  readFileSync,
  renameSync,
  rmSync,
} from "node:fs";
import { unpackPraypkg } from "../archive/praypkg.js";
import { PrayError } from "../errors.js";
import { sha256Prefixed } from "../hashing.js";
import type { ManifestPackage } from "../manifest/types.js";
import {
  findPrayspecFile,
  parsePackageSpec,
  treeHashForRoot,
} from "../package-spec/index.js";
import { verifyRegistrySignature } from "./signature.js";
import type { RegistryPackageVersion } from "./types.js";

export function installArtifactToCache(
  cacheDirectory: string,
  declaration: ManifestPackage,
  selected: RegistryPackageVersion,
  artifactBytes: Buffer,
): void {
  requireIntegrityFields(declaration.name, selected);
  const stagingDirectory = `${cacheDirectory}.staging`;
  rmSync(stagingDirectory, { recursive: true, force: true });
  mkdirSync(stagingDirectory, { recursive: true });
  const unpackedDirectory = `${stagingDirectory}/unpacked`;
  mkdirSync(unpackedDirectory, { recursive: true });

  try {
    validateAndUnpack(unpackedDirectory, declaration, selected, artifactBytes);
    if (existsSync(cacheDirectory)) {
      rmSync(cacheDirectory, { recursive: true, force: true });
    }
    renameSync(unpackedDirectory, cacheDirectory);
  } catch (error) {
    rmSync(stagingDirectory, { recursive: true, force: true });
    throw error;
  }
  rmSync(stagingDirectory, { recursive: true, force: true });
}

export function requireIntegrityFields(
  packageName: string,
  selected: RegistryPackageVersion,
): void {
  if (!selected.artifactHash) {
    throw PrayError.integrity(
      `package ${packageName} ${selected.version} is missing artifact_hash`,
    );
  }
  if (!selected.treeHash) {
    throw PrayError.integrity(
      `package ${packageName} ${selected.version} is missing tree_hash`,
    );
  }
}

function validateAndUnpack(
  cacheDirectory: string,
  declaration: ManifestPackage,
  selected: RegistryPackageVersion,
  artifactBytes: Buffer,
): void {
  const artifactHash = sha256Prefixed(artifactBytes);
  if (artifactHash !== selected.artifactHash) {
    throw PrayError.integrity(
      `package artifact hash mismatch for ${declaration.name} ${selected.version}`,
    );
  }

  verifyRegistrySignature(declaration.name, selected, artifactBytes);
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
  const treeHash = treeHashForRoot(cacheDirectory, spec);
  if (treeHash !== selected.treeHash) {
    throw PrayError.integrity(
      `package tree hash mismatch for ${declaration.name} ${selected.version}`,
    );
  }
}
