import { spawnSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { PrayError } from "../errors.js";
import { catalogSparseCones } from "./clone.js";
import { runGit } from "./run.js";

const REVISION_MARKER = ".pray-revision";
const GIT_DIR_MARKER = ".pray-git-dir";

export function materializeCatalogTree(
  gitDir: string,
  dest: string,
  revision: string,
  subdir: string | undefined,
  refresh: boolean,
): void {
  if (!refresh && catalogMatches(dest, gitDir, revision)) {
    return;
  }
  if (existsSync(dest)) {
    rmSync(dest, { recursive: true, force: true });
  }
  mkdirSync(dest, { recursive: true });
  let unpacked = false;
  for (const prefix of catalogSparseCones(subdir)) {
    if (unpackArchivePrefix(gitDir, dest, revision, prefix)) {
      unpacked = true;
    }
  }
  if (!unpacked) {
    throw PrayError.resolution(
      `no pray distribution root in git source at revision ${revision}`,
    );
  }
  writeFileSync(join(dest, REVISION_MARKER), revision);
  writeFileSync(join(dest, GIT_DIR_MARKER), gitDir);
}

export function materializeGitCatalogFile(
  sourceRoot: string,
  relative: string,
): void {
  if (existsSync(join(sourceRoot, relative))) {
    return;
  }
  const markers = findCatalogMarkers(sourceRoot);
  if (markers === undefined) {
    return;
  }
  const fullPath = join(sourceRoot, relative);
  if (!fullPath.startsWith(markers.workTree)) {
    return;
  }
  const repoRelative = fullPath
    .slice(markers.workTree.length)
    .replace(/^\/+/, "");
  runGit(
    markers.gitDir,
    "--work-tree",
    markers.workTree,
    "checkout",
    markers.revision,
    "--",
    repoRelative,
  );
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
): boolean {
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
    { encoding: "buffer", env: { ...process.env, GIT_TERMINAL_PROMPT: "0" } },
  );
  if (archive.status !== 0 || !archive.stdout || archive.stdout.length === 0) {
    return false;
  }
  const extract = spawnSync("tar", ["-x", "-C", dest], {
    input: archive.stdout,
    encoding: "buffer",
  });
  if (extract.status !== 0) {
    throw PrayError.resolution("failed to unpack git catalog archive");
  }
  return true;
}

function findCatalogMarkers(
  start: string,
): { gitDir: string; revision: string; workTree: string } | undefined {
  let current = start;
  for (let index = 0; index < 8; index += 1) {
    const revisionPath = join(current, REVISION_MARKER);
    const gitDirPath = join(current, GIT_DIR_MARKER);
    if (existsSync(revisionPath) && existsSync(gitDirPath)) {
      const revision = readFileSync(revisionPath, "utf8").trim();
      const gitDir = readFileSync(gitDirPath, "utf8").trim();
      if (revision.length === 0 || !existsSync(gitDir)) {
        return undefined;
      }
      return { gitDir, revision, workTree: current };
    }
    const parent = join(current, "..");
    if (parent === current) {
      return undefined;
    }
    current = parent;
  }
  return undefined;
}
