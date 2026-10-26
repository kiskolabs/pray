import { spawnSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  renameSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { dirname, join } from "node:path";
import { extractTarArchive } from "../archive/tar.js";
import { PrayError } from "../errors.js";
import {
  MAX_ARCHIVE_ENTRIES,
  MAX_ARCHIVE_ENTRY_BYTES,
  MAX_ARCHIVE_TOTAL_BYTES,
} from "../resource-limits.js";
import { catalogSparseCones } from "./clone.js";
import { gitEnvironment } from "./environment.js";
import { requireRevisionPresent } from "./materialize-check.js";
import { offlineGitEnv } from "./materialize-file.js";
import { tryRunGit } from "./run.js";
import { withGitCacheLock } from "./store-lock.js";

export { materializeGitCatalogFile } from "./materialize-file.js";

const REVISION_MARKER = ".pray-revision";
const GIT_DIR_MARKER = ".pray-git-dir";

export function materializeCatalogTree(
  gitDir: string,
  dest: string,
  revision: string,
  subdir: string | undefined,
  refresh: boolean,
  offline = false,
): void {
  requireRevisionPresent(gitDir, revision, offline);
  withGitCacheLock(dest, () => {
    if (!refresh && catalogMatches(dest, gitDir, revision)) {
      return;
    }
    const staging = `${dest}.partial`;
    if (existsSync(staging)) {
      rmSync(staging, { recursive: true, force: true });
    }
    mkdirSync(staging, { recursive: true });
    let unpacked = false;
    for (const prefix of catalogSparseCones(subdir)) {
      if (unpackArchivePrefix(gitDir, staging, revision, prefix, offline)) {
        unpacked = true;
      }
    }
    if (!unpacked) {
      rmSync(staging, { recursive: true, force: true });
      if (offline) {
        throw PrayError.resolution(
          `git catalog at revision ${revision} is not materializable locally and offline mode is enabled`,
        );
      }
      throw PrayError.resolution(
        `no pray distribution root in git source at revision ${revision}`,
      );
    }
    writeFileSync(join(staging, REVISION_MARKER), revision);
    writeFileSync(join(staging, GIT_DIR_MARKER), gitDir);
    publishCatalogTree(dest, staging);
  });
}

function catalogMatches(
  dest: string,
  gitDir: string,
  revision: string,
): boolean {
  const revisionPath = join(dest, REVISION_MARKER);
  const gitDirPath = join(dest, GIT_DIR_MARKER);
  if (!existsSync(revisionPath) || !existsSync(gitDirPath)) {
    return false;
  }
  if (readFileSync(revisionPath, "utf8").trim() !== revision) {
    return false;
  }
  if (readFileSync(gitDirPath, "utf8").trim() !== gitDir) {
    return false;
  }
  if (!tryRunGit(gitDir, "cat-file", "-e", revision)) {
    return false;
  }
  return (
    existsSync(join(dest, "v1/packages")) ||
    existsSync(join(dest, "prayers/v1/packages")) ||
    (existsSync(dest) &&
      readdirSync(dest).some((name) =>
        existsSync(join(dest, name, "v1/packages")),
      ))
  );
}

function unpackArchivePrefix(
  gitDir: string,
  dest: string,
  revision: string,
  prefix: string,
  offline: boolean,
): boolean {
  if (!treePrefixWithinLimits(gitDir, revision, prefix, offline)) {
    return false;
  }
  const archive = spawnSync(
    "git",
    [
      "-c",
      "protocol.file.allow=always",
      "-C",
      gitDir,
      "archive",
      "--format=tar",
      revision,
      "--",
      prefix,
    ],
    {
      encoding: "buffer",
      env: gitEnvironment(offlineGitEnv(offline)),
      maxBuffer: MAX_ARCHIVE_TOTAL_BYTES + 1,
    },
  );
  if (
    archive.error &&
    (archive.error as NodeJS.ErrnoException).code === "ENOBUFS"
  ) {
    throw PrayError.integrity(
      `git catalog archive exceeds ${MAX_ARCHIVE_TOTAL_BYTES} bytes`,
    );
  }
  if (archive.status !== 0 || !archive.stdout || archive.stdout.length === 0) {
    return false;
  }
  if (archive.stdout.length > MAX_ARCHIVE_TOTAL_BYTES) {
    throw PrayError.integrity(
      `git catalog archive exceeds ${MAX_ARCHIVE_TOTAL_BYTES} bytes`,
    );
  }
  extractTarArchive(archive.stdout, dest);
  return true;
}

function treePrefixWithinLimits(
  gitDir: string,
  revision: string,
  prefix: string,
  offline: boolean,
): boolean {
  const listed = spawnSync(
    "git",
    [
      "-c",
      "protocol.file.allow=always",
      "-C",
      gitDir,
      "ls-tree",
      "-r",
      "-l",
      revision,
      "--",
      prefix,
    ],
    {
      encoding: "utf8",
      env: gitEnvironment(offlineGitEnv(offline)),
    },
  );
  if (listed.status !== 0 || !listed.stdout?.trim()) {
    return false;
  }
  let entryCount = 0;
  let totalBytes = 0;
  for (const line of listed.stdout.split("\n")) {
    const parts = line.trim().split(/\s+/);
    if (parts[1] !== "blob") {
      continue;
    }
    const size = Number.parseInt(parts[3] ?? "", 10);
    if (!Number.isFinite(size)) {
      continue;
    }
    entryCount += 1;
    if (entryCount > MAX_ARCHIVE_ENTRIES) {
      throw PrayError.integrity(
        `git catalog archive exceeds ${MAX_ARCHIVE_ENTRIES} entries`,
      );
    }
    if (size > MAX_ARCHIVE_ENTRY_BYTES) {
      throw PrayError.integrity(
        `git catalog archive entry exceeds ${MAX_ARCHIVE_ENTRY_BYTES} bytes`,
      );
    }
    totalBytes += size;
    if (totalBytes > MAX_ARCHIVE_TOTAL_BYTES) {
      throw PrayError.integrity(
        `git catalog archive exceeds ${MAX_ARCHIVE_TOTAL_BYTES} bytes`,
      );
    }
  }
  return entryCount > 0;
}

function publishCatalogTree(dest: string, staging: string): void {
  mkdirSync(dirname(dest), { recursive: true });
  if (existsSync(dest)) {
    const backup = `${dest}.previous`;
    if (existsSync(backup)) {
      rmSync(backup, { recursive: true, force: true });
    }
    renameSync(dest, backup);
    try {
      renameSync(staging, dest);
    } catch (error) {
      if (existsSync(backup)) {
        renameSync(backup, dest);
      }
      rmSync(staging, { recursive: true, force: true });
      throw error;
    }
    rmSync(backup, { recursive: true, force: true });
    return;
  }
  renameSync(staging, dest);
}
