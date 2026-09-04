import { lstatSync, readdirSync, rmSync, unlinkSync } from "node:fs";
import { isAbsolute, join, relative, resolve } from "node:path";
import { PrayError } from "../errors.js";
import { readLockfile } from "../lockfile/index.js";
import type { Lockfile } from "../lockfile/types.js";

export function cleanUnusedRegistryCache(projectRoot: string): void {
  const lockfile = readLockfile(join(projectRoot, "Prayfile.lock"));
  validateLockfileForCleanup(lockfile);
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

function validateLockfileForCleanup(lockfile: Lockfile): void {
  validateSha256Digest("manifest_hash", lockfile.manifest_hash);
  for (const packageEntry of lockfile.package) {
    if (packageEntry.path.length === 0) {
      throw PrayError.parse("lockfile", "package path must not be empty");
    }
    validateSha256Digest("package tree_hash", packageEntry.tree_hash);
    validateSha256Digest("package artifact_hash", packageEntry.artifact_hash);
  }
}

function validateSha256Digest(field: string, value: string): void {
  if (/^sha256:[0-9a-f]{64}$/.test(value)) {
    return;
  }
  throw PrayError.parse("lockfile", `${field} must be a sha256 digest`);
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
