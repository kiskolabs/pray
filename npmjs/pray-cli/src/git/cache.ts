import { materializeCatalogTree } from "./materialize.js";
import { gitSourceCacheDirectory } from "./paths.js";
import { ensureGlobalGitDb } from "./store.js";

export { gitSourceCacheDirectory } from "./paths.js";

export function ensureGitRepository(
  projectRoot: string,
  cloneUrl: string,
  refresh: boolean,
  pinnedRevision?: string,
  sparseSubdir?: string,
  offline = false,
): { cacheDirectory: string; revision: string } {
  const { db, revision } = ensureGlobalGitDb(
    cloneUrl,
    pinnedRevision,
    refresh,
    offline,
    projectRoot,
  );
  const cacheDirectory = gitSourceCacheDirectory(
    projectRoot,
    cloneUrl,
    sparseSubdir,
  );
  materializeCatalogTree(db, cacheDirectory, revision, sparseSubdir, refresh);
  return { cacheDirectory, revision };
}
