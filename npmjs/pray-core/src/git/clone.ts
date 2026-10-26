import { runGit, tryRunGit } from "./run.js";

export function cloneBareGitDb(
  workingDirectory: string,
  source: string,
  destination: string,
  quiet: boolean,
): void {
  const local =
    source.startsWith("file://") || source.startsWith("/")
      ? ["--no-local"]
      : [];
  const filtered = [
    "clone",
    "--bare",
    "--depth",
    "1",
    "--filter=blob:none",
    ...local,
    source,
    destination,
  ];
  if (quiet) {
    filtered.splice(1, 0, "--quiet");
  }
  if (tryRunGit(workingDirectory, ...filtered)) {
    return;
  }
  const full = [
    "clone",
    "--bare",
    "--depth",
    "1",
    ...local,
    source,
    destination,
  ];
  if (quiet) {
    full.splice(1, 0, "--quiet");
  }
  runGit(workingDirectory, ...full);
}

export function catalogSparseCones(subdir?: string): string[] {
  if (subdir && subdir.length > 0) {
    return [`${subdir}/v1/packages`];
  }
  return ["v1/packages", "prayers/v1/packages"];
}
