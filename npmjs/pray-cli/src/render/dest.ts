import {
  closeSync,
  constants,
  ftruncateSync,
  mkdirSync,
  readFileSync,
  unlinkSync,
} from "node:fs";
import { dirname, resolve } from "node:path";
import { PrayError } from "../errors.js";
import { sha256Prefixed } from "../hashing.js";
import type { Lockfile, ProvisionedFileRecord } from "../lockfile/types.js";
import { validateDestinationPath } from "../manifest/validate.js";
import type { ResolvedProject } from "../resolve/types.js";
import {
  createBytes,
  destinationKind,
  openRegular,
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
    previousMap(previousLockfile).get(file.path.replaceAll("\\", "/")),
  );
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
      `refusing to overwrite \`${display}\`; it was provisioned and then edited`,
    );
  }
  throw PrayError.render(
    `refusing to overwrite \`${display}\`; it already exists and is not the expected provisioned file`,
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
    const onDisk = readFileSync(descriptor);
    if (onDisk.equals(bytes)) return;
    if (sha256Prefixed(onDisk) !== authorizedHash) {
      throw PrayError.render(
        `refusing to overwrite \`${display}\`; it was provisioned and then edited`,
      );
    }
    ftruncateSync(descriptor, 0);
    writeAll(descriptor, bytes);
  } finally {
    closeSync(descriptor);
  }
}
