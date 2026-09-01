import { join } from "node:path";
import { sha256Hex } from "../hashing.js";
import {
  resolveDistributionPath,
  validatePackageName,
  validatePathSegment,
} from "../sync/path-safety.js";

export function registryCacheDirectory(
  projectRoot: string,
  sourceKey: string,
  packageName: string,
  version: string,
  artifactHash?: string,
): string {
  validatePackageName(packageName);
  validatePathSegment(version, "package version");
  const identifier = [
    sourceKey,
    packageName,
    version,
    artifactHash ?? "no-artifact-hash",
  ].join(":");
  const digest = sha256Hex(identifier).slice(0, 16);
  return resolveDistributionPath(
    projectRoot,
    join(
      ".pray",
      "cache",
      "registry",
      packageName.replaceAll("/", "-"),
      version,
      digest,
    ),
  );
}
