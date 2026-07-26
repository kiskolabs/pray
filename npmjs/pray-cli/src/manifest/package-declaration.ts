import { PrayError } from "../errors.js";
import type { ManifestPackage } from "./types.js";

function formatStringKeywordList(values: string[]): string {
  return values.map((value) => `"${value}"`).join(", ");
}

export function formatPackageDeclaration(
  packageEntry: ManifestPackage,
): string {
  const parts = [`pray "${packageEntry.name}"`];
  if (packageEntry.constraint !== "*") {
    parts.push(`"${packageEntry.constraint}"`);
  }
  if (packageEntry.path) {
    parts.push(`path: "${packageEntry.path}"`);
  }
  if (packageEntry.source) {
    parts.push(`source: "${packageEntry.source}"`);
  }
  if (packageEntry.git) {
    parts.push(`git: "${packageEntry.git}"`);
  }
  if (packageEntry.tag) {
    parts.push(`tag: "${packageEntry.tag}"`);
  }
  if (packageEntry.rev) {
    parts.push(`rev: "${packageEntry.rev}"`);
  }
  if (packageEntry.tarball) {
    parts.push(`tarball: "${packageEntry.tarball}"`);
  }
  if (packageEntry.oci) {
    parts.push(`oci: "${packageEntry.oci}"`);
  }
  if (packageEntry.file) {
    parts.push(`file: "${packageEntry.file}"`);
  }
  if (packageEntry.exports.length > 0) {
    if (packageEntry.exports.length === 1) {
      parts.push(`export: "${packageEntry.exports[0]}"`);
    } else {
      parts.push(`exports: [${formatStringKeywordList(packageEntry.exports)}]`);
    }
  }
  if (packageEntry.targets.length > 0) {
    parts.push(`targets: [${formatStringKeywordList(packageEntry.targets)}]`);
  }
  if (packageEntry.features.length > 0) {
    parts.push(`features: [${formatStringKeywordList(packageEntry.features)}]`);
  }
  if (packageEntry.optional) {
    parts.push("optional: true");
  }
  return parts.join(", ");
}

export function replacePackageDeclaration(
  text: string,
  packageEntry: ManifestPackage,
): string {
  const name = packageEntry.name;
  const prefixes = [
    `pray "${name}"`,
    `pray '${name}'`,
    `use "${name}"`,
    `include "${name}"`,
    `agent "${name}"`,
    `agent '${name}'`,
    `package "${name}"`,
    `package '${name}'`,
  ];
  const lines = text.split(/\r?\n/);
  const index = lines.findIndex((line) => {
    const trimmed = line.trimStart();
    return prefixes.some((prefix) => trimmed.startsWith(prefix));
  });
  if (index === -1) {
    throw PrayError.manifest(`package ${name} not found in manifest`);
  }
  lines[index] = formatPackageDeclaration(packageEntry);
  let output = lines.join("\n");
  if (text.endsWith("\n") && !output.endsWith("\n")) {
    output += "\n";
  }
  return output;
}
