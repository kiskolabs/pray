import type {
  LocalPosition,
  RenderChurn,
  RenderConflict,
  RenderMode,
  SourceKind,
} from "../domain/types.js";

export type {
  LocalPosition,
  PackageExportKind,
  RenderChurn,
  RenderConflict,
  RenderMode,
  SourceKind,
} from "../domain/types.js";

/** Named source declared in a Prayfile. */
export interface ManifestSource {
  /** Source name referenced by packages. */
  name: string;
  /** Source kind. */
  kind: SourceKind;
  /** Source URL. */
  url: string;
  /** Optional subdirectory within the source. */
  subdir?: string;
  /** Optional revision pin. */
  rev?: string;
  /** Optional tag pin. */
  tag?: string;
}

export type DestinationMode = "legacy" | "compose" | "tree";

export type ExportRole = "fragment" | "folder" | "file";

export type DestinationEntry =
  | { kind: "package"; name: string }
  | { kind: "local"; path: string };

/** Destination target declared in a Prayfile. */
export interface ManifestTarget {
  /** Target name. */
  name: string;
  /** Output paths to render. */
  outputs: string[];
  /** Skill export selectors. */
  skills: string[];
  /** Command export selectors. */
  commands: string[];
  /** Rule export selectors. */
  rules: string[];
  /** Maximum rendered byte size when set. */
  maxBytes?: number;
  /** Destination layout mode. */
  mode?: DestinationMode;
  /** Whether scoped compose is enabled. */
  scoped?: boolean;
  /** Explicit destination entry list when using compose entries. */
  entries?: DestinationEntry[];
  /** Whether to emit destination headers. */
  header?: boolean;
}

/** Package declaration in a Prayfile. */
export interface ManifestPackage {
  /** Package name. */
  name: string;
  /** Version constraint. */
  constraint: string;
  /** Named source binding. */
  source?: string;
  /** Selected export names. */
  exports: string[];
  /** Target names this package contributes to. */
  targets: string[];
  /** Feature flags. */
  features: string[];
  /** Group membership. */
  groups: string[];
  /** Whether the package may be missing. */
  optional: boolean;
  /** Local path source. */
  path?: string;
  /** Inline git URL. */
  git?: string;
  /** Git tag pin. */
  tag?: string;
  /** Git revision pin. */
  rev?: string;
  /** Tarball URL. */
  tarball?: string;
  /** OCI reference. */
  oci?: string;
  /** Single-file package path. */
  file?: string;
  /** Export role overrides. */
  roles?: ExportRole[];
  /** Whether the declaration is bound to a concrete locator. */
  bound?: boolean;
}

/** Local file embed declared in a Prayfile. */
export interface ManifestLocal {
  /** Path to the local file. */
  path: string;
  /** Placement position in compose destinations. */
  position: LocalPosition;
  /** Whether the file may be missing. */
  optional: boolean;
  /** Whether the declaration is bound. */
  bound?: boolean;
  /** Alternate file path when set. */
  file?: string;
}

/** Project-wide render policy from the Prayfile. */
export interface RenderPolicy {
  /** Render mode for managed destinations. */
  mode: RenderMode;
  /** Conflict handling policy. */
  conflict: RenderConflict;
  /** Churn policy for managed spans. */
  churn: RenderChurn;
  /** Whether destination headers are enabled by default. */
  header: boolean;
}

export interface ManifestPublishRemote {
  name: string;
  path?: string;
  url?: string;
  packages: string[];
}

/** Parsed Prayfile document. */
export interface Manifest {
  /** Prayfile language version. */
  prayfileVersion: string;
  /** Declared sources. */
  sources: ManifestSource[];
  /** Declared destination targets. */
  targets: ManifestTarget[];
  /** Declared packages. */
  packages: ManifestPackage[];
  /** Local file embeds. */
  local: ManifestLocal[];
  /** Named publish remotes. */
  publishRemotes?: ManifestPublishRemote[];
  /** Symbol substitutions. */
  symbols: Record<string, string>;
  /** Render policy. */
  render: RenderPolicy;
  /** Deprecated Prayfile keywords encountered while parsing (`target`, `output`, `agent`). */
  deprecatedKeywords?: string[];
}

export const defaultRenderPolicy = (): RenderPolicy => ({
  mode: "managed",
  conflict: "fail",
  churn: "minimal",
  header: true,
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
    publishRemotes: [...(manifest.publishRemotes ?? [])].sort((left, right) =>
      left.name.localeCompare(right.name),
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
      max_bytes: target.maxBytes ?? null,
      mode: target.mode ?? "legacy",
      scoped: target.scoped ?? false,
      entries: (target.entries ?? []).map((entry) =>
        entry.kind === "package"
          ? { kind: "package", name: entry.name }
          : { kind: "local", path: entry.path },
      ),
      ...(target.header !== undefined ? { header: target.header } : {}),
    })),
    packages: canonical.packages.map((packageEntry) => ({
      name: packageEntry.name,
      constraint: packageEntry.constraint,
      source: packageEntry.source ?? null,
      exports: packageEntry.exports,
      targets: packageEntry.targets,
      features: packageEntry.features,
      groups: packageEntry.groups,
      optional: packageEntry.optional,
      path: packageEntry.path ?? null,
      git: packageEntry.git ?? null,
      tag: packageEntry.tag ?? null,
      rev: packageEntry.rev ?? null,
      tarball: packageEntry.tarball ?? null,
      oci: packageEntry.oci ?? null,
      ...(packageEntry.file ? { file: packageEntry.file } : {}),
      roles: packageEntry.roles ?? [],
      bound: packageEntry.bound ?? false,
    })),
    local: canonical.local.map((entry) => ({
      path: entry.path,
      position: entry.position,
      optional: entry.optional,
      ...(entry.bound ? { bound: true } : {}),
      ...(entry.file ? { file: entry.file } : {}),
    })),
    ...(Object.keys(canonical.symbols ?? {}).length > 0
      ? { symbols: canonical.symbols }
      : {}),
    ...((canonical.publishRemotes ?? []).length > 0
      ? {
          publish_remotes: canonical.publishRemotes?.map((remote) => ({
            name: remote.name,
            ...(remote.path ? { path: remote.path } : {}),
            ...(remote.url ? { url: remote.url } : {}),
            packages: remote.packages,
          })),
        }
      : {}),
    render: {
      mode: canonical.render.mode,
      conflict: canonical.render.conflict,
      churn: canonical.render.churn,
      header: canonical.render.header,
    },
  };
}
