import { submitConfession } from "pray-core/confess/index.js";
import { PrayError } from "pray-core/errors.js";
import { defaultManifestPath } from "pray-core/lockfile/paths.js";
import { optionalPathRemoteRoot } from "pray-core/publish/command.js";
import { runServer, runStdioRpc } from "pray-core/serve/index.js";
import { syncDistributionRoot } from "pray-core/sync/index.js";

export { runPublish } from "pray-core/publish/command.js";

export async function runServe(argumentsList: string[]): Promise<void> {
  let root = ".";
  let to: string | undefined;
  let host = "127.0.0.1";
  let port = 7429;
  let stdio = false;
  for (let index = 0; index < argumentsList.length; index += 1) {
    const argument = argumentsList[index];
    if (argument === undefined) {
      continue;
    }
    if (argument === "--root") {
      root = requiredValue(argumentsList, index, "--root");
      index += 1;
    } else if (argument === "--to") {
      to = requiredValue(argumentsList, index, "--to");
      index += 1;
    } else if (argument === "--host") {
      host = requiredValue(argumentsList, index, "--host");
      index += 1;
    } else if (argument === "--port") {
      port = Number(requiredValue(argumentsList, index, "--port"));
      if (!Number.isInteger(port) || port < 0 || port > 65_535) {
        throw PrayError.usage("--port requires an integer from 0 to 65535");
      }
      index += 1;
    } else if (argument === "--stdio") {
      stdio = true;
    } else {
      throw PrayError.usage(`unknown serve argument: ${argument}`);
    }
  }
  if (stdio) {
    runStdioRpc();
  }
  await runServer({ root: optionalPathRemoteRoot(to, root), host, port });
}

export async function runSync(argumentsList: string[]): Promise<void> {
  let root = ".";
  const peers: string[] = [];
  for (let index = 0; index < argumentsList.length; index += 1) {
    const argument = argumentsList[index];
    if (argument === undefined) {
      continue;
    }
    if (argument === "--root") {
      root = requiredValue(argumentsList, index, "--root");
      index += 1;
    } else if (argument === "--peer") {
      peers.push(requiredValue(argumentsList, index, "--peer"));
      index += 1;
    } else {
      throw PrayError.usage(`unknown sync argument: ${argument}`);
    }
  }
  if (peers.length === 0) {
    throw PrayError.usage("sync requires at least one --peer URL");
  }
  const summary = await syncDistributionRoot(root, peers);
  process.stdout.write(
    `synced ${summary.packages.length} packages from ${summary.peers.length} peers\n`,
  );
}

export async function runConfess(argumentsList: string[]): Promise<void> {
  const options: {
    packageName?: string;
    fromLock?: string;
    version?: string;
    accepted?: boolean;
    rejected?: boolean;
    note?: string;
    url?: string;
  } = {};
  for (let index = 0; index < argumentsList.length; index += 1) {
    const argument = argumentsList[index];
    if (argument === undefined) {
      continue;
    }
    if (argument === "--from-lock") {
      options.fromLock = requiredValue(argumentsList, index, "--from-lock");
      index += 1;
    } else if (argument === "--version") {
      options.version = requiredValue(argumentsList, index, "--version");
      index += 1;
    } else if (argument === "--note") {
      options.note = requiredValue(argumentsList, index, "--note");
      index += 1;
    } else if (argument === "--url") {
      options.url = requiredValue(argumentsList, index, "--url");
      index += 1;
    } else if (argument === "--accepted") {
      options.accepted = true;
    } else if (argument === "--rejected") {
      options.rejected = true;
    } else if (!argument.startsWith("-") && !options.packageName) {
      options.packageName = argument;
    } else {
      throw PrayError.usage(`unknown confess argument: ${argument}`);
    }
  }
  await submitConfession(defaultManifestPath(), options);
}

function requiredValue(
  argumentsList: string[],
  index: number,
  flag: string,
): string {
  const value = argumentsList[index + 1];
  if (!value || value.startsWith("-")) {
    throw PrayError.usage(`${flag} requires a value`);
  }
  return value;
}
