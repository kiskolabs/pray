import type { PackageExportKind } from "../domain/types.js";
import type { LiteralValue } from "../literal/types.js";

export interface PackageExport {
  kind: PackageExportKind;
  path: string;
  summary?: string;
  only?: string[];
  except?: string[];
  defaultPath?: string;
}

export interface PackageSkill {
  path: string;
  summary?: string;
}

export interface PackageTemplate {
  path: string;
  summary?: string;
}

export interface PackageDependency {
  name: string;
  constraint: string;
  optional: boolean;
}

export interface PackageUpstream {
  name: string;
  constraint: string;
}

export interface PackageSpec {
  name: string;
  version: string;
  summary?: string;
  description?: string;
  authors: string[];
  maintainers: string[];
  license?: string;
  homepage?: string;
  sourceCodeUri?: string;
  changelogUri?: string;
  prayfileVersion?: string;
  files: string[];
  exports: Map<string, PackageExport>;
  skills: Map<string, PackageSkill>;
  templates: Map<string, PackageTemplate>;
  adapters: Map<string, string>;
  targets: string[];
  dependencies: PackageDependency[];
  metadata: Map<string, LiteralValue>;
  upstream?: PackageUpstream;
}

export function canonicalPackageSpec(spec: PackageSpec): PackageSpec {
  return {
    ...spec,
    files: [...spec.files].sort(),
    authors: [...spec.authors].sort(),
    maintainers: [...(spec.maintainers ?? [])].sort(),
    targets: [...spec.targets].sort(),
    dependencies: [...spec.dependencies].sort(
      (left, right) =>
        left.name.localeCompare(right.name) ||
        left.constraint.localeCompare(right.constraint) ||
        Number(left.optional) - Number(right.optional),
    ),
  };
}
