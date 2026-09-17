import { runGit, tryRunGit } from "./run.js";

export function cloneGitCache(
  workingDirectory: string,
  source: string,
  destination: string,
  quiet: boolean,
): void {
  const filtered = [
    "clone",
    "--depth",
    "1",
    "--filter=blob:none",
    "--sparse",
    source,
    destination,
  ];
  if (quiet) {
    filtered.splice(1, 0, "--quiet");
  }
  if (tryRunGit(workingDirectory, ...filtered)) {
    return;
  }
  const full = ["clone", "--depth", "1", source, destination];
  if (quiet) {
    full.splice(1, 0, "--quiet");
  }
  runGit(workingDirectory, ...full);
}

export function applySparseCheckout(repository: string, subdir?: string): void {
  runGit(repository, "sparse-checkout", "init", "--cone");
  runGit(repository, "sparse-checkout", "set", ...catalogSparseCones(subdir));
}

function catalogSparseCones(subdir?: string): string[] {
  if (subdir && subdir.length > 0) {
    return [`${subdir}/v1/packages`];
  }
  return ["v1/packages", "prayers/v1/packages"];
}
