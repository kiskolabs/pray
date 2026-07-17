import { existsSync } from "node:fs";
import { join, resolve } from "node:path";
import { PrayError } from "../errors.js";
import {
  discoverDistributionRoot,
  type GitSourceCheckout,
  localGitSourceRoot,
  resolveDistributionRoot,
} from "../git/sources.js";
import type { Lockfile } from "../lockfile/types.js";
import type { ManifestPackage, ManifestSource } from "../manifest/types.js";
import {
  resolveLocalRegistryPackageRoot,
  resolveRegistryPackageRoot,
} from "../registry/index.js";
import { lockfilePreferredVersion, type ResolveOptions } from "./context.js";

export interface PackageRootResolution {
  root: string;
  signerFingerprint?: string;
  registryLatestVersion?: string;
}

export async function resolvePackageRoot(
  projectRoot: string,
  sources: Map<string, ManifestSource>,
  gitSources: Map<string, GitSourceCheckout>,
  declaration: ManifestPackage,
  lockfile: Lockfile | undefined,
  options: ResolveOptions,
): Promise<PackageRootResolution> {
  if (declaration.path) {
    return { root: resolve(projectRoot, declaration.path) };
  }

  if (declaration.source) {
    const source = sources.get(declaration.source);
    if (!source) {
      throw PrayError.resolution(`unknown source: ${declaration.source}`);
    }

    const registryOptions = {
      preferredVersion: lockfilePreferredVersion(
        lockfile,
        declaration.name,
        options,
      ),
      offline: options.offline,
    };

    if (source.kind === "path") {
      const slug = declaration.name.replaceAll("/", "-");
      return { root: resolve(projectRoot, source.url, slug) };
    }

    if (source.kind === "registry" || source.kind === "static index") {
      const resolved = await resolveRegistryPackageRoot(
        projectRoot,
        source.url,
        declaration,
        registryOptions,
      );
      return {
        root: resolved.root,
        signerFingerprint: resolved.signerFingerprint,
        registryLatestVersion: resolved.registryLatestVersion,
      };
    }

    if (source.kind === "pray_ssh") {
      throw PrayError.unsupported(
        "pray_ssh sources require SSH RPC support; use registry or git sources for now",
      );
    }

    if (source.kind === "git") {
      const checkout = gitSources.get(source.name);
      if (!checkout) {
        throw PrayError.resolution(
          `git source ${source.name} was not prepared`,
        );
      }
      const cloneUrl = source.url.replace(/^git\+/, "");
      const distributionRoot = resolveDistributionRoot(
        checkout.cacheDirectory,
        checkout.subdir,
      );
      const sourceKey = checkout.revision
        ? `${cloneUrl}@${checkout.revision}`
        : cloneUrl;
      const resolved = await resolveLocalRegistryPackageRoot(
        projectRoot,
        sourceKey,
        distributionRoot,
        declaration,
        registryOptions,
      );
      return {
        root: resolved.root,
        signerFingerprint: resolved.signerFingerprint,
        registryLatestVersion: resolved.registryLatestVersion,
      };
    }

    throw PrayError.unsupported(
      `source kind ${source.kind} not implemented yet`,
    );
  }

  if (declaration.git || declaration.tarball || declaration.oci) {
    throw PrayError.unsupported(
      "remote package sources are not implemented yet",
    );
  }

  const slug = declaration.name.replaceAll("/", "-");
  return { root: resolve(projectRoot, slug) };
}

export function vendoredPackageRoot(
  projectRoot: string,
  packageName: string,
  version: string,
): string | undefined {
  const path = join(
    projectRoot,
    ".pray",
    "vendor",
    packageName.replaceAll("/", "-"),
    version,
  );
  return existsSync(path) ? path : undefined;
}

export { discoverDistributionRoot, localGitSourceRoot };
