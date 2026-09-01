import { spawnSync } from "node:child_process";
import {
  copyFileSync,
  lstatSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { basename, dirname, join, resolve } from "node:path";
import { PrayError } from "../errors.js";
import { findPrayspecFile } from "../package-spec/index.js";
import type { ResolvedPackage } from "../resolve/types.js";
import { MAX_ARCHIVE_TOTAL_BYTES } from "../resource-limits.js";
import { validateArchiveMemberPath } from "./path-safety.js";
import { extractTarArchive } from "./tar.js";

export function buildPackageArchiveBytes(
  packageEntry: ResolvedPackage,
): Buffer {
  const prayspecPath = findPrayspecFile(packageEntry.root);
  const staging = mkdtempSync(join(tmpdir(), "pray-package-"));
  try {
    writeFileSync(
      join(staging, "metadata.json"),
      packageMetadataJson(packageEntry),
      "utf8",
    );
    writeFileSync(
      join(staging, basename(prayspecPath)),
      readFileSync(prayspecPath),
    );
    for (const file of packageEntry.spec.files) {
      const destination = join(staging, file);
      mkdirSync(dirname(destination), { recursive: true });
      copyFileSync(join(packageEntry.root, file), destination);
    }
    const tarBytes = runCommand("tar", ["-cf", "-", "-C", staging, "."]);
    return runCommand("zstd", ["-q", "-c"], tarBytes);
  } finally {
    rmSync(staging, { recursive: true, force: true });
  }
}

export function unpackPraypkg(
  artifactBytes: Buffer,
  outputDirectory: string,
): void {
  if (artifactBytes.byteLength > MAX_ARCHIVE_TOTAL_BYTES) {
    throw PrayError.integrity(
      `package archive exceeds ${MAX_ARCHIVE_TOTAL_BYTES} bytes`,
    );
  }

  const tarBytes = runCommand("zstd", ["-d", "-q", "-c"], artifactBytes);
  if (tarBytes.byteLength > MAX_ARCHIVE_TOTAL_BYTES) {
    throw PrayError.integrity(
      `package archive exceeds ${MAX_ARCHIVE_TOTAL_BYTES} decompressed bytes`,
    );
  }

  extractTarArchive(tarBytes, outputDirectory);
  rejectUnsafeExtractedTree(outputDirectory);
}

export function packageArchivePath(
  packageName: string,
  version: string,
): string {
  const slug = packageName.replaceAll("/", "-");
  return join(".pray", "packages", `${slug}-${version}.praypkg`);
}

export function writePackageArchive(
  packageEntry: ResolvedPackage,
  outputPath: string,
): void {
  const bytes = buildPackageArchiveBytes(packageEntry);
  mkdirSync(dirname(resolve(outputPath)), { recursive: true });
  writeFileSync(outputPath, bytes);
}

function packageMetadataJson(packageEntry: ResolvedPackage): string {
  return JSON.stringify({
    name: packageEntry.spec.name,
    version: packageEntry.spec.version,
    tree_hash: packageEntry.treeHash,
    exports: packageEntry.selectedExports,
  });
}

function rejectUnsafeExtractedTree(outputDirectory: string): void {
  const stack = [outputDirectory];
  while (stack.length > 0) {
    const current = stack.pop();
    if (current === undefined) {
      continue;
    }
    for (const entry of readdirSync(current, { withFileTypes: true })) {
      const fullPath = join(current, entry.name);
      if (entry.isDirectory()) {
        stack.push(fullPath);
        continue;
      }
      if (lstatSync(fullPath).isSymbolicLink()) {
        throw PrayError.integrity("unsupported package archive entry type");
      }
      const relative = fullPath.slice(outputDirectory.length + 1);
      validateArchiveMemberPath(relative);
    }
  }
}

function runCommand(
  program: string,
  argumentsList: string[],
  input?: Buffer,
): Buffer {
  const result = spawnSync(program, argumentsList, {
    input,
    encoding: "buffer",
    env: { ...process.env, COPYFILE_DISABLE: "1" },
    maxBuffer: MAX_ARCHIVE_TOTAL_BYTES,
  });
  if ((result.error as NodeJS.ErrnoException | undefined)?.code === "ENOENT") {
    throw PrayError.unsupported(
      `${program} is required to build or unpack package archives`,
    );
  }
  if (result.status !== 0) {
    const message = result.stderr?.toString("utf8").trim();
    throw PrayError.integrity(
      message && message.length > 0 ? message : `${program} failed`,
    );
  }
  return result.stdout as Buffer;
}
