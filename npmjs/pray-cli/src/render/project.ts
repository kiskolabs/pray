import { copyFileSync, existsSync, mkdirSync, writeFileSync } from "node:fs";
import { basename, resolve } from "node:path";
import { packageMatchesEnvironment } from "../environment.js";
import { PrayError } from "../errors.js";
import {
  checksumManagedSpanContent,
  markerId,
  normalizeLineEndings,
} from "../hashing.js";
import type { ManagedSpanRecord } from "../lockfile/types.js";
import type { ManifestTarget } from "../manifest/types.js";
import type { ResolvedPackage, ResolvedProject } from "../resolve/types.js";
import type { RenderedTarget } from "./types.js";

export function renderProject(project: ResolvedProject): RenderedTarget[] {
  return project.manifest.targets.map((target) => {
    const output = target.outputs[0];
    if (!output) {
      throw PrayError.render(`target ${target.name} has no output file`);
    }
    return renderTarget(project, target, output);
  });
}

export function writeRenderedTargets(
  project: ResolvedProject,
  rendered: RenderedTarget[],
): void {
  for (const target of rendered) {
    const path = resolve(project.projectRoot, target.path);
    mkdirSync(resolve(path, ".."), { recursive: true });
    writeFileSync(path, target.content, "utf8");
  }
  materializeProvisionedExports(project);
}

function renderTarget(
  project: ResolvedProject,
  target: ManifestTarget,
  output: string,
): RenderedTarget {
  const builder = new ContentBuilder();
  if (project.manifest.render.header) {
    const outputName = basename(output);
    builder.appendLine("<!-- pray:0 ignore-comments -->");
    builder.appendEmptyLine();
    builder.appendLine("# Agent context");
    builder.appendEmptyLine();
    builder.appendLine(
      `Do not edit managed blocks in \`${outputName}\` or provisioned files under \`.agents/\`.`,
    );
    builder.appendLine(
      "To change shared guidance, update `Prayfile` and run `pray install`.",
    );
    builder.appendEmptyLine();
  }

  if (project.localFiles.length > 0) {
    builder.appendLine("## Additional instructions");
    builder.appendEmptyLine();
  }
  for (const local of project.localFiles) {
    if (local.content.length === 0 && local.optional) {
      continue;
    }
    builder.appendLine(`### ${local.manifestPath}`);
    builder.appendBody(local.content);
    builder.appendEmptyLine();
  }

  builder.appendLine("## Shared instructions");
  builder.appendEmptyLine();

  const managedSpans: ManagedSpanRecord[] = [];
  for (const packageEntry of project.packages) {
    if (
      !packageMatchesEnvironment(
        packageEntry.declaration.groups,
        project.environment,
      )
    ) {
      continue;
    }
    for (const exportName of packageEntry.selectedExports) {
      if (!shouldInlineExport(packageEntry, exportName)) {
        continue;
      }
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
  }

  return {
    path: output,
    content: builder.finish(),
    managedSpans,
  };
}

function shouldInlineExport(
  packageEntry: ResolvedPackage,
  exportName: string,
): boolean {
  const exportEntry = packageEntry.spec.exports.get(exportName);
  return !exportEntry || exportEntry.kind === "fragment";
}

function materializeProvisionedExports(project: ResolvedProject): void {
  for (const file of plannedProvisionedFiles(project)) {
    const destination = resolve(project.projectRoot, file.path);
    mkdirSync(resolve(destination, ".."), { recursive: true });
    copyFileSync(file.source, destination);
  }
}

interface PlannedProvisionedFile {
  path: string;
  source: string;
}

function plannedProvisionedFiles(
  project: ResolvedProject,
): PlannedProvisionedFile[] {
  const planned: PlannedProvisionedFile[] = [];
  for (const target of project.manifest.targets) {
    for (const folderRoot of target.skills) {
      const destinationRoot = resolve(project.projectRoot, folderRoot);
      for (const packageEntry of project.packages) {
        if (
          !packageMatchesEnvironment(
            packageEntry.declaration.groups,
            project.environment,
          )
        ) {
          continue;
        }
        collectSelectedExportFiles(
          project,
          packageEntry,
          destinationRoot,
          planned,
        );
      }
    }
  }
  return planned.sort((left, right) => left.path.localeCompare(right.path));
}

function collectSelectedExportFiles(
  project: ResolvedProject,
  packageEntry: ResolvedPackage,
  destinationRoot: string,
  planned: PlannedProvisionedFile[],
): void {
  for (const exportName of packageEntry.selectedExports) {
    const exportEntry = packageEntry.spec.exports.get(exportName);
    if (!exportEntry) {
      continue;
    }
    if (exportEntry.kind === "folder" || exportEntry.kind === "skill") {
      const indexedFiles = packageEntry.skillFiles.get(exportName);
      if (!indexedFiles) {
        throw PrayError.render(
          `package ${packageEntry.declaration.name} has no indexed files for folder export ${exportName}`,
        );
      }
      const destinationName = folderDestinationName(
        exportName,
        exportEntry.path,
      );
      collectTreeFiles(
        project,
        resolve(packageEntry.root, exportEntry.path),
        resolve(destinationRoot, destinationName),
        indexedFiles,
        planned,
      );
    }
  }
}

function folderDestinationName(exportName: string, exportPath: string): string {
  const trimmed = exportPath.replace(/\/$/, "");
  const name = basename(trimmed);
  return name.length > 0 ? name : exportName;
}

function collectTreeFiles(
  project: ResolvedProject,
  sourceRoot: string,
  destinationRoot: string,
  relativeFiles: string[],
  planned: PlannedProvisionedFile[],
): void {
  if (!existsSync(sourceRoot)) {
    throw PrayError.render(`folder source directory missing: ${sourceRoot}`);
  }
  if (relativeFiles.length === 0) {
    throw PrayError.render(
      `no files listed in package manifest for ${sourceRoot}`,
    );
  }
  for (const relative of relativeFiles) {
    const source = resolve(sourceRoot, relative);
    if (!existsSync(source)) {
      throw PrayError.render(`provisioned file missing: ${source}`);
    }
    const destination = resolve(destinationRoot, relative);
    planned.push({
      path: relativeProjectPath(project.projectRoot, destination),
      source,
    });
  }
}

function relativeProjectPath(
  projectRoot: string,
  absolutePath: string,
): string {
  return absolutePath
    .slice(projectRoot.length)
    .replace(/^[/\\]/, "")
    .replace(/\\/g, "/");
}

class ContentBuilder {
  private content = "";

  nextLineNumber(): number {
    return this.content.split("\n").length;
  }

  appendLine(line: string): void {
    this.content += `${line}\n`;
  }

  appendEmptyLine(): void {
    this.content += "\n";
  }

  appendBody(body: string): void {
    const trimmed = body.replace(/\n+$/, "");
    if (trimmed.length === 0) {
      return;
    }
    for (const line of trimmed.split("\n")) {
      this.appendLine(line);
    }
  }

  finish(): string {
    while (this.content.endsWith("\n\n")) {
      this.content = this.content.slice(0, -1);
    }
    if (!this.content.endsWith("\n")) {
      this.content += "\n";
    }
    return this.content;
  }
}

export { normalizeLineEndings };
