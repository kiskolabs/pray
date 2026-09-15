import { existsSync } from "node:fs";
import { resolve } from "node:path";
import type { Lockfile, ManagedSpanRecord } from "../lockfile/types.js";
import { readRegularBytes } from "../render/destination-io.js";
import { markerPositions } from "./markers.js";
import type { VerificationReport } from "./project.js";

export function inspectLockedDestinations(
  projectRoot: string,
  lockfile: Lockfile,
): VerificationReport {
  const report: VerificationReport = { findings: [] };
  const targetSpans = new Map<string, ManagedSpanRecord[]>();
  for (const span of lockfile.managed_span) {
    const spans = targetSpans.get(span.target) ?? [];
    spans.push(span);
    targetSpans.set(span.target, spans);
  }

  for (const [targetPath, spans] of targetSpans.entries()) {
    const absolutePath = resolve(projectRoot, targetPath);
    if (!existsSync(absolutePath)) {
      report.findings.push({
        kind: "verify_error",
        message: `Rendered file \`${targetPath}\` is missing. Run \`pray install\` to generate it.`,
      });
      continue;
    }
    const text = readRegularBytes(absolutePath, targetPath).toString("utf8");
    const markers = markerPositions(text.split("\n"));
    for (const span of spans) {
      const marker = markers.get(span.id);
      if (!marker) {
        report.findings.push({
          kind: "removed_prayer",
          message: `\`${targetPath}\` is missing managed marker \`${span.id}\` for \`${span.package}::${span.export}\`. Run \`pray install\` to restore the managed span.`,
        });
        continue;
      }
      if (marker.checksum !== span.ideal_checksum) {
        report.findings.push({
          kind: "custom_implementation",
          message: `\`${targetPath}\` marker \`${span.id}\` (\`${span.package}::${span.export}\`) was edited. Restore the managed block or run \`pray install\` to regenerate it.`,
        });
      }
    }
    const trackedIds = new Set(spans.map((span) => span.id));
    for (const markerId of markers.keys()) {
      if (markerId !== "0" && !trackedIds.has(markerId)) {
        report.findings.push({
          kind: "orphan_marker",
          message: `\`${targetPath}\` contains marker \`${markerId}\` that is not tracked in \`Prayfile.lock\`. Remove the marker or run \`pray install\` to reconcile.`,
        });
      }
    }
  }

  return report;
}
