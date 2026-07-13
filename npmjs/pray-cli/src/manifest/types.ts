import type {
  LineEndings,
  LocalPosition,
  RenderChurn,
  RenderConflict,
  RenderMode,
  SourceKind,
} from "../domain/types.js";

export type {
  LineEndings,
  LocalPosition,
  PackageExportKind,
  RenderChurn,
  RenderConflict,
  RenderMode,
  SourceKind,
} from "../domain/types.js";

export interface ManifestSource {
  name: string;
  kind: SourceKind;
  url: string;
  subdir?: string;
  rev?: string;
  tag?: string;
}

export interface ManifestTarget {
  name: string;
  outputs: string[];
  skills: string[];
  commands: string[];
  rules: string[];
  maxBytes?: number;
}

export interface ManifestPackage {
  name: string;
  constraint: string;
  source?: string;
  exports: string[];
  targets: string[];
  features: string[];
  optional: boolean;
  path?: string;
  git?: string;
  tag?: string;
  rev?: string;
  tarball?: string;
  oci?: string;
}

export interface ManifestLocal {
  path: string;
  position: LocalPosition;
  optional: boolean;
}

export interface RenderPolicy {
  mode: RenderMode;
  conflict: RenderConflict;
  churn: RenderChurn;
  header: boolean;
  sectionMarkers: boolean;
  lineEndings: LineEndings;
}

export interface Manifest {
  prayfileVersion: string;
  sources: ManifestSource[];
  targets: ManifestTarget[];
  packages: ManifestPackage[];
  local: ManifestLocal[];
  render: RenderPolicy;
}

export const defaultRenderPolicy = (): RenderPolicy => ({
  mode: "managed",
  conflict: "fail",
  churn: "minimal",
  header: true,
  sectionMarkers: true,
  lineEndings: "lf",
});

export function canonicalManifest(manifest: Manifest): Manifest {
  return {
    ...manifest,
    sources: [...manifest.sources].sort((left, right) =>
      left.name.localeCompare(right.name),
    ),
    targets: [...manifest.targets].sort((left, right) =>
      left.name.localeCompare(right.name),
    ),
    packages: [...manifest.packages].sort((left, right) =>
      left.name.localeCompare(right.name) ||
      (left.source ?? "").localeCompare(right.source ?? "") ||
      left.constraint.localeCompare(right.constraint),
    ),
    local: [...manifest.local].sort((left, right) =>
      left.path.localeCompare(right.path),
    ),
  };
}

export function manifestToJson(manifest: Manifest): Record<string, unknown> {
  const canonical = canonicalManifest(manifest);
  return {
    prayfile_version: canonical.prayfileVersion,
    sources: canonical.sources.map((source) => ({
      name: source.name,
      kind: source.kind,
      url: source.url,
      ...(source.subdir ? { subdir: source.subdir } : {}),
      ...(source.rev ? { rev: source.rev } : {}),
      ...(source.tag ? { tag: source.tag } : {}),
    })),
    targets: canonical.targets.map((target) => ({
      name: target.name,
      outputs: target.outputs,
      skills: target.skills,
      commands: target.commands,
      rules: target.rules,
      ...(target.maxBytes !== undefined ? { max_bytes: target.maxBytes } : {}),
    })),
    packages: canonical.packages.map((packageEntry) => ({
      name: packageEntry.name,
      constraint: packageEntry.constraint,
      ...(packageEntry.source ? { source: packageEntry.source } : {}),
      exports: packageEntry.exports,
      targets: packageEntry.targets,
      features: packageEntry.features,
      optional: packageEntry.optional,
      ...(packageEntry.path ? { path: packageEntry.path } : {}),
      ...(packageEntry.git ? { git: packageEntry.git } : {}),
      ...(packageEntry.tag ? { tag: packageEntry.tag } : {}),
      ...(packageEntry.rev ? { rev: packageEntry.rev } : {}),
      ...(packageEntry.tarball ? { tarball: packageEntry.tarball } : {}),
      ...(packageEntry.oci ? { oci: packageEntry.oci } : {}),
    })),
    local: canonical.local.map((entry) => ({
      path: entry.path,
      position: entry.position,
      optional: entry.optional,
    })),
    render: {
      mode: canonical.render.mode,
      conflict: canonical.render.conflict,
      churn: canonical.render.churn,
      header: canonical.render.header,
      section_markers: canonical.render.sectionMarkers,
      line_endings: canonical.render.lineEndings,
    },
  };
}
