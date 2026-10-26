import { realpathSync } from "node:fs";
import { join } from "node:path";
import { PrayError } from "../errors.js";
import { runGitCapture } from "./run.js";

// Git searches parent directories when `repository` is not a valid git
// directory, so a damaged cache inside another checkout would otherwise
// rewrite that checkout's origin.
export function requireOwnGitDir(repository: string): void {
  const resolved = resolvedGitDir(repository);
  const own = [repository, join(repository, ".git")].some(
    (candidate) =>
      realPath(candidate) !== undefined &&
      realPath(candidate) === realPath(resolved),
  );
  if (own) {
    return;
  }
  throw PrayError.resolution(
    `git cache ${repository} is not a usable git repository (git resolved ${JSON.stringify(resolved)}). Remove it and run the command again.`,
  );
}

function resolvedGitDir(repository: string): string {
  try {
    return runGitCapture(repository, "rev-parse", "--absolute-git-dir").trim();
  } catch {
    return "";
  }
}

function realPath(path: string): string | undefined {
  try {
    return realpathSync(path);
  } catch {
    return undefined;
  }
}
