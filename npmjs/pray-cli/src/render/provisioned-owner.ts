import { PrayError } from "../errors.js";

export interface PlannedProvisionedFile {
  path: string;
  source: string;
  package: string;
  export: string;
}

export function dedupeProvisionedOwners(
  files: PlannedProvisionedFile[],
): PlannedProvisionedFile[] {
  const result: PlannedProvisionedFile[] = [];
  for (const file of files) {
    const previous = result.at(-1);
    if (previous?.path === file.path) {
      if (
        previous.package !== file.package ||
        previous.export !== file.export ||
        previous.source !== file.source
      ) {
        throw PrayError.render(
          `destination \`${file.path}\` is claimed by both \`${previous.package}::${previous.export}\` and \`${file.package}::${file.export}\`; choose separate destinations`,
        );
      }
      continue;
    }
    result.push(file);
  }
  return result;
}
