import { existsSync } from "node:fs";
import { join } from "node:path";
import { runGit, runGitCapture, tryRunGit } from "./run.js";

export function materializeGitCatalogFile(
  sourceRoot: string,
  relative: string,
): void {
  if (existsSync(join(sourceRoot, relative))) {
    return;
  }
  if (!sparseCheckoutEnabled(sourceRoot)) {
    return;
  }
  const toplevel = gitToplevel(sourceRoot);
  if (toplevel === undefined) {
    return;
  }
  const repoRelative = repoRelativeText(sourceRoot, relative);
  if (repoRelative === undefined) {
    return;
  }
  const cone = artifactSparseCone(repoRelative);
  if (cone !== undefined) {
    runGit(toplevel, "sparse-checkout", "add", cone);
  }
  runGit(toplevel, "checkout", "HEAD", "--", repoRelative);
}

function sparseCheckoutEnabled(sourceRoot: string): boolean {
  if (!tryRunGit(sourceRoot, "config", "--get", "core.sparseCheckout")) {
    return false;
  }
  return (
    runGitCapture(
      sourceRoot,
      "config",
      "--get",
      "core.sparseCheckout",
    ).trim() === "true"
  );
}

function gitToplevel(sourceRoot: string): string | undefined {
  if (!tryRunGit(sourceRoot, "rev-parse", "--show-toplevel")) {
    return undefined;
  }
  const text = runGitCapture(sourceRoot, "rev-parse", "--show-toplevel").trim();
  return text.length === 0 ? undefined : text;
}

function repoRelativeText(
  sourceRoot: string,
  relative: string,
): string | undefined {
  if (!tryRunGit(sourceRoot, "rev-parse", "--show-prefix")) {
    return undefined;
  }
  const prefix = runGitCapture(sourceRoot, "rev-parse", "--show-prefix").trim();
  if (prefix.length === 0) {
    return relative;
  }
  return `${prefix.replace(/\/$/, "")}/${relative}`;
}

function artifactSparseCone(repoRelative: string): string | undefined {
  const parts = repoRelative.split("/").filter((part) => part.length > 0);
  if (parts.length < 2) {
    return repoRelative;
  }
  parts.pop();
  if (parts[parts.length - 1] !== "artifacts" && parts.length > 3) {
    parts.pop();
  }
  return parts.join("/");
}
