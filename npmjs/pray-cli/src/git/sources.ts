import type { Lockfile } from "../lockfile/types.js";
import type { ManifestSource } from "../manifest/types.js";
import { ensureGitRepository } from "./cache.js";
import {
  isLocalFilesystemSource,
  localGitRepoPath,
  localGitSourceRoot,
} from "./local-root.js";

export { gitSourceCacheDirectory } from "./cache.js";
export {
  discoverDistributionRoot,
  localDistributionRoot,
  resolveDistributionRoot,
} from "./distribution-root.js";
export { localGitSourceRoot } from "./local-root.js";

export interface GitSourceCheckout {
  cacheDirectory: string;
  revision: string;
  subdir?: string;
}

export class GitSourceSet {
  private readonly checkouts = new Map<string, GitSourceCheckout>();
  private readonly sourcesByName: Map<string, ManifestSource>;

  constructor(
    private readonly projectRoot: string,
    sources: ManifestSource[],
    private readonly lockfile: Lockfile | undefined,
    private readonly refresh: boolean,
  ) {
    this.sourcesByName = new Map(
      sources
        .filter((source) => source.kind === "git")
        .map((source) => [source.name, source]),
    );
  }

  get(name: string): GitSourceCheckout | undefined {
    const cached = this.checkouts.get(name);
    if (cached) {
      return cached;
    }
    const source = this.sourcesByName.get(name);
    if (!source) {
      return undefined;
    }
    const checkout = prepareOneGitSource(
      this.projectRoot,
      source,
      this.lockfile,
      this.refresh,
    );
    if (!checkout) {
      return undefined;
    }
    this.checkouts.set(name, checkout);
    return checkout;
  }

  entries(): IterableIterator<[string, GitSourceCheckout]> {
    return this.checkouts.entries();
  }
}

export function prepareGitSources(
  projectRoot: string,
  sources: ManifestSource[],
  lockfile: Lockfile | undefined,
  refresh = false,
): GitSourceSet {
  return new GitSourceSet(projectRoot, sources, lockfile, refresh);
}

function prepareOneGitSource(
  projectRoot: string,
  source: ManifestSource,
  lockfile: Lockfile | undefined,
  refresh: boolean,
): GitSourceCheckout | undefined {
  const cloneUrl = source.url.replace(/^git\+/, "");
  if (
    isLocalFilesystemSource(cloneUrl) &&
    !localGitRepoPath(projectRoot, cloneUrl)
  ) {
    const sourceRoot = localGitSourceRoot(projectRoot, cloneUrl);
    if (!sourceRoot) {
      return undefined;
    }
    return {
      cacheDirectory: sourceRoot,
      revision: "",
      subdir: source.subdir,
    };
  }
  const pinnedRevision = refresh
    ? undefined
    : pinnedRevisionForSource(lockfile, source);
  const { cacheDirectory, revision } = ensureGitRepository(
    projectRoot,
    cloneUrl,
    refresh,
    pinnedRevision,
    source.subdir,
  );
  return {
    cacheDirectory,
    revision,
    subdir: source.subdir,
  };
}

function pinnedRevisionForSource(
  lockfile: Lockfile | undefined,
  source: ManifestSource,
): string | undefined {
  const locked = lockfile?.source.find(
    (entry) => entry.name === source.name && entry.kind === "git",
  );
  if (locked?.revision) {
    return locked.revision;
  }
  if (source.kind === "git") {
    return source.rev ?? source.tag;
  }
  return undefined;
}
