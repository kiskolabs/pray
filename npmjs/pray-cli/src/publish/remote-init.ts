import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { parseManifest } from "../manifest/index.js";

export function maybeDeclarePublishRemote(
  projectRoot: string,
  distributionRoot: string,
): void {
  const manifestPath = join(projectRoot, "Prayfile");
  if (!existsSync(manifestPath)) {
    return;
  }
  const text = readFileSync(manifestPath, "utf8");
  const manifest = parseManifest(text);
  if ((manifest.publishRemotes ?? []).length > 0) {
    return;
  }
  const relative = relativeDistributionPath(projectRoot, distributionRoot);
  const statement = `publish "prayers", path: "${relative}"`;
  const lines = text.split("\n");
  let insertion = lastMatchingIndex(lines, (line) =>
    line.trimStart().startsWith("source "),
  );
  if (insertion === -1) {
    insertion = lines.findIndex((line) =>
      line.trimStart().startsWith("prayfile "),
    );
  }
  insertion = insertion === -1 ? 1 : insertion + 1;
  lines.splice(insertion, 0, statement);
  let updated = lines.join("\n");
  if (!updated.endsWith("\n")) {
    updated += "\n";
  }
  writeFileSync(manifestPath, updated, "utf8");
}

function lastMatchingIndex(
  lines: string[],
  match: (line: string) => boolean,
): number {
  for (let index = lines.length - 1; index >= 0; index -= 1) {
    const line = lines[index];
    if (line !== undefined && match(line)) {
      return index;
    }
  }
  return -1;
}

function relativeDistributionPath(
  projectRoot: string,
  distributionRoot: string,
): string {
  if (distributionRoot === projectRoot) {
    return ".";
  }
  const prefix = projectRoot.endsWith("/") ? projectRoot : `${projectRoot}/`;
  if (distributionRoot.startsWith(prefix)) {
    return distributionRoot.slice(prefix.length).replaceAll("\\", "/");
  }
  return "prayers";
}
