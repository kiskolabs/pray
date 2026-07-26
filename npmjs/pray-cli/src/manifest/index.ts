import { readFileSync } from "node:fs";
import { PrayError } from "../errors.js";
import { sha256Prefixed } from "../hashing.js";
import { parseManifestText } from "./parser.js";
import {
  canonicalManifest,
  defaultRenderPolicy,
  type Manifest,
  type ManifestLocal,
  type ManifestPackage,
  type ManifestSource,
  type ManifestTarget,
  manifestToJson,
  type RenderPolicy,
} from "./types.js";

export function readManifestText(manifestPath: string): string {
  try {
    return readFileSync(manifestPath, "utf8");
  } catch (error) {
    if (
      typeof error === "object" &&
      error !== null &&
      "code" in error &&
      error.code === "ENOENT"
    ) {
      throw PrayError.manifest(
        `missing ${manifestPath}; run pray init to create one`,
      );
    }
    const message = error instanceof Error ? error.message : String(error);
    throw PrayError.io(message);
  }
}

export function parseManifest(text: string): Manifest {
  return parseManifestText(text);
}

export function manifestHash(manifest: Manifest): string {
  const bytes = Buffer.from(JSON.stringify(manifestToJson(manifest)), "utf8");
  return sha256Prefixed(bytes);
}

export {
  bindLocalEntry,
  bindPackageEntry,
  destinationTargetName,
  exportKindMatchesRole,
  isLocalPathForm,
  localBound,
  newDestinationTarget,
  packageBound,
  packageBoundToCompose,
  packageBoundToTree,
  packageRoles,
  roleForDestination,
  targetEntries,
  targetMode,
  targetScoped,
  upsertLocal,
  upsertPackage,
} from "./destination.js";
export {
  classifyFormatHints,
  formatRecommended,
  type PackageFormatHint,
  recommendManifest,
  usesDestinationDsl,
} from "./format-manifest.js";
export { serializeRecommended } from "./format-serialize.js";
export {
  formatPackageDeclaration,
  replacePackageDeclaration,
} from "./package-declaration.js";

export {
  canonicalManifest,
  defaultRenderPolicy,
  type Manifest,
  type ManifestLocal,
  type ManifestPackage,
  type ManifestSource,
  type ManifestTarget,
  type RenderPolicy,
};
