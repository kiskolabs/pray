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

export type DestinationMode = "legacy" | "compose" | "tree";

export type ExportRole = "fragment" | "folder" | "file";

export type DestinationEntry =
  | { kind: "package"; name: string }
  | { kind: "local"; path: string };

export interface ManifestTarget {
  name: string;
  outputs: string[];
  skills: string[];
  commands: string[];
  rules: string[];
  maxBytes?: number;
  mode?: DestinationMode;
  scoped?: boolean;
  entries?: DestinationEntry[];
}

export interface ManifestPackage {
  name: string;
  constraint: string;
  source?: string;
  exports: string[];
  targets: string[];
  features: string[];
  groups: string[];
  optional: boolean;
  path?: string;
  git?: string;
  tag?: string;
  rev?: string;
  tarball?: string;
  oci?: string;
  file?: string;
  roles?: ExportRole[];
  bound?: boolean;
}

export interface ManifestLocal {
  path: string;
  position: LocalPosition;
  optional: boolean;
  bound?: boolean;
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
  symbols: Record<string, string>;
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
    packages: [...manifest.packages].sort(
      (left, right) =>
        left.name.localeCompare(right.name) ||
        (left.source ?? "").localeCompare(right.source ?? "") ||
        left.constraint.localeCompare(right.constraint),
    ),
    local: [...manifest.local].sort((left, right) =>
      left.path.localeCompare(right.path),
    ),
    symbols: Object.fromEntries(
      Object.entries(manifest.symbols ?? {}).sort(([left], [right]) =>
        left.localeCompare(right),
      ),
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
      mode: target.mode ?? "legacy",
      scoped: target.scoped ?? false,
      entries: (target.entries ?? []).map((entry) =>
        entry.kind === "package"
          ? { kind: "package", name: entry.name }
          : { kind: "local", path: entry.path },
      ),
    })),
    packages: canonical.packages.map((packageEntry) => ({
      name: packageEntry.name,
      constraint: packageEntry.constraint,
      ...(packageEntry.source ? { source: packageEntry.source } : {}),
      exports: packageEntry.exports,
      targets: packageEntry.targets,
      features: packageEntry.features,
      groups: packageEntry.groups,
      optional: packageEntry.optional,
      ...(packageEntry.path ? { path: packageEntry.path } : {}),
      ...(packageEntry.git ? { git: packageEntry.git } : {}),
      ...(packageEntry.tag ? { tag: packageEntry.tag } : {}),
      ...(packageEntry.rev ? { rev: packageEntry.rev } : {}),
      ...(packageEntry.tarball ? { tarball: packageEntry.tarball } : {}),
      ...(packageEntry.oci ? { oci: packageEntry.oci } : {}),
      ...(packageEntry.file ? { file: packageEntry.file } : {}),
      roles: packageEntry.roles ?? [],
      bound: packageEntry.bound ?? false,
    })),
    local: canonical.local.map((entry) => ({
      path: entry.path,
      position: entry.position,
      optional: entry.optional,
      bound: entry.bound ?? false,
    })),
    ...(Object.keys(canonical.symbols ?? {}).length > 0
      ? { symbols: canonical.symbols }
      : {}),
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
