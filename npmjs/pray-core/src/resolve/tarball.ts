import {
  existsSync,
  mkdirSync,
  readFileSync,
  renameSync,
  rmSync,
} from "node:fs";
import { isAbsolute, join, resolve } from "node:path";
import { unpackPraypkg } from "../archive/praypkg.js";
import { PrayError } from "../errors.js";
import { sha256Prefixed } from "../hashing.js";
import { httpGet } from "../http/client.js";
import { findPrayspecFile } from "../package-spec/index.js";
import { MAX_ARCHIVE_TOTAL_BYTES } from "../resource-limits.js";
import type { ResolveOptions } from "./context.js";

export async function resolveTarballPackageRoot(
  projectRoot: string,
  tarball: string,
  options: ResolveOptions,
): Promise<string> {
  const artifactBytes = await readTarballBytes(projectRoot, tarball, options);
  if (artifactBytes.byteLength > MAX_ARCHIVE_TOTAL_BYTES) {
    throw PrayError.integrity(
      `package archive exceeds ${MAX_ARCHIVE_TOTAL_BYTES} bytes`,
    );
  }
  const cacheKey = sha256Prefixed(artifactBytes).slice("sha256:".length, 23);
  const cacheDirectory = join(
    projectRoot,
    ".pray",
    "cache",
    "tarball",
    cacheKey,
  );
  if (existsSync(cacheDirectory)) {
    try {
      findPrayspecFile(cacheDirectory);
      return cacheDirectory;
    } catch {
      // unpack below
    }
  }
  const stagingDirectory = `${cacheDirectory}.staging`;
  rmSync(stagingDirectory, { recursive: true, force: true });
  mkdirSync(stagingDirectory, { recursive: true });
  try {
    unpackPraypkg(artifactBytes, stagingDirectory);
    findPrayspecFile(stagingDirectory);
  } catch (error) {
    rmSync(stagingDirectory, { recursive: true, force: true });
    throw error;
  }
  mkdirSync(join(cacheDirectory, ".."), { recursive: true });
  rmSync(cacheDirectory, { recursive: true, force: true });
  renameSync(stagingDirectory, cacheDirectory);
  return cacheDirectory;
}

async function readTarballBytes(
  projectRoot: string,
  tarball: string,
  options: ResolveOptions,
): Promise<Buffer> {
  if (tarball.startsWith("http://") || tarball.startsWith("https://")) {
    if (options.offline) {
      throw PrayError.resolution(
        `tarball ${tarball} is not cached locally and offline mode is enabled`,
      );
    }
    return httpGet(tarball);
  }
  const path = localTarballPath(projectRoot, tarball);
  if (!existsSync(path)) {
    throw PrayError.resolution(`tarball missing at ${path}`);
  }
  return readFileSync(path);
}

function localTarballPath(projectRoot: string, tarball: string): string {
  const path = tarball.startsWith("file://")
    ? tarball.slice("file://".length)
    : tarball;
  return isAbsolute(path) ? path : resolve(projectRoot, path);
}
