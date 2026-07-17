import { writeFileSync } from "node:fs";
import { PrayError } from "../errors.js";
import {
  formatPackageDeclaration,
  parseManifest,
  readManifestText,
} from "../manifest/index.js";
import type { ManifestPackage } from "../manifest/types.js";

export function insertManifestStatement(
  manifestText: string,
  statement: string,
): string {
  const lines = manifestText.split(/\r?\n/);
  const versionIndex = lines.findIndex((line) =>
    line.trim().startsWith("prayfile "),
  );
  const insertAt = versionIndex >= 0 ? versionIndex + 1 : 0;
  lines.splice(insertAt, 0, statement);
  return `${lines.join("\n").replace(/\n*$/, "")}\n`;
}

export function removeManifestStatement(
  manifestText: string,
  packageName: string,
): string {
  const prefix = `agent "${packageName}"`;
  const lines = manifestText.split(/\r?\n/).filter((line) => {
    const trimmed = line.trim();
    return !trimmed.startsWith(prefix);
  });
  return `${lines.join("\n").replace(/\n*$/, "")}\n`;
}

export function addPackageToManifest(
  manifestPath: string,
  name: string,
  options: { constraint?: string; path?: string; source?: string } = {},
): void {
  const manifestText = readManifestText(manifestPath);
  const manifest = parseManifest(manifestText);
  if (manifest.packages.some((packageEntry) => packageEntry.name === name)) {
    throw PrayError.manifest(`package ${name} already exists`);
  }
  const declaration: ManifestPackage = {
    name,
    constraint: options.constraint ?? "*",
    exports: [],
    targets: [],
    features: [],
    groups: [],
    optional: false,
    ...(options.path ? { path: options.path } : {}),
    ...(options.source ? { source: options.source } : {}),
  };
  const updated = insertManifestStatement(
    manifestText,
    formatPackageDeclaration(declaration),
  );
  writeFileSync(manifestPath, updated, "utf8");
}

export function removePackageFromManifest(
  manifestPath: string,
  name: string,
): void {
  const manifestText = readManifestText(manifestPath);
  const manifest = parseManifest(manifestText);
  if (!manifest.packages.some((packageEntry) => packageEntry.name === name)) {
    throw PrayError.manifest(`package ${name} not found`);
  }
  writeFileSync(
    manifestPath,
    removeManifestStatement(manifestText, name),
    "utf8",
  );
}
