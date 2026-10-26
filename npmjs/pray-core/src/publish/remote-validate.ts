import { PrayError } from "../errors.js";
import type {
  Manifest,
  ManifestPackage,
  ManifestPublishRemote,
  ManifestSource,
} from "../manifest/types.js";
import { validateProjectRelativePath } from "../manifest/validate.js";
import { impliedSourceName } from "../resolve/implied-source.js";

/** Validate publish remote declarations on a parsed manifest. */
export function validatePublishRemotes(manifest: Manifest): void {
  const seen = new Set<string>();
  for (const remote of manifest.publishRemotes ?? []) {
    if (remote.name.trim().length === 0) {
      throw PrayError.parse("manifest", "publish requires a name");
    }
    if (seen.has(remote.name)) {
      throw PrayError.manifest(`duplicate publish remote: ${remote.name}`);
    }
    seen.add(remote.name);
    if (remote.path && remote.url) {
      throw PrayError.parse(
        "manifest",
        `publish "${remote.name}" must set path: or a URL, not both`,
      );
    }
    if (!remote.path && !remote.url) {
      throw PrayError.parse(
        "manifest",
        `publish "${remote.name}" requires path: or a URL`,
      );
    }
    if (remote.path) {
      validateProjectRelativePath(remote.path);
    }
    if (remote.url && !isPublishUrl(remote.url)) {
      throw PrayError.parse(
        "manifest",
        `publish "${remote.name}" URL must be https://, http://, pray+ssh://, or ssh+pray://`,
      );
    }
    validateRemotePackages(manifest, remote);
  }
}

function validateRemotePackages(
  manifest: Manifest,
  remote: ManifestPublishRemote,
): void {
  for (const packageName of remote.packages) {
    const packageEntry = manifest.packages.find(
      (entry) => entry.name === packageName,
    );
    if (!packageEntry) {
      throw PrayError.manifest(
        `publish "${remote.name}" lists unknown package ${packageName}`,
      );
    }
    if (!packageIsPathOwned(packageEntry, manifest.sources)) {
      throw PrayError.manifest(
        `publish "${remote.name}" lists ${packageName}, which is not a path package`,
      );
    }
  }
}

export function packageIsPathOwned(
  packageEntry: ManifestPackage,
  sources: ManifestSource[],
): boolean {
  if (packageEntry.path) {
    return true;
  }
  if (packageEntry.git || packageEntry.tarball || packageEntry.oci) {
    return false;
  }
  const map = new Map(sources.map((source) => [source.name, source]));
  try {
    const name = impliedSourceName(packageEntry, map);
    return name !== undefined && map.get(name)?.kind === "path";
  } catch {
    return false;
  }
}

export function pathOwnedPackageNames(manifest: Manifest): string[] {
  return manifest.packages
    .filter((entry) => packageIsPathOwned(entry, manifest.sources))
    .map((entry) => entry.name);
}

function isPublishUrl(url: string): boolean {
  return (
    url.startsWith("https://") ||
    url.startsWith("http://") ||
    url.startsWith("pray+ssh://") ||
    url.startsWith("ssh+pray://")
  );
}
