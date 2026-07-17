import type {
  LockedPackage,
  LockedTarget,
  Lockfile,
  LockSource,
  ManagedSpanRecord,
} from "./types.js";
import { canonicalLockfile } from "./types.js";

function formatString(value: string): string {
  return `"${value.replace(/\\/g, "\\\\").replace(/"/g, '\\"')}"`;
}

function formatStringArray(values: string[]): string {
  if (values.length === 0) {
    return "[]";
  }
  if (values.length === 1) {
    return `[${formatString(values[0]!)}]`;
  }
  const items = values.map((value) => `    ${formatString(value)}`).join(",\n");
  return `[\n${items},\n]`;
}

function appendScalars(
  lines: string[],
  entries: Array<[string, string | number | boolean]>,
): void {
  for (const [key, value] of entries) {
    if (typeof value === "string") {
      lines.push(`${key} = ${formatString(value)}`);
      continue;
    }
    if (typeof value === "number") {
      lines.push(`${key} = ${value}`);
      continue;
    }
    lines.push(`${key} = ${value ? "true" : "false"}`);
  }
}

function formatSource(source: LockSource): string[] {
  const lines = ["[[source]]"];
  const entries: Array<[string, string | number | boolean]> = [
    ["name", source.name],
    ["kind", source.kind],
    ["url", source.url],
  ];
  if (source.revision !== undefined) {
    entries.push(["revision", source.revision]);
  }
  if (source.host_key_fingerprint !== undefined) {
    entries.push(["host_key_fingerprint", source.host_key_fingerprint]);
  }
  appendScalars(lines, entries);
  return lines;
}

function formatPackage(packageEntry: LockedPackage): string[] {
  const lines = ["[[package]]"];
  const entries: Array<[string, string | number | boolean]> = [
    ["name", packageEntry.name],
    ["version", packageEntry.version],
  ];
  if (packageEntry.source !== undefined) {
    entries.push(["source", packageEntry.source]);
  }
  entries.push(
    ["path", packageEntry.path],
    ["tree_hash", packageEntry.tree_hash],
    ["artifact_hash", packageEntry.artifact_hash],
    ["artifact", packageEntry.artifact],
  );
  appendScalars(lines, entries);
  lines.push(`exports = ${formatStringArray(packageEntry.exports)}`);
  lines.push(`dependencies = ${formatStringArray(packageEntry.dependencies)}`);
  if (packageEntry.signer_fingerprint !== undefined) {
    lines.push(
      `signer_fingerprint = ${formatString(packageEntry.signer_fingerprint)}`,
    );
  }
  return lines;
}

function formatTarget(target: LockedTarget): string[] {
  const lines = ["[[target]]"];
  appendScalars(lines, [["name", target.name]]);
  lines.push(`outputs = ${formatStringArray(target.outputs)}`);
  return lines;
}

function formatManagedSpan(span: ManagedSpanRecord): string[] {
  const lines = ["[[managed_span]]"];
  appendScalars(lines, [
    ["id", span.id],
    ["target", span.target],
    ["open_line", span.open_line],
    ["close_line", span.close_line],
    ["ideal_checksum", span.ideal_checksum],
    ["package", span.package],
    ["export", span.export],
    ["source_checksum", span.source_checksum],
    ["silenced", span.silenced],
  ]);
  return lines;
}

function appendSection(lines: string[], sectionLines: string[]): void {
  if (sectionLines.length === 0) {
    return;
  }
  lines.push(...sectionLines, "");
}

export function serializeLockfileText(lockfile: Lockfile): string {
  const canonical = canonicalLockfile(lockfile);
  const lines = [
    `prayfile_lock = ${formatString(canonical.prayfile_lock)}`,
    `spec = ${formatString(canonical.spec)}`,
    `generated_by = ${formatString(canonical.generated_by)}`,
    `manifest_hash = ${formatString(canonical.manifest_hash)}`,
  ];
  if (canonical.environment !== undefined) {
    lines.push(`environment = ${formatString(canonical.environment)}`);
  }
  lines.push("");

  for (const source of canonical.source) {
    appendSection(lines, formatSource(source));
  }
  for (const packageEntry of canonical.package) {
    appendSection(lines, formatPackage(packageEntry));
  }
  for (const target of canonical.target) {
    appendSection(lines, formatTarget(target));
  }
  for (const span of canonical.managed_span) {
    appendSection(lines, formatManagedSpan(span));
  }

  while (lines.length > 0 && lines[lines.length - 1] === "") {
    lines.pop();
  }
  return `${lines.join("\n")}\n`;
}
