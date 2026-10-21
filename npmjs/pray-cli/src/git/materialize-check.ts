import { PrayError } from "../errors.js";
import { tryRunGit } from "./run.js";

export function requireRevisionPresent(gitDir: string, revision: string): void {
  if (tryRunGit(gitDir, "cat-file", "-e", revision)) {
    return;
  }
  throw PrayError.resolution(
    `git revision ${revision} is not available in ${gitDir} (git_revision_unfetched). Run \`pray install --locked\`, or clear the project git cache and shared bare then install again.`,
  );
}

export function annotateGitMaterializeFailure(
  gitDir: string,
  revision: string,
  error: unknown,
): never {
  throw PrayError.resolution(
    `git catalog materialize failed for revision ${revision} in ${gitDir} (git_cache_stale): ${String(error)}. Run \`pray install --locked\`, or clear \`.pray/cache/git\` and the shared bare then install again.`,
  );
}
