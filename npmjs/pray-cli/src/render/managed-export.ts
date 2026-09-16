import { PrayError } from "../errors.js";
import { checksumManagedSpanContent, markerId } from "../hashing.js";
import {
  LOCAL_EMBED_PACKAGE,
  type ManagedSpanRecord,
} from "../lockfile/types.js";
import type { ManifestTarget } from "../manifest/types.js";
import type { ResolvedLocalFile, ResolvedPackage } from "../resolve/types.js";
import { substitutePraySymbols } from "../substitute.js";
import type { ContentBuilder } from "./content-builder.js";

export function shouldInlineExport(
  packageEntry: ResolvedPackage,
  exportName: string,
): boolean {
  const exportEntry = packageEntry.spec.exports.get(exportName);
  return (
    !exportEntry ||
    exportEntry.kind === "fragment" ||
    exportEntry.kind === "file"
  );
}

export function appendManagedExport(
  builder: ContentBuilder,
  managedSpans: ManagedSpanRecord[],
  packageEntry: ResolvedPackage,
  exportName: string,
  target: ManifestTarget,
  output: string,
  symbols: Record<string, string>,
): void {
  const raw = packageEntry.exportBodies.get(exportName);
  if (raw === undefined) {
    const exportEntry = packageEntry.spec.exports.get(exportName);
    if (exportEntry?.kind === "file") {
      throw PrayError.integrity(
        `compose cannot write binary export ${exportName}; use file: for unmarked bytes`,
      );
    }
    throw PrayError.render(
      `package ${packageEntry.declaration.name} is missing cached export ${exportName}`,
    );
  }
  const body = substitutePraySymbols(raw, symbols);
  appendManagedSpan(
    builder,
    managedSpans,
    `${packageEntry.declaration.name}:${exportName}:${target.name}`,
    body,
    output,
    packageEntry.declaration.name,
    exportName,
    packageEntry.sourceChecksum,
  );
}

export function appendManagedLocal(
  builder: ContentBuilder,
  managedSpans: ManagedSpanRecord[],
  local: ResolvedLocalFile,
  target: ManifestTarget,
  output: string,
  symbols: Record<string, string>,
): void {
  if (local.content.length === 0 && local.optional) {
    return;
  }
  const body = substitutePraySymbols(local.content, symbols);
  appendManagedSpan(
    builder,
    managedSpans,
    `local:${local.manifestPath}:${target.name}`,
    body,
    output,
    LOCAL_EMBED_PACKAGE,
    local.manifestPath,
    local.sourceChecksum,
  );
}

function appendManagedSpan(
  builder: ContentBuilder,
  managedSpans: ManagedSpanRecord[],
  seed: string,
  body: string,
  output: string,
  packageName: string,
  exportName: string,
  sourceChecksum: string,
): void {
  const id = markerId(seed);
  const openLine = builder.nextLineNumber();
  builder.appendLine(`<!-- pray:${id} -->`);
  builder.appendBody(body);
  const closeLine = builder.nextLineNumber();
  builder.appendLine(`<!-- pray:${id} -->`);
  managedSpans.push({
    id,
    target: output,
    open_line: openLine,
    close_line: closeLine,
    ideal_checksum: checksumManagedSpanContent(body),
    package: packageName,
    export: exportName,
    source_checksum: sourceChecksum,
    silenced: false,
  });
  builder.appendEmptyLine();
}
