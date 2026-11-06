import { readdirSync } from "node:fs";
import { join } from "node:path";
import { PrayError } from "../errors.js";

export function findPrayspecFile(root: string): string {
  let found: string | undefined;
  for (const entry of readdirSync(root, { withFileTypes: true })) {
    if (!entry.isFile() || !entry.name.endsWith(".prayspec")) {
      continue;
    }
    if (found !== undefined) {
      throw PrayError.resolution(`multiple prayspec files found in ${root}`);
    }
    found = join(root, entry.name);
  }
  if (found === undefined) {
    throw PrayError.resolution(`no prayspec file found in ${root}`);
  }
  return found;
}
