import { packageMatchesEnvironment } from "../environment.js";
import { targetEntries } from "../manifest/destination.js";
import type { ManifestTarget } from "../manifest/types.js";
import type { ResolvedProject } from "../resolve/types.js";
import { ContentBuilder } from "./content-builder.js";
import { appendHeaderIfEnabled } from "./header.js";
import { appendManagedExport, shouldInlineExport } from "./managed-export.js";
import type { RenderedTarget } from "./types.js";

export function renderScopedCompose(
  project: ResolvedProject,
  target: ManifestTarget,
  output: string,
): RenderedTarget {
  const builder = new ContentBuilder();
  appendHeaderIfEnabled(builder, project, output);

  const managedSpans: RenderedTarget["managedSpans"] = [];
  for (const entry of targetEntries(target)) {
    if (entry.kind === "local") {
      appendLocalEntry(builder, project, entry.path);
      continue;
    }
    appendPackageEntry(
      builder,
      managedSpans,
      project,
      target,
      output,
      entry.name,
    );
  }

  return {
    path: output,
    content: builder.finish(),
    managedSpans,
  };
}

function appendLocalEntry(
  builder: ContentBuilder,
  project: ResolvedProject,
  path: string,
): void {
  const local = project.localFiles.find(
    (candidate) => candidate.manifestPath === path,
  );
  if (!local) {
    return;
  }
  if (local.content.length === 0 && local.optional) {
    return;
  }
  builder.appendBody(local.content);
  builder.appendEmptyLine();
}

function appendPackageEntry(
  builder: ContentBuilder,
  managedSpans: RenderedTarget["managedSpans"],
  project: ResolvedProject,
  target: ManifestTarget,
  output: string,
  packageName: string,
): void {
  const packageEntry = project.packages.find(
    (candidate) => candidate.declaration.name === packageName,
  );
  if (!packageEntry) {
    return;
  }
  if (
    !packageMatchesEnvironment(
      packageEntry.declaration.groups,
      project.environment,
    )
  ) {
    return;
  }
  for (const exportName of packageEntry.selectedExports) {
    if (!shouldInlineExport(packageEntry, exportName)) {
      continue;
    }
    appendManagedExport(
      builder,
      managedSpans,
      packageEntry,
      exportName,
      target,
      output,
    );
  }
}
