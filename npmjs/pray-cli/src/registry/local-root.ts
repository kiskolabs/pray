import { isAbsolute, resolve } from "node:path";

export function localRegistryRoot(
  projectRoot: string,
  sourceUrl: string,
): string {
  const path = sourceUrl.startsWith("file://")
    ? sourceUrl.slice("file://".length)
    : sourceUrl;
  return isAbsolute(path) ? path : resolve(projectRoot, path);
}
