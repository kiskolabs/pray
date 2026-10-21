import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, relative as pathRelative } from "node:path";
import { PrayError } from "../errors.js";
import { MAX_ARCHIVE_ENTRY_BYTES } from "../resource-limits.js";
import { gitEnvironment } from "./environment.js";
import {
  annotateGitMaterializeFailure,
  requireRevisionPresent,
} from "./materialize-check.js";

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
  requireRevisionPresent(markers.gitDir, markers.revision, offline);
  const repoRelative = pathRelative(markers.catalogRoot, fullPath).replace(
    /\\/g,
    "/",
  );
  if (repoRelative.startsWith("..")) {
    return;
  }
  const object = `${markers.revision}:${repoRelative}`;
  const shown = spawnSync(
    "git",
    ["-c", "protocol.file.allow=always", "-C", markers.gitDir, "show", object],
    {
      encoding: "buffer",
      env: gitEnvironment(offlineGitEnv(offline)),
      input: "",
      maxBuffer: MAX_ARCHIVE_ENTRY_BYTES + 1,
    },
  );
  if (shown.error && (shown.error as NodeJS.ErrnoException).code === "ENOBUFS") {
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
  if (shown.stdout.length > MAX_ARCHIVE_ENTRY_BYTES) {
    throw PrayError.integrity(
      `git catalog blob ${repoRelative} exceeds ${MAX_ARCHIVE_ENTRY_BYTES} bytes`,
    );
  }
  mkdirSync(dirname(fullPath), { recursive: true });
  writeFileSync(fullPath, shown.stdout);
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
      if (revision.length === 0 || !existsSync(gitDir)) {
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
