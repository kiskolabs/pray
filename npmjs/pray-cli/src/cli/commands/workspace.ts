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
import { resolveProject } from "../../resolve/project.js";

export async function runFormat(): Promise<void> {
  const lockfile = readLockfile(defaultLockfilePath(process.cwd()));
  for (const target of lockfile.target) {
    for (const output of target.outputs) {
      if (!existsSync(output)) {
        continue;
      }
      const original = readFileSync(output, "utf8");
      const formatted = formatMarkerComments(normalizeLineEndings(original));
      if (formatted !== original) {
        writeFileSync(output, formatted, "utf8");
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
