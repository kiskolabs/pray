import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { PrayError } from "../errors.js";
import {
  checksumManagedBodyLineRefs,
  normalizeLineEndings,
} from "../hashing.js";
import type { Lockfile, ManagedSpanRecord } from "../lockfile/types.js";
import { missingLocalEmbedGuidance } from "../resolve/project.js";
import type { ResolvedProject } from "../resolve/types.js";
import { renderProject } from "../render/project.js";

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
  const { report, renderedTargets } = collectVerificationReport(
    project,
    lockfile,
  );
  const rendered = renderProject(project);
  const lockTargets = new Set(
    lockfile.target.flatMap((target) => target.outputs),
  );

  for (const target of rendered) {
    const normalizedFresh = normalizeLineEndings(target.content);
    const onDisk = renderedTargets.get(target.path);
    if (!onDisk || normalizeLineEndings(onDisk) !== normalizedFresh) {
      report.findings.push({
        kind: "renderer_drift",
        message: `${target.path} differs from fresh render`,
      });
    }
    if (!lockTargets.has(target.path)) {
      report.findings.push({
        kind: "renderer_drift",
        message: `${target.path} is not tracked in lockfile`,
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
): { report: VerificationReport; renderedTargets: Map<string, string> } {
  const report: VerificationReport = { findings: [] };
  const renderedTargets = new Map<string, string>();

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
      if (
        marker.openLine !== span.open_line ||
        marker.closeLine !== span.close_line
      ) {
        report.findings.push({
          kind: "position_drift",
          message: `\`${targetPath}\` marker \`${span.id}\` (\`${span.package}::${span.export}\`) moved to different lines. Run \`pray install\` to restore expected positions.`,
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

  return { report, renderedTargets };
}

function isWarning(finding: VerificationFinding): boolean {
  return finding.kind === "orphan_marker";
}

function markerPositions(
  lines: string[],
): Map<string, { openLine: number; closeLine: number; checksum: string }> {
  const markers = new Map<
    string,
    { openLine: number; closeLine: number; checksum: string }
  >();
  let active:
    | { id: string; openLine: number; body: string[] }
    | undefined;

  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index]!;
    const parsed = parseMarker(line);
    if (!parsed) {
      active?.body.push(line);
      continue;
    }
    if (parsed === "ignore") {
      continue;
    }
    if (!active) {
      active = { id: parsed, openLine: index + 1, body: [] };
      continue;
    }
    if (active.id === parsed) {
      markers.set(active.id, {
        openLine: active.openLine,
        closeLine: index + 1,
        checksum: checksumManagedBodyLineRefs(active.body),
      });
      active = undefined;
    }
  }

  return markers;
}

function parseMarker(line: string): string | "ignore" | undefined {
  const trimmed = line.trim();
  if (!trimmed.startsWith("<!-- pray:") || !trimmed.endsWith(" -->")) {
    return undefined;
  }
  const id = trimmed.slice("<!-- pray:".length, -" -->".length);
  if (id === "0 ignore-comments") {
    return "ignore";
  }
  if (/^[a-z0-9]+$/.test(id)) {
    return id;
  }
  return undefined;
}
