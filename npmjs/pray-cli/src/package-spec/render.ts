import type { LiteralValue } from "../literal/types.js";
import type {
  PackageExport,
  PackageSkill,
  PackageSpec,
  PackageTemplate,
} from "./types.js";

export function renderPackageSpec(spec: PackageSpec): string {
  const lines = ["Package::Specification.new do |spec|"];
  pushAssignment(lines, "name", spec.name);
  if (spec.version.length > 0) {
    pushAssignment(lines, "version", spec.version);
  }
  pushOptional(lines, "summary", spec.summary);
  pushOptional(lines, "description", spec.description);
  if (spec.authors.length > 0) {
    pushArray(lines, "authors", spec.authors);
  }
  if (spec.maintainers.length > 0) {
    pushArray(lines, "maintainers", spec.maintainers);
  }
  pushOptional(lines, "license", spec.license);
  pushOptional(lines, "homepage", spec.homepage);
  pushOptional(lines, "source_code_uri", spec.sourceCodeUri);
  pushOptional(lines, "changelog_uri", spec.changelogUri);
  pushOptional(lines, "prayfile_version", spec.prayfileVersion);
  pushArray(lines, "files", spec.files);
  pushExports(lines, spec.exports);
  pushNamedPaths(lines, "skills", spec.skills);
  pushNamedPaths(lines, "templates", spec.templates);
  pushStringMap(lines, "adapters", spec.adapters);
  if (spec.targets.length > 0) {
    pushArray(lines, "targets", spec.targets);
  }
  for (const dependency of spec.dependencies) {
    const method = dependency.optional
      ? "add_optional_dependency"
      : "add_dependency";
    lines.push(
      `  spec.${method} ${quote(dependency.name)}, ${quote(dependency.constraint)}`,
    );
  }
  if (spec.metadata.size > 0) {
    lines.push(`  spec.metadata = ${literalMap(spec.metadata)}`);
  }
  if (spec.upstream) {
    lines.push(
      `  spec.upstream ${quote(spec.upstream.name)}, ${quote(spec.upstream.constraint)}`,
    );
  }
  lines.push("end", "");
  return lines.join("\n");
}

function pushAssignment(lines: string[], field: string, value: string): void {
  lines.push(`  spec.${field} = ${quote(value)}`);
}

function pushOptional(
  lines: string[],
  field: string,
  value: string | undefined,
): void {
  if (value !== undefined) {
    pushAssignment(lines, field, value);
  }
}

function pushArray(
  lines: string[],
  field: string,
  values: readonly string[],
): void {
  lines.push(`  spec.${field} = [${stringArray(values)}]`);
}

function pushExports(
  lines: string[],
  exports: Map<string, PackageExport>,
): void {
  if (exports.size === 0) {
    return;
  }
  lines.push("  spec.exports = {");
  for (const [name, exportEntry] of exports) {
    const fields = [
      `type: ${quote(exportEntry.kind)}`,
      `path: ${quote(exportEntry.path)}`,
    ];
    if (exportEntry.summary !== undefined) {
      fields.push(`summary: ${quote(exportEntry.summary)}`);
    }
    if (exportEntry.only && exportEntry.only.length > 0) {
      fields.push(`only: [${stringArray(exportEntry.only)}]`);
    }
    if (exportEntry.except && exportEntry.except.length > 0) {
      fields.push(`except: [${stringArray(exportEntry.except)}]`);
    }
    if (exportEntry.defaultPath !== undefined) {
      fields.push(`default_path: ${quote(exportEntry.defaultPath)}`);
    }
    lines.push(`    ${quote(name)} => { ${fields.join(", ")} },`);
  }
  lines.push("  }");
}

function pushNamedPaths(
  lines: string[],
  field: string,
  entries: Map<string, PackageSkill | PackageTemplate>,
): void {
  if (entries.size === 0) {
    return;
  }
  lines.push(`  spec.${field} = {`);
  for (const [name, entry] of entries) {
    const summary =
      entry.summary === undefined ? "" : `, summary: ${quote(entry.summary)}`;
    lines.push(
      `    ${quote(name)} => { path: ${quote(entry.path)}${summary} },`,
    );
  }
  lines.push("  }");
}

function pushStringMap(
  lines: string[],
  field: string,
  entries: Map<string, string>,
): void {
  if (entries.size === 0) {
    return;
  }
  const values = [...entries.entries()]
    .map(([name, value]) => `${quote(name)} => ${quote(value)}`)
    .join(", ");
  lines.push(`  spec.${field} = { ${values} }`);
}

function stringArray(values: readonly string[]): string {
  return values.map((value) => quote(value)).join(", ");
}

function quote(value: string): string {
  const escaped = value
    .replaceAll("\\", "\\\\")
    .replaceAll('"', '\\"')
    .replaceAll("\n", "\\n")
    .replaceAll("\r", "\\r")
    .replaceAll("\t", "\\t");
  return `"${escaped}"`;
}

function literalMap(entries: Map<string, LiteralValue>): string {
  const values = [...entries.entries()]
    .map(([name, value]) => `${quote(name)} => ${literal(value)}`)
    .join(", ");
  return `{ ${values} }`;
}

function literal(value: LiteralValue): string {
  switch (value.kind) {
    case "string":
      return quote(value.value);
    case "symbol":
      return `:${value.value}`;
    case "bool":
      return value.value.toString();
    case "null":
      return "nil";
    case "integer":
      return value.value.toString();
    case "array":
      return `[${value.value.map((entry) => literal(entry)).join(", ")}]`;
    case "map":
      return literalMap(value.value);
  }
}
