import { existsSync, readFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { versionSatisfies } from "../constraint.js";
import { validateEnvironment } from "../environment.js";
import { PrayError } from "../errors.js";
import { prepareGitSources } from "../git/sources.js";
import { normalizeLineEndings } from "../hashing.js";
import { readLockfile } from "../lockfile/index.js";
import { defaultLockfilePath } from "../lockfile/paths.js";
import type { Lockfile } from "../lockfile/types.js";
import {
  manifestHash,
  parseManifest,
  readManifestText,
} from "../manifest/index.js";
import type {
  ManifestLocal,
  ManifestPackage,
  ManifestSource,
} from "../manifest/types.js";
import {
  findPrayspecFile,
  parsePackageSpec,
  treeHashForRoot,
} from "../package-spec/index.js";
import type { PackageSpec } from "../package-spec/types.js";
import { activeInvocationContext } from "../project-context/runtime.js";
import { defaultResolveOptions, type ResolveOptions } from "./context.js";
import { resolvePackageRoot, vendoredPackageRoot } from "./package-root.js";
import type {
  ResolvedLocalFile,
  ResolvedPackage,
  ResolvedProject,
} from "./types.js";

export async function resolveProject(
  manifestPath: string,
  options: ResolveOptions = defaultResolveOptions(),
): Promise<ResolvedProject> {
  const projectRoot = canonicalProjectRoot(manifestPath);
  const lockfilePath = defaultLockfilePath(projectRoot);
  const lockfile = existsSync(lockfilePath)
    ? readLockfile(lockfilePath)
    : undefined;
  const manifest = parseManifest(readManifestText(manifestPath));
  const environment =
    options.environment ?? activeInvocationContext()?.environment;
  validateEnvironment(manifest, environment);
  const manifestHashValue = manifestHash(manifest);
  const sources = sourceMap(manifest.sources);
  const gitSources = prepareGitSources(
    projectRoot,
    manifest.sources,
    lockfile,
    options.refreshSourceRevisions,
  );
  const sourceRevisions = new Map<string, string>();
  for (const [name, checkout] of gitSources.entries()) {
    if (checkout.revision) {
      sourceRevisions.set(name, checkout.revision);
    }
  }

  const packages: ResolvedPackage[] = [];
  const seen = new Set<string>();
  const resolutionErrors: string[] = [];

  for (const declaration of manifest.packages) {
    try {
      const packageEntry = await resolvePackage(
        projectRoot,
        sources,
        gitSources,
        declaration,
        lockfile,
        options,
      );
      if (seen.has(packageEntry.declaration.name)) {
        throw PrayError.resolution(
          `duplicate package declaration: ${packageEntry.declaration.name}`,
        );
      }
      seen.add(packageEntry.declaration.name);
      packages.push(packageEntry);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      resolutionErrors.push(`${declaration.name}: ${message}`);
    }
  }

  if (resolutionErrors.length > 0) {
    throw PrayError.resolution(resolutionErrors.join("\n"));
  }

  const localFiles: ResolvedLocalFile[] = [];
  const localErrors: string[] = [];
  for (const local of manifest.local) {
    try {
      localFiles.push(resolveLocalFile(projectRoot, local));
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      localErrors.push(`local ${local.path}: ${message}`);
    }
  }
  if (localErrors.length > 0) {
    throw PrayError.resolution(localErrors.join("\n"));
  }

  return {
    manifestPath: resolve(manifestPath),
    projectRoot,
    manifest,
    manifestHash: manifestHashValue,
    packages,
    localFiles,
    sourceRevisions,
    sourceHostKeys: new Map(),
    ...(environment ? { environment } : {}),
  };
}

export async function resolveProjectWithGitRefreshFallback(
  manifestPath: string,
  options: ResolveOptions = defaultResolveOptions(),
  allowGitRefreshFallback = false,
): Promise<ResolvedProject> {
  try {
    return await resolveProject(manifestPath, options);
  } catch (error) {
    if (
      allowGitRefreshFallback &&
      !options.offline &&
      !options.refreshSourceRevisions &&
      resolutionMayBenefitFromGitSourceRefresh(error)
    ) {
      return resolveProject(manifestPath, {
        ...options,
        refreshSourceRevisions: true,
      });
    }
    throw error;
  }
}

function resolutionMayBenefitFromGitSourceRefresh(error: unknown): boolean {
  return (
    error instanceof PrayError &&
    error.kind === "resolution" &&
    error.message.includes("no registry version")
  );
}

function canonicalProjectRoot(manifestPath: string): string {
  const root = resolve(manifestPath, "..");
  return root.length === 0 ? process.cwd() : root;
}

async function resolvePackage(
  projectRoot: string,
  sources: Map<string, ManifestSource>,
  gitSources: ReturnType<typeof prepareGitSources>,
  declaration: ManifestPackage,
  lockfile: Lockfile | undefined,
  options: ResolveOptions,
): Promise<ResolvedPackage> {
  const vendoredVersion = lockfile?.package.find(
    (entry) => entry.name === declaration.name,
  )?.version;
  const vendoredRoot = vendoredVersion
    ? vendoredPackageRoot(projectRoot, declaration.name, vendoredVersion)
    : undefined;

  const resolution = vendoredRoot
    ? { root: vendoredRoot }
    : await resolvePackageRoot(
        projectRoot,
        sources,
        gitSources,
        declaration,
        lockfile,
        options,
      );

  const root = resolution.root;
  const specPath = findPrayspecFile(root);
  const spec = parsePackageSpec(readFileSync(specPath, "utf8"));
  if (spec.name !== declaration.name) {
    throw PrayError.resolution(
      `package path ${root} declares ${spec.name}, expected ${declaration.name}`,
    );
  }
  if (!versionSatisfies(spec.version, declaration.constraint)) {
    throw PrayError.resolution(
      `package ${declaration.name} version ${spec.version} does not satisfy constraint ${declaration.constraint}`,
    );
  }
  const selectedExports = selectExports(declaration, spec);
  const treeHash = treeHashForRoot(root, spec);
  const exportBodies = loadExportBodies(root, spec, selectedExports);
  const skillFiles = buildSkillFileIndex(spec);
  return {
    declaration,
    root,
    spec,
    treeHash,
    artifactHash: treeHash,
    artifact: `path:${resolve(root)}`,
    selectedExports,
    sourceChecksum: treeHash,
    exportBodies,
    skillFiles,
    signerFingerprint: resolution.signerFingerprint,
    registryLatestVersion: resolution.registryLatestVersion,
  };
}

function resolveLocalFile(
  projectRoot: string,
  declaration: ManifestLocal,
): ResolvedLocalFile {
  const path = resolve(projectRoot, declaration.path);
  if (!existsSync(path)) {
    if (declaration.optional) {
      return {
        path,
        manifestPath: declaration.path,
        content: "",
        position: declaration.position,
        optional: true,
      };
    }
    throw PrayError.resolution(missingLocalEmbedGuidance(declaration.path));
  }
  return {
    path,
    manifestPath: declaration.path,
    content: normalizeLineEndings(readFileSync(path, "utf8")),
    position: declaration.position,
    optional: declaration.optional,
  };
}

export function missingLocalEmbedGuidance(path: string): string {
  return (
    `Prayfile lists \`local "${path}"\` but the file does not exist. ` +
    "Create the file or remove the entry from Prayfile, then run `pray install`."
  );
}

function sourceMap(sources: ManifestSource[]): Map<string, ManifestSource> {
  return new Map(sources.map((source) => [source.name, source]));
}

function selectExports(
  declaration: ManifestPackage,
  spec: PackageSpec,
): string[] {
  if (declaration.exports.length === 0) {
    return [...spec.exports.keys()].sort();
  }
  for (const exportName of declaration.exports) {
    if (!spec.exports.has(exportName)) {
      throw PrayError.resolution(
        `package ${declaration.name} does not export ${exportName}`,
      );
    }
  }
  return [...declaration.exports];
}

function loadExportBodies(
  root: string,
  spec: PackageSpec,
  selectedExports: string[],
): Map<string, string> {
  const exportBodies = new Map<string, string>();
  for (const exportName of selectedExports) {
    const entry = spec.exports.get(exportName);
    if (!entry) {
      throw PrayError.resolution(
        `package ${spec.name} is missing export ${exportName}`,
      );
    }
    if (entry.kind !== "fragment") {
      continue;
    }
    const filePath = join(root, entry.path);
    if (!existsSync(filePath)) {
      throw PrayError.integrity(
        `package file missing for export ${exportName}: ${entry.path}`,
      );
    }
    exportBodies.set(
      exportName,
      normalizeLineEndings(readFileSync(filePath, "utf8")),
    );
  }
  return exportBodies;
}

function buildSkillFileIndex(spec: PackageSpec): Map<string, string[]> {
  const index = new Map<string, string[]>();
  for (const [exportName, exportEntry] of spec.exports.entries()) {
    if (exportEntry.kind !== "folder" && exportEntry.kind !== "skill") {
      continue;
    }
    const prefix = exportEntry.path.replace(/\/$/, "");
    index.set(
      exportName,
      indexedFilesUnderPrefix(spec.files, prefix).map((file) =>
        skillRelativeFile(file, prefix),
      ),
    );
  }
  for (const [skillName, skill] of spec.skills.entries()) {
    const prefix = skill.path.replace(/\/$/, "");
    index.set(
      skillName,
      indexedFilesUnderPrefix(spec.files, prefix).map((file) =>
        skillRelativeFile(file, prefix),
      ),
    );
  }
  return index;
}

function indexedFilesUnderPrefix(files: string[], prefix: string): string[] {
  const normalizedPrefix = prefix.endsWith("/") ? prefix : `${prefix}/`;
  return files
    .filter((file) => file === prefix || file.startsWith(normalizedPrefix))
    .sort();
}

function skillRelativeFile(file: string, skillPrefix: string): string {
  const normalizedPrefix = skillPrefix.replace(/\/$/, "");
  if (!file.startsWith(normalizedPrefix)) {
    return file;
  }
  const relative = file.slice(normalizedPrefix.length).replace(/^\//, "");
  if (relative.length === 0 || file === normalizedPrefix) {
    return file.split("/").pop() ?? file;
  }
  return relative;
}

export { defaultResolveOptions, type ResolveOptions } from "./context.js";
export type { ResolvedProject } from "./types.js";
