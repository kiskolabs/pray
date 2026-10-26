import type { ManifestSource } from "../manifest/types.js";

export function sourceMap(
  sources: ManifestSource[],
): Map<string, ManifestSource> {
  return new Map(sources.map((source) => [source.name, source]));
}
