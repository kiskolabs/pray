import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { parse } from "smol-toml";
import { PrayError } from "../errors.js";
import { sha256Prefixed } from "../hashing.js";
import type { ManifestSource, ManifestTarget } from "../manifest/types.js";
import type { ResolvedPackage } from "../resolve/types.js";
import type { RenderedTarget } from "../render/types.js";
import {
  canonicalLockfile,
  GENERATED_BY,
  type Lockfile,
  type LockSource,
} from "./types.js";
import { parseLockfileValue } from "./parse.js";
import { normalizeLockfileArtifact, relativeLockfilePath } from "./paths.js";
import { serializeLockfileText } from "./serialize.js";

export function parseLockfile(text: string): Lockfile {
  try {
    return parseLockfileValue(parse(text));
  } catch (error) {
    if (error instanceof PrayError) {
      throw error;
    }
    const message = error instanceof Error ? error.message : String(error);
    throw PrayError.parse("lockfile", message);
  }
}

export function readLockfile(path: string): Lockfile {
  try {
    const text = readFileSync(path, "utf8");
    return parseLockfile(text);
  } catch (error) {
    if (error instanceof PrayError) {
      throw error;
    }
    const message = error instanceof Error ? error.message : String(error);
    throw PrayError.parse("lockfile", message);
  }
}

export function lockfilesEquivalent(left: Lockfile, right: Lockfile): boolean {
  return serializeLockfile(left) === serializeLockfile(right);
}

export function serializeLockfile(lockfile: Lockfile): string {
  return serializeLockfileText(lockfile);
}

export function writeLockfile(path: string, lockfile: Lockfile): void {
  writeFileSync(path, serializeLockfile(lockfile), "utf8");
}

export function writeLockfileIfChanged(path: string, lockfile: Lockfile): void {
  const serialized = serializeLockfile(lockfile);
  if (existsSync(path)) {
    const existing = readFileSync(path, "utf8");
    if (existing === serialized) {
      return;
    }
  }
  writeFileSync(path, serialized, "utf8");
}

export function lockfileHash(lockfile: Lockfile): string {
  return sha256Prefixed(serializeLockfile(lockfile));
}

export function buildLockfile(input: {
  manifestHash: string;
  environment?: string;
  projectRoot: string;
  manifestSources: ManifestSource[];
  manifestTargets: ManifestTarget[];
  rendered: RenderedTarget[];
  packages: ResolvedPackage[];
  sourceRevisions?: Map<string, string>;
  sourceHostKeys?: Map<string, string>;
}): Lockfile {
  const sourceRevisions = input.sourceRevisions ?? new Map();
  const sourceHostKeys = input.sourceHostKeys ?? new Map();
  return canonicalLockfile({
    prayfile_lock: "1",
    spec: "0.1",
    generated_by: GENERATED_BY,
    manifest_hash: input.manifestHash,
    ...(input.environment ? { environment: input.environment } : {}),
    source: input.manifestSources.map(
      (source): LockSource => ({
        name: source.name,
        kind: source.kind,
        url: source.url,
        ...(sourceRevisions.has(source.name)
          ? { revision: sourceRevisions.get(source.name) }
          : {}),
        ...(sourceHostKeys.has(source.name)
          ? { host_key_fingerprint: sourceHostKeys.get(source.name) }
          : {}),
      }),
    ),
    package: input.packages.map((packageEntry) => ({
      name: packageEntry.declaration.name,
      version: packageEntry.spec.version,
      ...(packageEntry.declaration.source
        ? { source: packageEntry.declaration.source }
        : {}),
      path: relativeLockfilePath(input.projectRoot, packageEntry.root),
      tree_hash: packageEntry.treeHash,
      artifact_hash: packageEntry.artifactHash,
      artifact: normalizeLockfileArtifact(
        input.projectRoot,
        packageEntry.artifact,
        packageEntry.root,
      ),
      exports: packageEntry.selectedExports,
      dependencies: packageEntry.spec.dependencies.map(
        (dependency) => dependency.name,
      ),
      ...(packageEntry.signerFingerprint
        ? { signer_fingerprint: packageEntry.signerFingerprint }
        : {}),
    })),
    target: input.manifestTargets.map((target) => ({
      name: target.name,
      outputs: target.outputs,
    })),
    managed_span: input.rendered.flatMap((target) => target.managedSpans),
  });
}

export { canonicalLockfile, type Lockfile } from "./types.js";
