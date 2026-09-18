import { join } from "node:path";
import { PrayError } from "../errors.js";
import type {
  Manifest,
  ManifestPackage,
  ManifestPublishRemote,
  ManifestSource,
} from "../manifest/types.js";
import { validateProjectRelativePath } from "../manifest/validate.js";
import { impliedSourceName } from "../resolve/package-root.js";

export interface PublishDestination {
  name: string;
  root?: string;
  server?: string;
  packages: string[];
}

export interface PublishCliDest {
  to: string[];
  roots: string[];
  servers: string[];
}

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

export function allowedPublishNames(
  manifest: Manifest,
  listed: string[],
): string[] {
  return listed.length === 0 ? pathOwnedPackageNames(manifest) : listed;
}

export function resolvePublishDestinations(
  remotes: ManifestPublishRemote[],
  cli: PublishCliDest,
  projectRoot: string,
): PublishDestination[] {
  if (cli.to.length > 0 && (cli.roots.length > 0 || cli.servers.length > 0)) {
    throw PrayError.usage(
      "publish --to cannot be combined with --root or --server",
    );
  }
  if (remotes.length === 0) {
    return resolveUndeclaredDestinations(cli);
  }
  return selectRemotes(remotes, cli, projectRoot).map((remote) => ({
    name: remote.name,
    root: remote.path ? join(projectRoot, remote.path) : undefined,
    server: remote.url,
    packages: remote.packages,
  }));
}

export function resolvePathRemote(
  remotes: ManifestPublishRemote[],
  to: string | undefined,
  root: string | undefined,
  projectRoot: string,
): string {
  const pathRemotes = remotes.filter((remote) => remote.path);
  if (to) {
    const remote = remotes.find((entry) => entry.name === to);
    if (!remote) {
      throw PrayError.usage(`unknown publish remote: ${to}`);
    }
    if (!remote.path) {
      throw PrayError.usage(
        `publish remote ${to} is a URL; yank, serve, and token need a path remote`,
      );
    }
    return join(projectRoot, remote.path);
  }
  if (remotes.length === 0) {
    if (!root) {
      throw PrayError.usage("requires --root PATH");
    }
    return root;
  }
  if (root) {
    const matched = remotes.find(
      (remote) =>
        remote.path !== undefined &&
        rootMatches(root, remote.path, projectRoot),
    );
    if (!matched) {
      throw PrayError.usage(`path ${root} is not a declared publish remote`);
    }
    return root;
  }
  if (pathRemotes.length === 1 && pathRemotes[0]?.path) {
    return join(projectRoot, pathRemotes[0].path);
  }
  throw PrayError.usage("say which path remote with --to NAME or --root PATH");
}

function resolveUndeclaredDestinations(
  cli: PublishCliDest,
): PublishDestination[] {
  if (cli.to.length > 0) {
    throw PrayError.usage(
      "Prayfile has no publish remotes; pass --root PATH or --server URL",
    );
  }
  if (cli.roots.length === 0 && cli.servers.length === 0) {
    throw PrayError.unsupported(
      "publish requires at least one --root PATH or --server URL",
    );
  }
  return [
    ...cli.roots.map((root) => ({
      name: root,
      root,
      packages: [] as string[],
    })),
    ...cli.servers.map((server) => ({
      name: server,
      server,
      packages: [] as string[],
    })),
  ];
}

function selectRemotes(
  remotes: ManifestPublishRemote[],
  cli: PublishCliDest,
  projectRoot: string,
): ManifestPublishRemote[] {
  if (cli.to.length > 0) {
    return cli.to.map((name) => {
      const remote = remotes.find((entry) => entry.name === name);
      if (!remote) {
        throw PrayError.usage(`unknown publish remote: ${name}`);
      }
      return remote;
    });
  }
  if (cli.roots.length === 0 && cli.servers.length === 0) {
    return remotes;
  }
  const selected: ManifestPublishRemote[] = [];
  for (const root of cli.roots) {
    const remote = remotes.find(
      (entry) =>
        entry.path !== undefined && rootMatches(root, entry.path, projectRoot),
    );
    if (!remote) {
      throw PrayError.usage(`path ${root} is not a declared publish remote`);
    }
    selected.push(remote);
  }
  for (const server of cli.servers) {
    const remote = remotes.find((entry) => entry.url === server);
    if (!remote) {
      throw PrayError.usage(
        `server ${server} is not a declared publish remote`,
      );
    }
    selected.push(remote);
  }
  return selected;
}

function rootMatches(
  cliRoot: string,
  remotePath: string,
  projectRoot: string,
): boolean {
  const declared = join(projectRoot, remotePath);
  if (cliRoot === declared || cliRoot === remotePath) {
    return true;
  }
  return stripDotSlash(cliRoot) === stripDotSlash(remotePath);
}

function stripDotSlash(value: string): string {
  return value
    .trim()
    .replace(/^\.\//, "")
    .replace(/\/$/, "")
    .replaceAll("\\", "/");
}

function isPublishUrl(url: string): boolean {
  return (
    url.startsWith("https://") ||
    url.startsWith("http://") ||
    url.startsWith("pray+ssh://") ||
    url.startsWith("ssh+pray://")
  );
}
