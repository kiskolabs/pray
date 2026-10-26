import { sha256Prefixed } from "../hashing.js";
import { validatePublishRemotes } from "../publish/remote-validate.js";
import { parseManifestText } from "./parser.js";
import { type Manifest, manifestToJson } from "./types.js";
import { validateManifestPaths } from "./validate.js";

/** Parse and validate Prayfile text into a {@link Manifest}. */
export function parseManifest(text: string): Manifest {
  const manifest = parseManifestText(text);
  validateManifestPaths(manifest);
  validatePublishRemotes(manifest);
  return manifest;
}

/** Canonical hash of a manifest used for lockfile and resolve identity. */
export function manifestHash(manifest: Manifest): string {
  const bytes = new TextEncoder().encode(
    JSON.stringify(manifestToJson(manifest)),
  );
  return sha256Prefixed(bytes);
}
