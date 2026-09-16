import { PrayError } from "../errors.js";
import { parseCall, stringFromValue } from "../literal/call-parser.js";
import {
  bindLocalEntry,
  isLocalPathForm,
  targetMode,
  upsertLocal,
} from "./destination.js";
import type { Manifest, ManifestLocal } from "./types.js";

const PARSE_CONTEXT = "manifest";

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
