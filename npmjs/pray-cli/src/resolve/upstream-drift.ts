import { prepareGitSources } from "../git/sources.js";
import type { Lockfile } from "../lockfile/types.js";
import type { ResolveOptions } from "./context.js";
import { resolvePackage } from "./project.js";
import { sourceMap } from "./source-map.js";
import type { ResolvedProject } from "./types.js";
import { contentFileBytes, localContentForRefresh } from "./upstream-io.js";
import { overlayDriftLine, overlayFileChanges } from "./upstream-merge.js";

export async function pathForkDriftLines(
  project: ResolvedProject,
  previous: Lockfile | undefined,
  options: ResolveOptions,
): Promise<string[]> {
  const gitSources = prepareGitSources(
    project.projectRoot,
    project.manifest.sources,
    previous,
    options.refreshSourceRevisions,
    options.offline,
  );
  const sources = sourceMap(project.manifest.sources);
  const lines: string[] = [];
  for (const packageEntry of project.packages) {
    if (!packageEntry.declaration.path || !packageEntry.upstream) {
      continue;
    }
    const upstream = packageEntry.upstream;
    const declaration = {
      name: upstream.name,
      constraint: `= ${upstream.version}`,
      ...(upstream.source ? { source: upstream.source } : {}),
      exports: [],
      targets: [],
      features: [],
      groups: [],
      optional: false,
      roles: [],
      bound: false,
    };
    const resolved = await resolvePackage(
      project.projectRoot,
      sources,
      gitSources,
      declaration,
      previous,
      options,
    );
    const upstreamContent = contentFileBytes(resolved.root, resolved.spec);
    const localContent = localContentForRefresh(
      packageEntry.root,
      packageEntry.spec,
    );
    if (localContent.size === 0) {
      lines.push(
        `${packageEntry.declaration.name} has no content files; run pray install to copy ${upstream.name} ${upstream.version}`,
      );
      continue;
    }
    for (const [path, change] of overlayFileChanges(
      localContent,
      upstreamContent,
    )) {
      lines.push(
        overlayDriftLine(
          packageEntry.declaration.name,
          upstream.name,
          upstream.version,
          path,
          change,
        ),
      );
    }
  }
  return lines;
}
