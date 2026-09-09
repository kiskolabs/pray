import {
  closeSync,
  constants,
  ftruncateSync,
  mkdirSync,
  unlinkSync,
} from "node:fs";
import { dirname, resolve } from "node:path";
import { PrayError } from "../errors.js";
import { sha256Prefixed } from "../hashing.js";
import type { Lockfile, ProvisionedFileRecord } from "../lockfile/types.js";
import { validateDestinationPath } from "../manifest/validate.js";
import type { ResolvedProject } from "../resolve/types.js";
import { replaceProjectFile } from "../transaction/hooks.js";
import {
  createBytes,
  destinationKind,
  openRegular,
  readDestinationBytes,
  readRegularBytes,
  writeAll,
} from "./destination-io.js";
import { ensureSafeDestinationAncestors } from "./path-guard.js";
import {
  expectedProvisionedBytes,
  type PlannedProvisionedFile,
  plannedProvisionedFiles,
} from "./provisioned.js";

export function provisionedLockRecords(
  project: ResolvedProject,
): ProvisionedFileRecord[] {
  return plannedProvisionedFiles(project).map((file) => {
    const expected = expectedProvisionedBytes(
      file.source,
      project.manifest.symbols ?? {},
    );
    return {
      path: file.path.replaceAll("\\", "/"),
      content_hash: sha256Prefixed(expected),
      package: file.package,
      export: file.export,
    };
  });
}

export function materializeProvisionedExports(
  project: ResolvedProject,
  previousLockfile?: Lockfile,
): void {
  const planned = plannedProvisionedFiles(project);
  const previous = previousMap(previousLockfile);
  const plannedPaths = new Set(
    planned.map((file) => file.path.replaceAll("\\", "/")),
  );
  for (const file of planned) {
    writeLeaf(project, file, previous);
  }
  if (previousLockfile) {
    pruneDropped(project, previousLockfile, plannedPaths);
  }
}

export type ProvisionedDestinationStatus = "write" | "unchanged" | "update";

export function provisionedDestinationStatus(
  project: ResolvedProject,
  file: PlannedProvisionedFile,
  previousLockfile?: Lockfile,
  previous = previousMap(previousLockfile),
): ProvisionedDestinationStatus {
  validateDestinationPath(file.path);
  ensureSafeDestinationAncestors(project.projectRoot, file.path, file.path);
  const expected = expectedProvisionedBytes(
    file.source,
    project.manifest.symbols ?? {},
  );
  return classifyDestination(
    resolve(project.projectRoot, file.path),
    file.path,
    expected,
    previous.get(file.path.replaceAll("\\", "/")),
  );
}

export function provisionedDestinationStatuses(
  project: ResolvedProject,
  previousLockfile?: Lockfile,
): Array<[PlannedProvisionedFile, ProvisionedDestinationStatus]> {
  const statuses: Array<
    [PlannedProvisionedFile, ProvisionedDestinationStatus]
  > = [];
  const errors: string[] = [];
  let omitted = 0;
  let diagnosticBytes = 0;
  const previous = previousMap(previousLockfile);
  for (const file of plannedProvisionedFiles(project)) {
    try {
      statuses.push([
        file,
        provisionedDestinationStatus(project, file, previousLockfile, previous),
      ]);
    } catch (error) {
      if (!(error instanceof PrayError) || error.kind !== "render") throw error;
      const message = `${error.message} (package \`${file.package}\`, export \`${file.export}\`)`;
      const size = Buffer.byteLength(message);
      if (errors.length < 100 && diagnosticBytes + size < 60 * 1024) {
        diagnosticBytes += size;
        errors.push(message);
      } else {
        omitted++;
      }
    }
  }
  if (omitted > 0)
    errors.push(
      `${omitted} additional destination conflicts omitted; resolve the listed paths and run \`pray plan\` again`,
    );
  if (errors.length > 0) throw PrayError.render(errors.join("\n"));
  return statuses;
}

