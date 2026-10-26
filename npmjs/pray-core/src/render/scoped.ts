import { packageMatchesEnvironment } from "../environment.js";
import { targetEntries } from "../manifest/destination.js";
import type { ManifestTarget } from "../manifest/types.js";
import type { ResolvedProject } from "../resolve/types.js";
import { ContentBuilder } from "./content-builder.js";
import { appendHeaderIfEnabled } from "./header.js";
import {
  appendManagedExport,
  appendManagedLocal,
  shouldInlineExport,
} from "./managed-export.js";
import type { RenderedTarget } from "./types.js";

export function renderScopedCompose(
  project: ResolvedProject,
  target: ManifestTarget,
  output: string,
): RenderedTarget {
  const builder = new ContentBuilder();
  appendHeaderIfEnabled(builder, project, target, output);

  const managedSpans: RenderedTarget["managedSpans"] = [];
  const localsByPath = new Map(
    project.localFiles.map((local) => [local.manifestPath, local]),
  );
  const packagesByName = new Map(
    project.packages.map((packageEntry) => [
      packageEntry.declaration.name,
      packageEntry,
    ]),
  );
  for (const entry of targetEntries(target)) {
    if (entry.kind === "local") {
      const local = localsByPath.get(entry.path);
      if (!local) continue;
      appendLocalEntry(builder, managedSpans, project, target, output, local);
      continue;
    }
    const packageEntry = packagesByName.get(entry.name);
    if (!packageEntry) continue;
    appendPackageEntry(
      builder,
      managedSpans,
      project,
      target,
      output,
      packageEntry,
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
  managedSpans: RenderedTarget["managedSpans"],
  project: ResolvedProject,
  target: ManifestTarget,
  output: string,
  local: ResolvedProject["localFiles"][number],
): void {
  appendManagedLocal(
    builder,
    managedSpans,
    local,
    target,
    output,
    project.manifest.symbols ?? {},
  );
}

function appendPackageEntry(
  builder: ContentBuilder,
  managedSpans: RenderedTarget["managedSpans"],
  project: ResolvedProject,
  target: ManifestTarget,
  output: string,
  packageEntry: ResolvedProject["packages"][number],
): void {
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
      project.manifest.symbols ?? {},
    );
  }
}
