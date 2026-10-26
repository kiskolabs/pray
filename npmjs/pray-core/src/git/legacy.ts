import { spawnSync } from "node:child_process";
import { existsSync, readdirSync, renameSync, rmSync } from "node:fs";
import { join } from "node:path";
import { gitEnvironment } from "./environment.js";
import { isGitCheckout } from "./lookup.js";

/** Rename project-local full clones so they cannot win over the shared bare. */
export function quarantineLegacyProjectGitCheckouts(
  projectRoot: string,
  cloneUrl: string,
): void {
  const cacheRoot = join(projectRoot, ".pray", "cache", "git");
  if (!existsSync(cacheRoot)) {
    return;
  }
  for (const name of readdirSync(cacheRoot)) {
    if (name.endsWith(".legacy-git")) {
      continue;
    }
    const path = join(cacheRoot, name);
    if (!isGitCheckout(path) || !originMatches(path, cloneUrl)) {
      continue;
    }
    const legacy = join(cacheRoot, `${name}.legacy-git`);
    if (existsSync(legacy)) {
      rmSync(legacy, { recursive: true, force: true });
    }
    renameSync(path, legacy);
  }
}

function originMatches(repository: string, cloneUrl: string): boolean {
  const result = spawnSync(
    "git",
    ["-C", repository, "remote", "get-url", "origin"],
    {
      encoding: "utf8",
      env: gitEnvironment(),
    },
  );
  if (result.status !== 0) {
    return false;
  }
  const origin = (result.stdout ?? "").trim().replace(/^git\+/, "");
  return origin === cloneUrl;
}
