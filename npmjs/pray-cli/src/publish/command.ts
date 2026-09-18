import { PrayError } from "../errors.js";
import {
  defaultManifestPath,
  projectRootFromManifest,
} from "../lockfile/paths.js";
import { parseManifest, readManifestText } from "../manifest/index.js";
import type { ManifestPublishRemote } from "../manifest/types.js";
import { requireReleaseVersion } from "../package-spec/index.js";
import { resolveProject } from "../resolve/project.js";
import type { ResolvedPackage, ResolvedProject } from "../resolve/types.js";
import { publishToRoot, publishToServer } from "./index.js";
import {
  allowedPublishNames,
  type PublishCliDest,
  type PublishDestination,
  resolvePathRemote,
  resolvePublishDestinations,
} from "./remote.js";

export function loadPublishRemotes(): {
  projectRoot: string;
  remotes: ManifestPublishRemote[];
} {
  const path = defaultManifestPath();
  const manifest = parseManifest(readManifestText(path));
  return {
    projectRoot: projectRootFromManifest(path),
    remotes: manifest.publishRemotes ?? [],
  };
}

export async function runPublish(argumentsList: string[]): Promise<void> {
  const cli = parsePublishCli(argumentsList);
  const { projectRoot, remotes } = loadPublishRemotes();
  const dests = resolvePublishDestinations(remotes, cli, projectRoot);
  const project = await resolveProject(defaultManifestPath());
  if (cli.dryRun) {
    printPublishPlan(project, dests);
    return;
  }
  for (const dest of dests) {
    const packages = selectedPackages(project, dest.packages);
    const filtered = { ...project, packages };
    if (dest.root) {
      await publishToRoot(filtered, dest.root);
    }
    if (dest.server) {
      await publishToServer(filtered, dest.server);
    }
  }
}

export function optionalPathRemoteRoot(
  to: string | undefined,
  root: string,
): string {
  try {
    const { projectRoot, remotes } = loadPublishRemotes();
    if (remotes.length === 0 && !to) {
      return root;
    }
    const cliRoot = root === "." ? undefined : root;
    return resolvePathRemote(remotes, to, cliRoot, projectRoot);
  } catch (error) {
    if (
      to === undefined &&
      error instanceof PrayError &&
      error.kind === "manifest"
    ) {
      return root;
    }
    throw error;
  }
}

function parsePublishCli(
  argumentsList: string[],
): PublishCliDest & { dryRun: boolean } {
  const roots: string[] = [];
  const servers: string[] = [];
  const to: string[] = [];
  let dryRun = false;
  for (let index = 0; index < argumentsList.length; index += 1) {
    const argument = argumentsList[index];
    if (argument === "--root") {
      const value = argumentsList[index + 1];
      if (!value) {
        throw PrayError.unsupported("publish requires a path after --root");
      }
      roots.push(value);
      index += 1;
    } else if (argument === "--server") {
      const value = argumentsList[index + 1];
      if (!value) {
        throw PrayError.unsupported("publish requires a URL after --server");
      }
      servers.push(value);
      index += 1;
    } else if (argument === "--to") {
      const value = argumentsList[index + 1];
      if (!value) {
        throw PrayError.unsupported("publish requires a name after --to");
      }
      to.push(value);
      index += 1;
    } else if (argument === "--dry-run") {
      dryRun = true;
    } else if (argument === "--signing-key") {
      index += 1;
    } else if (argument?.startsWith("--")) {
      throw PrayError.unsupported(`unknown publish flag: ${argument}`);
    } else if (argument) {
      throw PrayError.unsupported(`unexpected publish argument: ${argument}`);
    }
  }
  return { to, roots, servers, dryRun };
}

function selectedPackages(
  project: ResolvedProject,
  listed: string[],
): ResolvedPackage[] {
  const allowed = allowedPublishNames(project.manifest, listed);
  const selected = project.packages.filter((entry) =>
    allowed.includes(entry.declaration.name),
  );
  if (selected.length === 0) {
    throw PrayError.usage(
      "no path packages to publish; remote dependencies are not published",
    );
  }
  for (const packageEntry of selected) {
    requireReleaseVersion(packageEntry.spec);
  }
  return selected;
}

function printPublishPlan(
  project: ResolvedProject,
  dests: PublishDestination[],
): void {
  for (const dest of dests) {
    const packages = selectedPackages(project, dest.packages);
    const target = dest.root ?? dest.server ?? dest.name;
    for (const packageEntry of packages) {
      process.stdout.write(
        `publish ${packageEntry.declaration.name} ${packageEntry.spec.version} -> ${dest.name} (${target})\n`,
      );
    }
  }
}
