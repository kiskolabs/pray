import { PrayError } from "../errors.js";
import {
  optionalString,
  requireBoolean,
  requireNumber,
  requireRecord,
  requireString,
  requireStringArray,
} from "../validation.js";
import type {
  Lockfile,
  LockedPackage,
  LockedTarget,
  LockSource,
  ManagedSpanRecord,
} from "./types.js";

const CONTEXT = "lockfile";

export function parseLockfileValue(value: unknown): Lockfile {
  const record = requireRecord(value, CONTEXT);
  return {
    prayfile_lock: requireString(record.prayfile_lock, "prayfile_lock", CONTEXT),
    spec: requireString(record.spec, "spec", CONTEXT),
    generated_by: requireString(record.generated_by, "generated_by", CONTEXT),
    manifest_hash: requireString(record.manifest_hash, "manifest_hash", CONTEXT),
    environment: optionalString(record.environment, "environment", CONTEXT),
    source: parseLockSources(record.source),
    package: parseLockedPackages(record.package),
    target: parseLockedTargets(record.target),
    managed_span: parseManagedSpans(record.managed_span),
  };
}

function parseLockSources(value: unknown): LockSource[] {
  if (!Array.isArray(value)) {
    throwFieldTypeError("source", "array");
  }
  return value.map((entry, index) => parseLockSource(entry, index));
}

function parseLockSource(value: unknown, index: number): LockSource {
  const record = requireRecord(value, `${CONTEXT}.source[${index}]`);
  return {
    name: requireString(record.name, "name", `${CONTEXT}.source[${index}]`),
    kind: requireString(record.kind, "kind", `${CONTEXT}.source[${index}]`),
    url: requireString(record.url, "url", `${CONTEXT}.source[${index}]`),
    revision: optionalString(
      record.revision,
      "revision",
      `${CONTEXT}.source[${index}]`,
    ),
    host_key_fingerprint: optionalString(
      record.host_key_fingerprint,
      "host_key_fingerprint",
      `${CONTEXT}.source[${index}]`,
    ),
  };
}

function parseLockedPackages(value: unknown): LockedPackage[] {
  if (!Array.isArray(value)) {
    throwFieldTypeError("package", "array");
  }
  return value.map((entry, index) => parseLockedPackage(entry, index));
}

function parseLockedPackage(value: unknown, index: number): LockedPackage {
  const context = `${CONTEXT}.package[${index}]`;
  const record = requireRecord(value, context);
  return {
    name: requireString(record.name, "name", context),
    version: requireString(record.version, "version", context),
    source: optionalString(record.source, "source", context),
    path: requireString(record.path, "path", context),
    tree_hash: requireString(record.tree_hash, "tree_hash", context),
    artifact_hash: requireString(record.artifact_hash, "artifact_hash", context),
    artifact: requireString(record.artifact, "artifact", context),
    exports: requireStringArray(record.exports, "exports", context),
    dependencies: requireStringArray(record.dependencies, "dependencies", context),
    signer_fingerprint: optionalString(
      record.signer_fingerprint,
      "signer_fingerprint",
      context,
    ),
  };
}

function parseLockedTargets(value: unknown): LockedTarget[] {
  if (!Array.isArray(value)) {
    throwFieldTypeError("target", "array");
  }
  return value.map((entry, index) => parseLockedTarget(entry, index));
}

function parseLockedTarget(value: unknown, index: number): LockedTarget {
  const context = `${CONTEXT}.target[${index}]`;
  const record = requireRecord(value, context);
  return {
    name: requireString(record.name, "name", context),
    outputs: requireStringArray(record.outputs, "outputs", context),
  };
}

function parseManagedSpans(value: unknown): ManagedSpanRecord[] {
  if (!Array.isArray(value)) {
    throwFieldTypeError("managed_span", "array");
  }
  return value.map((entry, index) => parseManagedSpan(entry, index));
}

function parseManagedSpan(value: unknown, index: number): ManagedSpanRecord {
  const context = `${CONTEXT}.managed_span[${index}]`;
  const record = requireRecord(value, context);
  return {
    id: requireString(record.id, "id", context),
    target: requireString(record.target, "target", context),
    open_line: requireNumber(record.open_line, "open_line", context),
    close_line: requireNumber(record.close_line, "close_line", context),
    ideal_checksum: requireString(record.ideal_checksum, "ideal_checksum", context),
    package: requireString(record.package, "package", context),
    export: requireString(record.export, "export", context),
    source_checksum: requireString(
      record.source_checksum,
      "source_checksum",
      context,
    ),
    silenced: requireBoolean(record.silenced, "silenced", context),
  };
}

function throwFieldTypeError(field: string, expected: string): never {
  throw PrayError.parse(CONTEXT, `${field} must be a ${expected}`);
}
