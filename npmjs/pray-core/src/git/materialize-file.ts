import { spawnSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  readFileSync,
  realpathSync,
  renameSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, relative as pathRelative } from "node:path";
import { PrayError } from "../errors.js";
import { MAX_ARCHIVE_ENTRY_BYTES } from "../resource-limits.js";
import { gitEnvironment } from "./environment.js";
import {
  annotateGitMaterializeFailure,
  requireRevisionPresent,
} from "./materialize-check.js";
import { globalCacheRoot } from "./store.js";
import { withGitCacheLock } from "./store-lock.js";

const REVISION_MARKER = ".pray-revision";
const GIT_DIR_MARKER = ".pray-git-dir";

export function materializeGitCatalogFile(
  sourceRoot: string,
  relative: string,
  offline = false,
): void {
  const fullPath = join(sourceRoot, relative);
  if (existsSync(fullPath)) {
    return;
  }
  const markers = findCatalogMarkers(sourceRoot);
  if (markers === undefined) {
    return;
  }
  withGitCacheLock(markers.catalogRoot, () => {
    if (existsSync(fullPath)) {
      return;
    }
    requireRevisionPresent(markers.gitDir, markers.revision, offline);
    const repoRelative = pathRelative(markers.catalogRoot, fullPath).replace(
      /\\/g,
      "/",
    );
    if (repoRelative.startsWith("..")) {
      return;
    }
    const object = `${markers.revision}:${repoRelative}`;
    requireBlobWithinLimit(markers.gitDir, object, offline);
    const shown = spawnSync(
      "git",
      [
        "-c",
        "protocol.file.allow=always",
        "-C",
        markers.gitDir,
        "show",
        object,
      ],
      {
        encoding: "buffer",
        env: gitEnvironment(offlineGitEnv(offline)),
        input: "",
        maxBuffer: MAX_ARCHIVE_ENTRY_BYTES + 1,
      },
    );
    if (
      shown.error &&
      (shown.error as NodeJS.ErrnoException).code === "ENOBUFS"
    ) {
      throw PrayError.integrity(
        `git catalog blob ${repoRelative} exceeds ${MAX_ARCHIVE_ENTRY_BYTES} bytes`,
      );
    }
    if (shown.status !== 0 || !shown.stdout) {
      if (offline) {
        throw PrayError.resolution(
          `git catalog blob ${repoRelative} at revision ${markers.revision} is not available locally and offline mode is enabled`,
        );
      }
      annotateGitMaterializeFailure(
        markers.gitDir,
        markers.revision,
        new Error("git show failed"),
      );
    }
    writeCatalogBlobAtomic(fullPath, shown.stdout);
  });
}

export function findCatalogMarkers(
  start: string,
): { gitDir: string; revision: string; catalogRoot: string } | undefined {
  let current = start;
  for (let index = 0; index < 8; index += 1) {
    const revisionPath = join(current, REVISION_MARKER);
    const gitDirPath = join(current, GIT_DIR_MARKER);
    if (existsSync(revisionPath) && existsSync(gitDirPath)) {
      const revision = readFileSync(revisionPath, "utf8").trim();
      const gitDir = readFileSync(gitDirPath, "utf8").trim();
      if (!acceptableRevision(revision) || !acceptableSharedBare(gitDir)) {
        return undefined;
      }
      return { gitDir, revision, catalogRoot: current };
    }
    const parent = join(current, "..");
    if (parent === current) {
      return undefined;
    }
    current = parent;
  }
  return undefined;
}

export function offlineGitEnv(offline: boolean): Record<string, string> {
  return offline
    ? { GIT_NO_LAZY_FETCH: "1", GIT_TERMINAL_PROMPT: "0" }
    : { GIT_TERMINAL_PROMPT: "0" };
}

function requireBlobWithinLimit(
  gitDir: string,
  object: string,
  offline: boolean,
): void {
  const sized = spawnSync(
    "git",
    [
      "-c",
      "protocol.file.allow=always",
      "-C",
      gitDir,
      "cat-file",
      "-s",
      object,
    ],
    {
      encoding: "utf8",
      env: gitEnvironment(offlineGitEnv(offline)),
    },
  );
  if (sized.status !== 0) {
    return;
  }
  const size = Number.parseInt(sized.stdout.trim(), 10);
  if (Number.isFinite(size) && size > MAX_ARCHIVE_ENTRY_BYTES) {
    throw PrayError.integrity(
      `git catalog blob exceeds ${MAX_ARCHIVE_ENTRY_BYTES} bytes`,
    );
  }
}

function writeCatalogBlobAtomic(fullPath: string, bytes: Buffer): void {
  mkdirSync(dirname(fullPath), { recursive: true });
  const partial = `${fullPath}.partial`;
  writeFileSync(partial, bytes);
  renameSync(partial, fullPath);
}

function acceptableRevision(revision: string): boolean {
  if (revision.length === 0 || revision.startsWith("-")) {
    return false;
  }
  if (revision.length !== 40 && revision.length !== 64) {
    return false;
  }
  return /^[0-9a-fA-F]+$/.test(revision);
}

function acceptableSharedBare(gitDir: string): boolean {
  if (
    !existsSync(join(gitDir, "HEAD")) ||
    !existsSync(join(gitDir, "objects"))
  ) {
    return false;
  }
  const root = globalCacheRoot();
  if (root === undefined) {
    return false;
  }
  const expected = join(root, "git");
  try {
    return realpathSync(gitDir).startsWith(realpathSync(expected));
  } catch {
    return gitDir.startsWith(expected);
  }
}
