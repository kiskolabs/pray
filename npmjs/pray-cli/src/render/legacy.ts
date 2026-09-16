import { packageMatchesEnvironment } from "../environment.js";
import { packageBoundToCompose } from "../manifest/destination.js";
import type { ManifestTarget } from "../manifest/types.js";
import type { ResolvedLocalFile, ResolvedProject } from "../resolve/types.js";
import { ContentBuilder } from "./content-builder.js";
import { appendHeaderIfEnabled } from "./header.js";
import {
  appendManagedExport,
  appendManagedLocal,
  shouldInlineExport,
} from "./managed-export.js";
import type { RenderedTarget } from "./types.js";

export function renderLegacyCompose(
  project: ResolvedProject,
  target: ManifestTarget,
  output: string,
): RenderedTarget {
  const builder = new ContentBuilder();
  appendHeaderIfEnabled(builder, project, target, output);
  const managedSpans: RenderedTarget["managedSpans"] = [];
  appendUnboundLocals(builder, managedSpans, project, target, output);

  builder.appendLine("## Shared instructions");
  builder.appendEmptyLine();

  for (const packageEntry of project.packages) {
    if (
      !packageMatchesEnvironment(
        packageEntry.declaration.groups,
        project.environment,
      )
    ) {
      continue;
    }
    if (!packageBoundToCompose(packageEntry.declaration, target)) {
      continue;
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
        project.manifest.symbols ?? {},
      );
    }
  }

  return {
    path: output,
    content: builder.finish(),
    managedSpans,
  };
}

function appendUnboundLocals(
  builder: ContentBuilder,
  managedSpans: RenderedTarget["managedSpans"],
  project: ResolvedProject,
  target: ManifestTarget,
  output: string,
): void {
  const unboundLocals = project.localFiles.filter((local) =>
    isUnbound(project, local),
  );

  if (unboundLocals.length > 0) {
    builder.appendLine("## Additional instructions");
    builder.appendEmptyLine();
  }
  for (const local of unboundLocals) {
    if (local.content.length === 0 && local.optional) {
      continue;
    }
    builder.appendLine(`### ${local.manifestPath}`);
    appendManagedLocal(
      builder,
      managedSpans,
      local,
      target,
      output,
      project.manifest.symbols ?? {},
    );
  }
}

function isUnbound(
  project: ResolvedProject,
  local: ResolvedLocalFile,
): boolean {
  const entry = project.manifest.local.find(
    (candidate) => candidate.path === local.manifestPath,
  );
  return !entry || !(entry.bound ?? false);
}
