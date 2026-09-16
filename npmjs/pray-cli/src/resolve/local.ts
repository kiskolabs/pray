import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { PrayError } from "../errors.js";
import { normalizeLineEndings, sha256Prefixed } from "../hashing.js";
import type { ManifestLocal } from "../manifest/types.js";
import type { ResolvedLocalFile } from "./types.js";

export function resolveLocalFile(
  projectRoot: string,
  declaration: ManifestLocal,
): ResolvedLocalFile {
  const path = resolve(projectRoot, declaration.path);
  if (!existsSync(path)) {
    if (declaration.optional) {
      return {
        path,
        manifestPath: declaration.path,
        content: "",
        sourceChecksum: sha256Prefixed(""),
        position: declaration.position,
        optional: true,
      };
    }
    throw PrayError.resolution(missingLocalEmbedGuidance(declaration.path));
  }
  const content = normalizeLineEndings(readFileSync(path, "utf8"));
  return {
    path,
    manifestPath: declaration.path,
    content,
    sourceChecksum: sha256Prefixed(content),
    position: declaration.position,
    optional: declaration.optional,
  };
}

export function missingLocalEmbedGuidance(path: string): string {
  return (
    `Prayfile lists \`local "${path}"\` but the file does not exist. ` +
    "Create the file or remove the entry from Prayfile, then run `pray install`."
  );
}
