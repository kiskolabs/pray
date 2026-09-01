import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { basename, resolve } from "node:path";
import { packageMatchesEnvironment } from "../environment.js";
import { PrayError } from "../errors.js";
import { packageBoundToTree } from "../manifest/destination.js";
import { validateProjectRelativePath } from "../manifest/validate.js";
import type { ResolvedPackage, ResolvedProject } from "../resolve/types.js";
import { substitutePraySymbols } from "../substitute.js";

export interface PlannedProvisionedFile {
  path: string;
  source: string;
}

export function plannedProvisionedFiles(
  project: ResolvedProject,
): PlannedProvisionedFile[] {
  const planned: PlannedProvisionedFile[] = [];
  collectExactFileBindings(project, planned);
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
        if (!packageBoundToTree(packageEntry.declaration, target)) {
          continue;
        }
        collectLegacySkillFiles(
          project,
          packageEntry,
          destinationRoot,
          planned,
        );
        collectSelectedExportFiles(
          project,
          packageEntry,
          destinationRoot,
          planned,
        );
      }
    }
  }
  const sorted = planned.sort((left, right) =>
    left.path.localeCompare(right.path),
  );
  return dedupeByPath(sorted);
}

function dedupeByPath(
  files: PlannedProvisionedFile[],
): PlannedProvisionedFile[] {
  const result: PlannedProvisionedFile[] = [];
  for (const file of files) {
    if (result.length > 0 && result.at(-1)?.path === file.path) {
      continue;
    }
    result.push(file);
  }
  return result;
}
export function materializeProvisionedExports(project: ResolvedProject): void {
  for (const file of plannedProvisionedFiles(project)) {
    validateProjectRelativePath(file.path);
    const destination = resolve(project.projectRoot, file.path);
    mkdirSync(resolve(destination, ".."), { recursive: true });
    writeProvisionedFile(
      file.source,
      destination,
      project.manifest.symbols ?? {},
    );
  }
}
export function expectedProvisionedBytes(
  source: string,
  symbols: Record<string, string>,
): Buffer {
  const bytes = readFileSync(source);
  try {
    const decoded = new TextDecoder("utf-8", { fatal: true }).decode(bytes);
    return Buffer.from(substitutePraySymbols(decoded, symbols), "utf8");
  } catch (error) {
    if (error instanceof PrayError) {
      throw error;
    }
    return bytes;
  }
}

function writeProvisionedFile(
  source: string,
  destination: string,
  symbols: Record<string, string>,
): void {
  writeFileSync(destination, expectedProvisionedBytes(source, symbols));
}

function collectExactFileBindings(
  project: ResolvedProject,
  planned: PlannedProvisionedFile[],
): void {
  for (const packageEntry of project.packages) {
    const destination = packageEntry.declaration.file;
    if (!destination) {
      continue;
    }
    if (
      !packageMatchesEnvironment(
        packageEntry.declaration.groups,
        project.environment,
      )
    ) {
      continue;
    }
    let matched = false;
    for (const exportName of packageEntry.selectedExports) {
      const exportEntry = packageEntry.spec.exports.get(exportName);
      if (!exportEntry || exportEntry.kind !== "file") {
        continue;
      }
      const source = resolve(packageEntry.root, exportEntry.path);
      if (!existsSync(source)) {
        throw PrayError.render(`file export source missing: ${source}`);
      }
      planned.push({ path: destination, source });
      matched = true;
      break;
    }
    if (!matched) {
      throw PrayError.render(
        `package ${packageEntry.declaration.name} has file: "${destination}" but no selected file export`,
      );
    }
  }
}

function collectLegacySkillFiles(
  project: ResolvedProject,
  packageEntry: ResolvedPackage,
  destinationRoot: string,
  planned: PlannedProvisionedFile[],
): void {
  for (const [skillName, skill] of packageEntry.spec.skills) {
    if (legacySkillCoveredByExport(packageEntry, skill.path)) {
      continue;
    }
    const skillFiles = packageEntry.skillFiles.get(skillName);
    if (!skillFiles) {
      throw PrayError.render(
        `package ${packageEntry.declaration.name} has no indexed files for legacy skill ${skillName}`,
      );
    }
    collectTreeFiles(
      project,
      resolve(packageEntry.root, skill.path),
      resolve(destinationRoot, skillName),
      skillFiles,
      [],
      [],
      planned,
    );
  }
}

function legacySkillCoveredByExport(
  packageEntry: ResolvedPackage,
  skillPath: string,
): boolean {
  const trimmedSkillPath = skillPath.replace(/\/$/, "");
  for (const [exportName, exportEntry] of packageEntry.spec.exports) {
    if (
      packageEntry.selectedExports.includes(exportName) &&
      isFolderExportKind(exportEntry.kind) &&
      exportEntry.path.replace(/\/$/, "") === trimmedSkillPath
    ) {
      return true;
    }
  }
  return false;
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
    if (isFolderExportKind(exportEntry.kind)) {
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
        exportEntry.only ?? [],
        exportEntry.except ?? [],
        planned,
      );
      continue;
    }
    if (exportEntry.kind === "file") {
      if (packageEntry.declaration.file) {
        continue;
      }
      const source = resolve(packageEntry.root, exportEntry.path);
      if (!existsSync(source)) {
        throw PrayError.render(`file export source missing: ${source}`);
      }
      const fileName = basename(exportEntry.path);
      if (!fileName) {
        throw PrayError.render(
          `file export path has no file name: ${exportEntry.path}`,
        );
      }
      const destination = resolve(destinationRoot, exportName, fileName);
      planned.push({
        path: relativeProjectPath(project.projectRoot, destination),
        source,
      });
    }
  }
}

function isFolderExportKind(kind: string): boolean {
  return kind === "folder" || kind === "skill";
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
  only: string[],
  except: string[],
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

  let matched = false;
  for (const relative of relativeFiles) {
    if (only.length > 0 && !only.includes(relative)) {
      continue;
    }
    if (except.includes(relative)) {
      continue;
    }
    const source = resolve(sourceRoot, relative);
    if (!existsSync(source)) {
      throw PrayError.render(`provisioned file missing: ${source}`);
    }
    const destination = resolve(destinationRoot, relative);
    planned.push({
      path: relativeProjectPath(project.projectRoot, destination),
      source,
    });
    matched = true;
  }

  if (!matched && only.length === 0 && except.length === 0) {
    throw PrayError.render(
      `no files listed in package manifest for ${sourceRoot}`,
    );
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
