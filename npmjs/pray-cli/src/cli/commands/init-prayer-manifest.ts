import { PrayError } from "../../errors.js";
import {
  DEFAULT_PATH_SOURCE_DIRECTORY,
  DEFAULT_PATH_SOURCE_NAME,
} from "../../local-prayer.js";
import { defaultManifestPath } from "../../lockfile/paths.js";
import { parseManifest, readManifestText } from "../../manifest/index.js";
import type { Manifest, ManifestSource } from "../../manifest/types.js";
import { writeProjectFile } from "../../transaction/index.js";

export function parsePrayerInitArguments(argumentsList: string[]): {
  name?: string;
  directory?: string;
} {
  let name: string | undefined;
  let directory: string | undefined;
  for (let index = 0; index < argumentsList.length; index += 1) {
    const argument = argumentsList[index];
    if (argument === undefined) {
      continue;
    }
    if (argument === "--path") {
      const value = argumentsList[index + 1];
      if (!value || value.startsWith("-")) {
        throw PrayError.usage("--path requires a directory");
      }
      directory = value;
      index += 1;
    } else if (argument.startsWith("-")) {
      throw PrayError.unsupported(`unexpected prayer argument: ${argument}`);
    } else if (!name) {
      name = argument;
    } else {
      throw PrayError.unsupported(`unexpected prayer argument: ${argument}`);
    }
  }
  return { name, directory };
}

export function resolveLocalPrayerDirectory(requested?: string): {
  name: string;
  directory: string;
} {
  const manifest = parseManifest(readManifestText(defaultManifestPath()));
  const pathSources = pathSourcesFor(manifest);
  if (requested !== undefined) {
    const directory = requested.trim();
    if (directory.length === 0) {
      throw PrayError.usage("--path requires a directory");
    }
    const matching = pathSources.find((source) => source.url === directory);
    if (matching) {
      return { name: matching.name, directory: matching.url };
    }
    const unique = uniquePathSource(pathSources);
    if (unique) {
      throw PrayError.usage(`path source already uses ${unique.url}`);
    }
    const name = ensurePathSource(directory);
    return { name, directory };
  }
  if (pathSources.length === 0) {
    const name = ensurePathSource(DEFAULT_PATH_SOURCE_DIRECTORY);
    return { name, directory: DEFAULT_PATH_SOURCE_DIRECTORY };
  }
  const unique = uniquePathSource(pathSources);
  if (unique) {
    return { name: unique.name, directory: unique.url };
  }
  throw PrayError.usage("say which directory with --path");
}

export function declareLocalPrayer(packageName: string): void {
  const manifestPath = defaultManifestPath();
  const manifestText = readManifestText(manifestPath);
  const manifest = parseManifest(manifestText);
  if (
    manifest.packages.some((packageEntry) => packageEntry.name === packageName)
  ) {
    return;
  }
  const statement = `pray "${packageName}"`;
  const updated =
    insertIntoFirstCompose(manifestText, statement) ??
    insertTopLevelStatement(manifestText, statement);
  writeProjectFile(manifestPath, updated);
}

function pathSourcesFor(manifest: Manifest): ManifestSource[] {
  return manifest.sources.filter((source) => source.kind === "path");
}

function uniquePathSource(
  pathSources: ManifestSource[],
): ManifestSource | undefined {
  return pathSources.length === 1 ? pathSources[0] : undefined;
}

function ensurePathSource(directory: string): string {
  const manifestPath = defaultManifestPath();
  const manifestText = readManifestText(manifestPath);
  const manifest = parseManifest(manifestText);
  const existing = manifest.sources.find(
    (source) => source.kind === "path" && source.url === directory,
  );
  if (existing) {
    return existing.name;
  }
  const name = unusedSourceName(manifest, directory);
  const statement = `source "${name}", path: "${directory}"`;
  writeProjectFile(
    manifestPath,
    insertSourceStatement(manifestText, statement),
  );
  return name;
}

function unusedSourceName(manifest: Manifest, directory: string): string {
  const taken = new Set(manifest.sources.map((source) => source.name));
  if (!taken.has(DEFAULT_PATH_SOURCE_NAME)) {
    return DEFAULT_PATH_SOURCE_NAME;
  }
  const segments = directory
    .split(/[/\\]/)
    .filter((segment) => segment.length > 0);
  const fallback = segments.at(-1) ?? DEFAULT_PATH_SOURCE_NAME;
  if (taken.has(fallback)) {
    throw PrayError.manifest(`source name ${fallback} is already used`);
  }
  return fallback;
}

function insertSourceStatement(text: string, statement: string): string {
  const lines = text.split(/\r?\n/);
  let insertionIndex = 0;
  for (let index = lines.length - 1; index >= 0; index -= 1) {
    const line = lines[index];
    if (line?.trimStart().startsWith("source ")) {
      insertionIndex = index + 1;
      break;
    }
    if (index === 0) {
      const prayfile = lines.findIndex((line) =>
        line.trimStart().startsWith("prayfile "),
      );
      insertionIndex = prayfile >= 0 ? prayfile + 1 : 0;
    }
  }
  lines.splice(insertionIndex, 0, statement);
  return joinManifestLines(lines);
}

function insertTopLevelStatement(text: string, statement: string): string {
  const lines = text.split(/\r?\n/);
  const insertionIndex = lines.findIndex((line) => {
    const trimmed = line.trimStart();
    return trimmed.startsWith("local ") || trimmed.startsWith("render ");
  });
  lines.splice(
    insertionIndex >= 0 ? insertionIndex : lines.length,
    0,
    statement,
  );
  return joinManifestLines(lines);
}

function insertIntoFirstCompose(
  text: string,
  statement: string,
): string | undefined {
  const lines = text.split(/\r?\n/);
  const index = lines.findIndex((line) => {
    const trimmed = line.trimStart();
    return trimmed.startsWith("compose ") && trimmed.endsWith(" do");
  });
  if (index < 0) {
    return undefined;
  }
  const composeLine = lines[index];
  if (composeLine === undefined) {
    return undefined;
  }
  const indent = composeLine.match(/^\s*/)?.[0] ?? "";
  lines.splice(index + 1, 0, `${indent}  ${statement}`);
  return joinManifestLines(lines);
}

function joinManifestLines(lines: string[]): string {
  return `${lines.join("\n").replace(/\n*$/, "")}\n`;
}
