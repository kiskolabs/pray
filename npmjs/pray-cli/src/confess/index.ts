import { existsSync } from "node:fs";
import { PrayError } from "../errors.js";
import { sha256Prefixed } from "../hashing.js";
import { httpPost, joinUrl } from "../http/client.js";
import { readLockfile } from "../lockfile/index.js";
import { defaultLockfilePath } from "../lockfile/paths.js";
import type { Lockfile } from "../lockfile/types.js";
import { resolveProject } from "../resolve/project.js";

export interface ConfessionOptions {
  packageName?: string;
  fromLock?: string;
  version?: string;
  accepted?: boolean;
  rejected?: boolean;
  note?: string;
  url?: string;
}

export async function submitConfession(
  manifestPath: string,
  options: ConfessionOptions,
): Promise<void> {
  const project = await resolveProject(manifestPath);
  const lockfilePath = defaultLockfilePath(project.projectRoot);
  if (!existsSync(lockfilePath)) {
    throw PrayError.manifest("Prayfile.lock is required for confess");
  }
  const lockfile = readLockfile(lockfilePath);
  const submission = buildConfessionSubmission(project.manifestHash, lockfile, options);
  const serverUrl = options.url ?? defaultDistributionUrl(project);
  await httpPost(
    joinUrl(serverUrl, "v1/confessions"),
    "application/json",
    JSON.stringify(submission),
  );
  process.stdout.write(`confession submitted to ${serverUrl}\n`);
}

function buildConfessionSubmission(
  manifestHash: string,
  lockfile: Lockfile,
  options: ConfessionOptions,
): Record<string, unknown> {
  const span = options.fromLock
    ? lockfile.managed_span.find((entry) => entry.id === options.fromLock)
    : undefined;
  const packageName = options.packageName ?? span?.package;
  if (!packageName) {
    throw PrayError.manifest("confess requires a package name or --from-lock span id");
  }
  const locked = lockfile.package.find((entry) => entry.name === packageName);
  const accepted = options.accepted ?? !options.rejected;
  const payload = {
    package: packageName,
    version: options.version ?? locked?.version ?? "unknown",
    accepted,
    note: options.note ?? "",
    manifest_hash: manifestHash,
    lockfile_hash: lockfile.manifest_hash,
    submitted_at: new Date().toISOString(),
  };
  return {
    ...payload,
    signature: sha256Prefixed(JSON.stringify(payload)),
  };
}

function defaultDistributionUrl(
  project: Awaited<ReturnType<typeof resolveProject>>,
): string {
  const source = project.manifest.sources[0];
  if (!source || source.kind !== "registry") {
    throw PrayError.unsupported("confess requires --url or a registry source in Prayfile");
  }
  return source.url;
}
