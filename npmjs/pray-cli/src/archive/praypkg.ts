import { spawnSync } from "node:child_process";
import {
  copyFileSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { basename, dirname, join, resolve } from "node:path";
import { PrayError } from "../errors.js";
import { findPrayspecFile } from "../package-spec/index.js";
import type { ResolvedPackage } from "../resolve/types.js";

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
  mkdirSync(outputDirectory, { recursive: true });
  const tarBytes = runCommand("zstd", ["-d", "-q", "-c"], artifactBytes);
  const result = spawnSync("tar", ["-xf", "-", "-C", outputDirectory], {
    input: tarBytes,
  });
  if (result.status !== 0) {
    throw PrayError.integrity("failed to unpack package archive");
  }
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

function runCommand(
  program: string,
  argumentsList: string[],
  input?: Buffer,
): Buffer {
  const result = spawnSync(program, argumentsList, {
    input,
    encoding: "buffer",
    maxBuffer: 64 * 1024 * 1024,
  });
  if ((result.error as NodeJS.ErrnoException | undefined)?.code === "ENOENT") {
    throw PrayError.unsupported(
      `${program} is required to build or unpack package archives`,
    );
  }
  if (result.status !== 0) {
    const message = result.stderr?.toString("utf8").trim();
    throw PrayError.integrity(
      message.length > 0 ? message : `${program} failed`,
    );
  }
  return result.stdout as Buffer;
}