function previousMap(lockfile?: Lockfile): Map<string, ProvisionedFileRecord> {
  const records = new Map<string, ProvisionedFileRecord>();
  for (const record of lockfile?.provisioned ?? []) {
    records.set(record.path, record);
  }
  return records;
}

function writeLeaf(
  project: ResolvedProject,
  file: PlannedProvisionedFile,
  previous: Map<string, ProvisionedFileRecord>,
): void {
  validateDestinationPath(file.path);
  ensureSafeDestinationAncestors(project.projectRoot, file.path, file.path);
  const destination = resolve(project.projectRoot, file.path);
  const expected = expectedProvisionedBytes(
    file.source,
    project.manifest.symbols ?? {},
  );
  const record = previous.get(file.path.replaceAll("\\", "/"));
  const status = classifyDestination(destination, file.path, expected, record);
  if (status === "write") {
    mkdirSync(dirname(destination), { recursive: true });
    ensureSafeDestinationAncestors(project.projectRoot, file.path, file.path);
    createBytes(destination, file.path, expected);
    return;
  }
  if (status === "unchanged") {
    return;
  }
  if (!record) {
    throw PrayError.render(`missing lock ownership for \`${file.path}\``);
  }
  ensureSafeDestinationAncestors(project.projectRoot, file.path, file.path);
  updateBytes(destination, file.path, expected, record.content_hash);
}

function classifyDestination(
  destination: string,
  display: string,
  expected: Buffer,
  record?: ProvisionedFileRecord,
): ProvisionedDestinationStatus {
  const kind = destinationKind(destination);
  if (kind === "missing") return "write";
  if (kind === "symlink") {
    throw PrayError.render(
      `refusing to write \`${display}\` because it is a symbolic link`,
    );
  }
  if (kind !== "regular") {
    throw PrayError.render(
      `refusing to write \`${display}\`; destination is not a regular file`,
    );
  }
  const onDisk = readRegularBytes(destination, display);
  if (onDisk.equals(expected)) return "unchanged";
  if (record && sha256Prefixed(onDisk) === record.content_hash) return "update";
  if (record) {
    throw PrayError.render(
      `refusing to overwrite \`${display}\`; it was written by pray and then edited. Inspect your changes and move the file aside, then run \`pray install\``,
    );
  }
  throw PrayError.render(
    `refusing to overwrite \`${display}\`; its existing contents differ from this package. Inspect the file and move it aside, then run \`pray install\`. If an older pray wrote it, restore the original Prayfile and package version, run \`pray install\`, then retry the update`,
  );
}

function pruneDropped(
  project: ResolvedProject,
  previous: Lockfile,
  plannedPaths: Set<string>,
): void {
  for (const record of previous.provisioned ?? []) {
    validateDestinationPath(record.path);
    if (plannedPaths.has(record.path)) {
      continue;
    }
    const destination = resolve(project.projectRoot, record.path);
    ensureSafeDestinationAncestors(
      project.projectRoot,
      record.path,
      record.path,
    );
    if (destinationKind(destination) !== "regular") {
      continue;
    }
    const onDisk = readRegularBytes(destination, record.path);
    if (sha256Prefixed(onDisk) === record.content_hash) {
      ensureSafeDestinationAncestors(
        project.projectRoot,
        record.path,
        record.path,
      );
      if (!replaceProjectFile(destination, onDisk, undefined))
        unlinkSync(destination);
    }
  }
}

function updateBytes(
  path: string,
  display: string,
  bytes: Buffer,
  authorizedHash: string,
): void {
  const descriptor = openRegular(path, display, constants.O_RDWR);
  try {
    const onDisk = readDestinationBytes(descriptor, display);
    if (onDisk.equals(bytes)) return;
    if (sha256Prefixed(onDisk) !== authorizedHash) {
      throw PrayError.render(
        `refusing to overwrite \`${display}\`; it was written by pray and then edited. Inspect your changes and move the file aside, then run \`pray install\``,
      );
    }
    if (replaceProjectFile(path, onDisk, bytes)) return;
    ftruncateSync(descriptor, 0);
    writeAll(descriptor, bytes);
  } finally {
    closeSync(descriptor);
  }
}
