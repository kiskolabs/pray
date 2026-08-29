import { PrayError } from "../../errors.js";
import {
  previewRemoteUpdates,
  updateWithManifestConstraints,
} from "./update-core.js";
import { updateLatestCommand } from "./update-latest.js";

export interface UpdateFlags {
  packageName?: string;
  major: boolean;
  latest: boolean;
  dryRun: boolean;
  json: boolean;
}

export function parseUpdateArguments(argumentsList: string[]): UpdateFlags {
  let packageName: string | undefined;
  let major = false;
  let latest = false;
  let dryRun = false;
  let json = false;
  for (const argument of argumentsList) {
    switch (argument) {
      case "--major":
        major = true;
        break;
      case "--latest":
        latest = true;
        break;
      case "--dry-run":
        dryRun = true;
        break;
      case "--json":
        json = true;
        break;
      default:
        if (argument.startsWith("--")) {
          throw PrayError.unsupported(`unknown update flag: ${argument}`);
        }
        if (packageName) {
          throw PrayError.unsupported(
            `unexpected update argument: ${argument}`,
          );
        }
        packageName = argument;
    }
  }
  return { packageName, major, latest, dryRun, json };
}

export async function runUpdateCommand(argumentsList: string[]): Promise<void> {
  const flags = parseUpdateArguments(argumentsList);
  if (flags.major && flags.latest) {
    throw PrayError.unsupported("use either --major or --latest, not both");
  }
  if (flags.major) {
    if (!flags.packageName) {
      throw PrayError.unsupported("major updates require a package name");
    }
    if (flags.dryRun) {
      throw PrayError.unsupported(
        "major updates are not supported with --dry-run",
      );
    }
    await updateLatestCommand(flags.packageName, flags.json, false);
    return;
  }
  if (flags.latest) {
    if (flags.dryRun && flags.json) {
      throw PrayError.unsupported("--json is not supported with --dry-run");
    }
    await updateLatestCommand(flags.packageName, flags.json, flags.dryRun);
    return;
  }
  if (flags.dryRun) {
    await previewRemoteUpdates(flags.packageName, flags.json);
    return;
  }
  await updateWithManifestConstraints(flags.packageName, flags.json, []);
}

export { previewRemoteUpdates } from "./update-core.js";
