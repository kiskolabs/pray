import { PrayError } from "../errors.js";
import { parseCall, stringFromValue } from "../literal/call-parser.js";
import {
  bindLocalEntry,
  isLocalPathForm,
  targetEntries,
  targetMode,
  upsertLocal,
  upsertPackageAt,
} from "./destination.js";
import type {
  Manifest,
  ManifestLocal,
  ManifestPackage,
  ManifestTarget,
} from "./types.js";

const PARSE_CONTEXT = "manifest";

export class ParserIndexes {
  private readonly keysByTarget = new WeakMap<ManifestTarget, Set<string>>();
  private readonly packageIndices = new Map<string, number>();

  upsertPackage(manifest: Manifest, packageEntry: ManifestPackage): void {
    const existingIndex = this.packageIndices.get(packageEntry.name);
    upsertPackageAt(manifest, packageEntry, existingIndex);
    if (existingIndex === undefined) {
      this.packageIndices.set(packageEntry.name, manifest.packages.length - 1);
    }
  }

  bindPackage(target: ManifestTarget, packageName: string): void {
    const keys = this.keysFor(target);
    const key = `package:${packageName}`;
    if (!keys.has(key)) {
      const entries = targetEntries(target);
      entries.push({ kind: "package", name: packageName });
      target.entries = entries;
      keys.add(key);
    }
  }

  private keysFor(target: ManifestTarget): Set<string> {
    const existing = this.keysByTarget.get(target);
    if (existing) {
      return existing;
    }
    const keys = new Set(
      (target.entries ?? []).map((entry) =>
        entry.kind === "package"
          ? `package:${entry.name}`
          : `local:${entry.path}`,
      ),
    );
    this.keysByTarget.set(target, keys);
    return keys;
  }
}

export function localIsComposeEmbed(
  manifest: Manifest,
  local: ManifestLocal,
): boolean {
  if (local.file) {
    return false;
  }
  return !manifest.targets.some(
    (target) =>
      targetMode(target) === "tree" &&
      (target.entries ?? []).some(
        (entry) => entry.kind === "local" && entry.path === local.path,
      ),
  );
}

export function tryApplyFileBlockPray(
  _manifest: Manifest,
  rest: string,
  _filePath: string,
): boolean {
  const { values, keywords } = parseCall(rest);
  if (values.length !== 1 || keywords.size !== 0) {
    return false;
  }
  const first = stringFromValue(values[0]!, PARSE_CONTEXT);
  if (!isLocalPathForm(first)) {
    return false;
  }
  throw PrayError.parse(PARSE_CONTEXT, "file: requires a package");
}

export function tryApplyLocalPray(
  manifest: Manifest,
  path: string,
  fileDest: string | undefined,
  otherPackageSignal: boolean,
  destinationIndex: number | undefined,
): boolean {
  if (!isLocalPathForm(path)) {
    return false;
  }
  if (fileDest !== undefined) {
    if (otherPackageSignal) {
      return false;
    }
    throw PrayError.parse(PARSE_CONTEXT, "a local file cannot use file:");
  }
  if (otherPackageSignal) {
    return false;
  }
  if (destinationIndex === undefined) {
    throw PrayError.parse(
      PARSE_CONTEXT,
      "local pray paths are only valid inside compose",
    );
  }
  const mode = targetMode(manifest.targets[destinationIndex]!);
  if (mode !== "compose") {
    throw PrayError.parse(
      PARSE_CONTEXT,
      "local pray paths are only valid inside compose",
    );
  }
  const local: ManifestLocal = {
    path,
    position: "after",
    optional: false,
    bound: true,
  };
  bindLocalEntry(manifest.targets[destinationIndex]!, local.path);
  upsertLocal(manifest, local);
  return true;
}
