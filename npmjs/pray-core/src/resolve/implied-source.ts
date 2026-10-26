import { PrayError } from "../errors.js";
import type { ManifestPackage, ManifestSource } from "../manifest/types.js";

function packageNamespace(name: string): string | undefined {
  const separator = name.indexOf("/");
  return separator === -1 ? undefined : name.slice(0, separator);
}

/** Resolve which declared source a package declaration implies. */
export function impliedSourceName(
  declaration: ManifestPackage,
  sources: Map<string, ManifestSource>,
): string | undefined {
  if (declaration.source) {
    return declaration.source;
  }
  const namespace = packageNamespace(declaration.name);
  if (namespace && sources.has(namespace)) {
    return namespace;
  }
  if (!declaration.name.includes("/")) {
    const pathSources = [...sources.entries()].filter(
      ([, source]) => source.kind === "path",
    );
    if (pathSources.length === 1) {
      return pathSources[0]?.[0];
    }
  }
  if (sources.size === 1) {
    return sources.keys().next()?.value;
  }
  if (sources.size > 1) {
    throw PrayError.resolution(
      `package ${declaration.name} requires source: when multiple sources are declared and the package namespace does not match a source`,
    );
  }
  return undefined;
}
