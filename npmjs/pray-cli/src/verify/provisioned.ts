import { existsSync, lstatSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { sha256Prefixed } from "../hashing.js";
import {
  expectedProvisionedBytes,
  plannedProvisionedFiles,
} from "../render/provisioned.js";
import type { ResolvedProject } from "../resolve/types.js";

interface VerificationReport {
  findings: Array<{ kind: string; message: string }>;
}

export function pushProvisionedFindings(
  project: ResolvedProject,
  report: VerificationReport,
): void {
  pushExclusiveFileExportFindings(project, report);
  for (const file of plannedProvisionedFiles(project)) {
    const pathText = file.path.replaceAll("\\", "/");
    const absolute = resolve(project.projectRoot, file.path);
    try {
      if (lstatSync(absolute).isSymbolicLink()) {
        report.findings.push({
          kind: "verify_error",
          message: `Provisioned file \`${pathText}\` is a symbolic link. Remove the link or choose another destination.`,
        });
        continue;
      }
    } catch (error) {
      if (!isNotFound(error)) {
        throw error;
      }
      report.findings.push({
        kind: "verify_error",
        message: `Provisioned file \`${pathText}\` from \`${file.package}\` is missing. Run \`pray install\` to materialize it.`,
      });
      continue;
    }
    if (!existsSync(absolute)) {
      report.findings.push({
        kind: "verify_error",
        message: `Provisioned file \`${pathText}\` from \`${file.package}\` is missing. Run \`pray install\` to materialize it.`,
      });
      continue;
    }
    const destinationBytes = readFileSync(absolute);
    const expectedBytes = expectedProvisionedBytes(
      file.source,
      project.manifest.symbols ?? {},
    );
    if (sha256Prefixed(destinationBytes) !== sha256Prefixed(expectedBytes)) {
      report.findings.push({
        kind: "package_integrity",
        message: `Provisioned file \`${pathText}\` no longer matches package \`${file.package}\`. Run \`pray install\` to restore it.`,
      });
    }
  }
}

function pushExclusiveFileExportFindings(
  project: ResolvedProject,
  report: VerificationReport,
): void {
  for (const packageEntry of project.packages) {
    const destination = packageEntry.declaration.file;
    if (!destination) {
      continue;
    }
    const hasFileExport = packageEntry.selectedExports.some((name) => {
      return packageEntry.spec.exports.get(name)?.kind === "file";
    });
    if (!hasFileExport) {
      report.findings.push({
        kind: "verify_error",
        message: `Package \`${packageEntry.declaration.name}\` declares file: "${destination}" but has no selected file export.`,
      });
    }
  }
}

function isNotFound(error: unknown): boolean {
  return (
    typeof error === "object" &&
    error !== null &&
    "code" in error &&
    (error as { code: string }).code === "ENOENT"
  );
}
