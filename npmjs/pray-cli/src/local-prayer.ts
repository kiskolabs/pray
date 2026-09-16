import { PrayError } from "./errors.js";

export const DEFAULT_LOCAL_PRAYER_NAME = "project";
export const DEFAULT_PATH_SOURCE_NAME = "local";
export const DEFAULT_PATH_SOURCE_DIRECTORY = "prayers";

export function validateLocalPrayerName(name: string): string {
  const trimmed = name.trim();
  if (trimmed.length === 0) {
    throw PrayError.usage("prayer name is missing");
  }
  if (
    trimmed.includes("/") ||
    trimmed.includes("\\") ||
    trimmed === "." ||
    trimmed === ".."
  ) {
    throw PrayError.usage("prayer name must be a single folder name");
  }
  if (isReservedDistributionLayoutName(trimmed)) {
    throw PrayError.usage(`${trimmed} is reserved for the distribution layout`);
  }
  return trimmed;
}

export function pathSourcePackageDirectory(
  sourceName: string,
  packageName: string,
): string {
  const prefix = `${sourceName}/`;
  if (packageName.startsWith(prefix)) {
    const rest = packageName.slice(prefix.length);
    if (rest.length > 0 && !rest.includes("/") && !rest.includes("\\")) {
      return rest;
    }
  }
  return packageName.replaceAll("/", "-").replaceAll("\\", "-");
}

function isReservedDistributionLayoutName(name: string): boolean {
  return /^v[0-9]+$/.test(name);
}
