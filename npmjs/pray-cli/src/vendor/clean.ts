import { lstatSync, readdirSync, rmSync, unlinkSync } from "node:fs";
import { isAbsolute, join, relative, resolve } from "node:path";
import { readLockfile } from "../lockfile/index.js";

export function cleanUnusedRegistryCache(projectRoot: string): void {
  const lockfile = readLockfile(join(projectRoot, "Prayfile.lock"));
  const registryRoot = resolve(projectRoot, ".pray/cache/registry");
  const retained = lockfile.package
    .map((packageEntry) => resolve(projectRoot, packageEntry.path))
    .filter((path) => pathWithinRoot(registryRoot, path));

  let metadata: ReturnType<typeof lstatSync>;
  try {
    metadata = lstatSync(registryRoot);
  } catch (error) {
    if (isMissingPathError(error)) {
      return;
    }
    throw error;
  }
  if (metadata.isSymbolicLink() || !metadata.isDirectory()) {
    removePath(registryRoot);
    return;
  }

  pruneDirectory(registryRoot, retained, false);
}

function pruneDirectory(
  path: string,
  retained: string[],
  removeWhenEmpty: boolean,
): void {
  for (const name of readdirSync(path)) {
    const entry = join(path, name);
    const protectsEntry = retained.includes(entry);
    const leadsToRetained = retained.some((kept) =>
      pathWithinRoot(entry, kept),
    );
    if (protectsEntry) {
      continue;
    }

    const metadata = lstatSync(entry);
    if (
      leadsToRetained &&
      metadata.isDirectory() &&
      !metadata.isSymbolicLink()
    ) {
      pruneDirectory(entry, retained, true);
    } else {
      removePath(entry);
    }
  }

  if (removeWhenEmpty && readdirSync(path).length === 0) {
    rmSync(path);
  }
}

function pathWithinRoot(root: string, candidate: string): boolean {
  const fromRoot = relative(root, candidate);
  return (
    fromRoot === "" ||
    (!fromRoot.startsWith(`..${process.platform === "win32" ? "\\" : "/"}`) &&
      fromRoot !== ".." &&
      !isAbsolute(fromRoot))
  );
}

function removePath(path: string): void {
  const metadata = lstatSync(path);
  if (metadata.isDirectory() && !metadata.isSymbolicLink()) {
    rmSync(path, { recursive: true });
  } else {
    unlinkSync(path);
  }
}

function isMissingPathError(error: unknown): boolean {
  return (
    error instanceof Error &&
    "code" in error &&
    (error as NodeJS.ErrnoException).code === "ENOENT"
  );
}
