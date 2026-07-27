import { formatPackageDeclaration } from "./package-declaration.js";
import type {
  DestinationEntry,
  Manifest,
  ManifestPackage,
  ManifestSource,
  ManifestTarget,
} from "./types.js";
import { defaultRenderPolicy, type RenderPolicy } from "./types.js";

export function targetHasExtras(target: ManifestTarget): boolean {
  return (
    target.commands.length > 0 ||
    target.rules.length > 0 ||
    target.maxBytes !== undefined
  );
}

export function serializeRecommended(manifest: Manifest): string {
  const lines: string[] = [`prayfile "${manifest.prayfileVersion}"`];

  if (manifest.sources.length > 0) {
    lines.push("");
    for (const source of manifest.sources) {
      lines.push(formatSource(source));
    }
  }

  const symbolEntries = Object.entries(manifest.symbols ?? {}).sort(
    ([left], [right]) => left.localeCompare(right),
  );
  if (symbolEntries.length > 0) {
    lines.push("");
    lines.push("pray do");
    for (const [key, value] of symbolEntries) {
      lines.push(`  ${key} "${value}"`);
    }
    lines.push("end");
  }

  for (const target of manifest.targets) {
    if (!target.scoped) {
      continue;
    }
    lines.push("");
    if (target.mode === "compose") {
      const path = target.outputs[0] ?? "";
      lines.push(`compose "${path}" do`);
      for (const entry of target.entries ?? []) {
        lines.push(`  ${formatDestinationEntry(entry, manifest)}`);
      }
      lines.push("end");
    } else if (target.mode === "tree") {
      const path = target.skills[0] ?? "";
      lines.push(`tree "${path}" do`);
      for (const entry of target.entries ?? []) {
        if (entry.kind === "package") {
          const packageEntry = findPackage(manifest, entry.name);
          if (packageEntry) {
            lines.push(`  ${formatPackageDeclaration(packageEntry)}`);
          }
        }
      }
      lines.push("end");
    }
  }

  const filePackages = manifest.packages.filter((entry) => entry.file);
  if (filePackages.length > 0) {
    lines.push("");
    for (const packageEntry of filePackages) {
      lines.push(formatPackageDeclaration(packageEntry));
    }
  }

  const unbound = manifest.packages.filter(
    (entry) => !entry.bound && !entry.file && entry.groups.length === 0,
  );
  if (unbound.length > 0) {
    lines.push("");
    for (const packageEntry of unbound) {
      lines.push(formatPackageDeclaration(packageEntry));
    }
  }

  for (const [groupNames, packages] of groupedPackages(manifest)) {
    lines.push("");
    lines.push(`group ${groupNames.map((name) => `:${name}`).join(", ")} do`);
    for (const packageEntry of packages) {
      lines.push(`  ${formatPackageDeclaration(packageEntry)}`);
    }
    lines.push("end");
  }

  for (const target of manifest.targets) {
    if (target.scoped || !targetHasExtras(target)) {
      continue;
    }
    lines.push("");
    lines.push(`target :${target.name} do`);
    for (const command of target.commands) {
      lines.push(`  commands "${command}"`);
    }
    for (const rule of target.rules) {
      lines.push(`  rules "${rule}"`);
    }
    if (target.maxBytes !== undefined) {
      lines.push(`  max_bytes ${target.maxBytes}`);
    }
    lines.push("end");
  }

  if (!renderPolicyEquals(manifest.render, defaultRenderPolicy())) {
    lines.push("");
    lines.push(
      `render mode: :${manifest.render.mode}, conflict: :${manifest.render.conflict}, churn: :${manifest.render.churn}`,
    );
  }

  lines.push("");
  return lines.join("\n");
}

function renderPolicyEquals(left: RenderPolicy, right: RenderPolicy): boolean {
  return (
    left.mode === right.mode &&
    left.conflict === right.conflict &&
    left.churn === right.churn &&
    left.header === right.header &&
    left.sectionMarkers === right.sectionMarkers &&
    left.lineEndings === right.lineEndings
  );
}

function formatSource(source: ManifestSource): string {
  const parts = [`source "${source.name}"`];
  if (source.kind === "path") {
    parts.push(`path: "${source.url}"`);
  } else if (source.kind === "git") {
    const url = source.url.startsWith("git+")
      ? source.url.slice("git+".length)
      : source.url;
    parts.push(`git: "${url}"`);
  } else {
    parts.push(`"${source.url}"`);
  }
  if (source.subdir) {
    parts.push(`distribution: "${source.subdir}"`);
  }
  if (source.tag) {
    parts.push(`tag: "${source.tag}"`);
  }
  if (source.rev) {
    parts.push(`rev: "${source.rev}"`);
  }
  return parts.join(", ");
}

function formatDestinationEntry(
  entry: DestinationEntry,
  manifest: Manifest,
): string {
  if (entry.kind === "local") {
    return `pray "${entry.path}"`;
  }
  const packageEntry = findPackage(manifest, entry.name);
  return packageEntry
    ? formatPackageDeclaration(packageEntry)
    : `pray "${entry.name}"`;
}

function findPackage(
  manifest: Manifest,
  name: string,
): ManifestPackage | undefined {
  return manifest.packages.find((entry) => entry.name === name);
}

function groupedPackages(
  manifest: Manifest,
): Array<[string[], ManifestPackage[]]> {
  const groups = new Map<
    string,
    { names: string[]; packages: ManifestPackage[] }
  >();
  for (const packageEntry of manifest.packages) {
    if (packageEntry.groups.length === 0) {
      continue;
    }
    const key = JSON.stringify(packageEntry.groups);
    const bucket = groups.get(key);
    if (bucket) {
      bucket.packages.push(packageEntry);
    } else {
      groups.set(key, { names: packageEntry.groups, packages: [packageEntry] });
    }
  }
  return [...groups.values()]
    .sort((left, right) => compareStringArrays(left.names, right.names))
    .map((bucket) => [bucket.names, bucket.packages]);
}

function compareStringArrays(left: string[], right: string[]): number {
  const length = Math.min(left.length, right.length);
  for (let index = 0; index < length; index += 1) {
    const comparison = left[index]!.localeCompare(right[index]!);
    if (comparison !== 0) {
      return comparison;
    }
  }
  return left.length - right.length;
}
