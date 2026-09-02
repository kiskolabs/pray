import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { PrayError } from "../errors.js";
import { normalizeLineEndings } from "../hashing.js";
import type { Lockfile, ManagedSpanRecord } from "../lockfile/types.js";
import { renderProject } from "../render/project.js";
import { missingLocalEmbedGuidance } from "../resolve/project.js";
import type { ResolvedProject } from "../resolve/types.js";
import { markerPositions } from "./markers.js";
import {
  formatPositionDriftMessage,
  summarizePositionDrift,
} from "./position.js";
import { pushProvisionedFindings } from "./provisioned.js";

export interface VerificationFinding {
  kind: string;
  message: string;
}

export interface VerificationReport {
  findings: VerificationFinding[];
}

export function inspectProject(
  project: ResolvedProject,
  lockfile: Lockfile,
): VerificationReport {
  return collectVerificationReport(project, lockfile).report;
}

export function verifyProject(
  project: ResolvedProject,
  lockfile: Lockfile,
  strict = false,
): VerificationReport {
  const report = inspectProject(project, lockfile);
  if (report.findings.length === 0) {
    return report;
  }
  if (strict || report.findings.some((finding) => !isWarning(finding))) {
    throw PrayError.verify(formatVerificationReport(report));
  }
  return report;
}

export function driftProject(
  project: ResolvedProject,
  lockfile: Lockfile,
): VerificationReport {
  const { report, renderedTargets, freshTargets } = collectVerificationReport(
    project,
    lockfile,
  );
  const lockTargets = new Set(
    lockfile.target.flatMap((target) => target.outputs),
  );

  for (const [path, freshContent] of freshTargets.entries()) {
    const normalizedFresh = normalizeLineEndings(freshContent);
    const onDisk = renderedTargets.get(path);
    if (!onDisk || normalizeLineEndings(onDisk) !== normalizedFresh) {
      report.findings.push({
        kind: "renderer_drift",
        message: `${path} differs from fresh render`,
      });
    }
    if (!lockTargets.has(path)) {
      report.findings.push({
        kind: "renderer_drift",
        message: `${path} is not tracked in lockfile`,
      });
    }
  }

  if (report.findings.length === 0) {
    return report;
  }
  throw PrayError.verify(formatDriftReport(report));
}

export function formatVerificationReport(report: VerificationReport): string {
  return report.findings
    .map((finding) => `${finding.kind}: ${finding.message}`)
    .join("\n");
}

function formatDriftReport(report: VerificationReport): string {
  return formatVerificationReport(report);
}

function collectVerificationReport(
  project: ResolvedProject,
  lockfile: Lockfile,
): {
  report: VerificationReport;
  renderedTargets: Map<string, string>;
  freshTargets: Map<string, string>;
} {
  const report: VerificationReport = { findings: [] };
  const renderedTargets = new Map<string, string>();
  const freshTargets = new Map(
    renderProject(project).map((target) => [target.path, target.content]),
  );

  if (project.manifestHash !== lockfile.manifest_hash) {
    report.findings.push({
      kind: "verify_error",
      message:
        "Prayfile changed since `Prayfile.lock` was generated. Run `pray install` to refresh the lockfile.",
    });
  }

  const lockedPackages = new Map(
    lockfile.package.map((packageEntry) => [packageEntry.name, packageEntry]),
  );
  for (const packageEntry of project.packages) {
    const locked = lockedPackages.get(packageEntry.declaration.name);
    if (!locked) {
      report.findings.push({
        kind: "verify_error",
        message: `Package \`${packageEntry.declaration.name}\` is declared in Prayfile but missing from \`Prayfile.lock\`. Run \`pray install\` to update the lockfile.`,
      });
      continue;
    }
    lockedPackages.delete(packageEntry.declaration.name);
    if (locked.tree_hash !== packageEntry.treeHash) {
      report.findings.push({
        kind: "package_integrity",
        message: `Package \`${packageEntry.declaration.name}\` no longer matches the locked tree hash. Run \`pray install\` to re-resolve packages.`,
      });
    }
    if (locked.version !== packageEntry.spec.version) {
      report.findings.push({
        kind: "verify_error",
        message: `Package \`${packageEntry.declaration.name}\` resolved to version ${packageEntry.spec.version} but \`Prayfile.lock\` has ${locked.version}. Run \`pray install\` to refresh the lockfile.`,
      });
    }
  }
  for (const locked of lockedPackages.values()) {
    report.findings.push({
      kind: "verify_error",
      message: `Package \`${locked.name}\` is in \`Prayfile.lock\` but not declared in Prayfile. Remove it from the lockfile with \`pray install\` or add it back to Prayfile.`,
    });
  }

  const targetSpans = new Map<string, ManagedSpanRecord[]>();
  for (const span of lockfile.managed_span) {
    const spans = targetSpans.get(span.target) ?? [];
    spans.push(span);
    targetSpans.set(span.target, spans);
  }

  for (const [targetPath, spans] of targetSpans.entries()) {
    const absolutePath = resolve(project.projectRoot, targetPath);
    if (!existsSync(absolutePath)) {
      report.findings.push({
        kind: "verify_error",
        message: `Rendered file \`${targetPath}\` is missing. Run \`pray install\` to generate it.`,
      });
      continue;
    }
    const text = readFileSync(absolutePath, "utf8");
    renderedTargets.set(targetPath, text);
    const lines = text.split("\n");
    const markers = markerPositions(lines);
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
    const fresh = freshTargets.get(targetPath);
    const summary = summarizePositionDrift(
      targetPath,
      spans,
      markers,
      lines,
      fresh?.split("\n"),
      project.localFiles,
    );
    if (summary) {
      report.findings.push({
        kind: "position_drift",
        message: formatPositionDriftMessage(summary),
      });
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

  pushProvisionedFindings(project, report);

  for (const local of project.localFiles) {
    if (local.optional) {
      continue;
    }
    if (!existsSync(resolve(project.projectRoot, local.path))) {
      report.findings.push({
        kind: "verify_error",
        message: missingLocalEmbedGuidance(local.path),
      });
    }
  }

  return { report, renderedTargets, freshTargets };
}

function isWarning(finding: VerificationFinding): boolean {
  return finding.kind === "orphan_marker";
}
