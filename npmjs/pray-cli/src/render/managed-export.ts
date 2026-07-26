import { PrayError } from "../errors.js";
import { checksumManagedSpanContent, markerId } from "../hashing.js";
import type { ManagedSpanRecord } from "../lockfile/types.js";
import type { ManifestTarget } from "../manifest/types.js";
import type { ResolvedPackage } from "../resolve/types.js";
import type { ContentBuilder } from "./content-builder.js";

export function shouldInlineExport(
  packageEntry: ResolvedPackage,
  exportName: string,
): boolean {
  const exportEntry = packageEntry.spec.exports.get(exportName);
  return !exportEntry || exportEntry.kind === "fragment";
}

export function appendManagedExport(
  builder: ContentBuilder,
  managedSpans: ManagedSpanRecord[],
  packageEntry: ResolvedPackage,
  exportName: string,
  target: ManifestTarget,
  output: string,
): void {
  const body = packageEntry.exportBodies.get(exportName);
  if (!body) {
    throw PrayError.render(
      `package ${packageEntry.declaration.name} is missing cached export ${exportName}`,
    );
  }
  const id = markerId(
    `${packageEntry.declaration.name}:${exportName}:${target.name}`,
  );
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
    package: packageEntry.declaration.name,
    export: exportName,
    source_checksum: packageEntry.sourceChecksum,
    silenced: false,
  });
  builder.appendEmptyLine();
}
