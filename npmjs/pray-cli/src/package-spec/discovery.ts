import { readdirSync } from "node:fs";
import { join } from "node:path";
import { PrayError } from "../errors.js";

export function findPrayspecFile(root: string): string {
  const entries = readdirSync(root, { withFileTypes: true });
  const prayspecFiles = entries
    .filter((entry) => entry.isFile() && entry.name.endsWith(".prayspec"))
    .map((entry) => join(root, entry.name));

  if (prayspecFiles.length === 1) {
    const file = prayspecFiles[0];
    if (file === undefined) {
      throw PrayError.resolution(`no prayspec file found in ${root}`);
    }
    return file;
  }
  if (prayspecFiles.length === 0) {
    throw PrayError.resolution(`no prayspec file found in ${root}`);
  }
  throw PrayError.resolution(`multiple prayspec files found in ${root}`);
}
