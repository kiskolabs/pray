import { join } from "node:path";
import { sha256Hex } from "../hashing.js";
import {
  resolveDistributionPath,
  validateRegistryCacheIdentity,
} from "../sync/path-safety.js";

export function registryCacheDirectory(
  projectRoot: string,
  sourceKey: string,
  packageName: string,
  version: string,
): string {
  const [namespace, name] = validateRegistryCacheIdentity(packageName, version);
  const sourceHash = sha256Hex(sourceKey).slice(0, 16);
  return resolveDistributionPath(
    projectRoot,
    join(".pray", "cache", "registry", namespace, name, version, sourceHash),
  );
}
