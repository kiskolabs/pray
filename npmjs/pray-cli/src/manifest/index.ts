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

export function formatPackageDeclaration(
  packageEntry: ManifestPackage,
): string {
  const parts = [`agent "${packageEntry.name}"`];
  if (packageEntry.constraint !== "*") {
    parts.push(`"${packageEntry.constraint}"`);
  }
  if (packageEntry.path) {
    parts.push(`path: "${packageEntry.path}"`);
  }
  if (packageEntry.source) {
    parts.push(`source: "${packageEntry.source}"`);
  }
  if (packageEntry.exports.length > 0) {
    parts.push(
      `exports: [${packageEntry.exports.map((value) => `"${value}"`).join(", ")}]`,
    );
  }
  if (packageEntry.optional) {
    parts.push("optional: true");
  }
  return parts.join(", ");
}

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
