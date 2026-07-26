import { existsSync, readFileSync, writeFileSync } from "node:fs";
import {
  packageArchivePath,
  writePackageArchive,
} from "../../archive/praypkg.js";
import { normalizeLineEndings } from "../../hashing.js";
import { readLockfile } from "../../lockfile/index.js";
import {
  defaultLockfilePath,
  defaultManifestPath,
} from "../../lockfile/paths.js";
import {
  classifyFormatHints,
  formatRecommended,
  parseManifest,
  readManifestText,
} from "../../manifest/index.js";
import { defaultResolveOptions } from "../../resolve/context.js";
import { resolveProject } from "../../resolve/project.js";
import type { ResolvedProject } from "../../resolve/types.js";

export async function runFormat(): Promise<void> {
  const manifestPath = defaultManifestPath();
  const original = readManifestText(manifestPath);
  const manifest = parseManifest(original);

  let project: ResolvedProject | undefined;
  try {
    project = await resolveProject(manifestPath, {
      ...defaultResolveOptions(),
      offline: true,
    });
  } catch {
    try {
      project = await resolveProject(manifestPath);
    } catch {
      project = undefined;
    }
  }

  if (project) {
    const hints = classifyFormatHints(project);
    const formatted = formatRecommended(manifest, hints);
    if (formatted !== original) {
      writeFileSync(manifestPath, formatted, "utf8");
    }
  }

  if (!existsSync(defaultLockfilePath(process.cwd()))) {
    return;
  }
  const lockfile = readLockfile(defaultLockfilePath(process.cwd()));
  for (const target of lockfile.target) {
    for (const output of target.outputs) {
      if (!existsSync(output)) {
        continue;
      }
      const originalOutput = readFileSync(output, "utf8");
      const formattedOutput = formatMarkerComments(
        normalizeLineEndings(originalOutput),
      );
      if (formattedOutput !== originalOutput) {
        writeFileSync(output, formattedOutput, "utf8");
      }
    }
  }
}

function formatMarkerComments(text: string): string {
  return text.replace(/<!--\s*pray:/g, "<!-- pray:");
}

export async function runPackage(): Promise<void> {
  const project = await resolveProject(defaultManifestPath());
  for (const packageEntry of project.packages) {
    const outputPath = packageArchivePath(
      packageEntry.declaration.name,
      packageEntry.spec.version,
    );
    writePackageArchive(packageEntry, outputPath);
    process.stdout.write(`wrote ${outputPath}\n`);
  }
}
