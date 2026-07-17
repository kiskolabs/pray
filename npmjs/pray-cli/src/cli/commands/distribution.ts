import { submitConfession } from "../../confess/index.js";
import { PrayError } from "../../errors.js";
import { defaultManifestPath } from "../../lockfile/paths.js";
import { publishToRoot, publishToServer } from "../../publish/index.js";
import { resolveProject } from "../../resolve/project.js";
import { runServer, runStdioRpc } from "../../serve/index.js";
import { syncDistributionRoot } from "../../sync/index.js";

export async function runPublish(argumentsList: string[]): Promise<void> {
  const roots: string[] = [];
  const servers: string[] = [];
  for (let index = 0; index < argumentsList.length; index += 1) {
    const argument = argumentsList[index];
    if (argument === undefined) {
      continue;
    }
    if (argument === "--root") {
      const root = argumentsList[index + 1];
      if (root) {
        roots.push(root);
      }
      index += 1;
    } else if (argument === "--server") {
      const server = argumentsList[index + 1];
      if (server) {
        servers.push(server);
      }
      index += 1;
    }
  }
  if (roots.length === 0 && servers.length === 0) {
    throw PrayError.unsupported(
      "publish requires at least one --root PATH or --server URL",
    );
  }
  const project = await resolveProject(defaultManifestPath());
  for (const root of roots) {
    await publishToRoot(project, root);
  }
  for (const server of servers) {
    await publishToServer(project, server);
  }
}

export async function runServe(argumentsList: string[]): Promise<void> {
  let root = ".";
  let host = "127.0.0.1";
  let port = 7429;
  let stdio = false;
  for (let index = 0; index < argumentsList.length; index += 1) {
    const argument = argumentsList[index];
    if (argument === undefined) {
      continue;
    }
    if (argument === "--root") {
      root = argumentsList[index + 1] ?? root;
      index += 1;
    } else if (argument === "--host") {
      host = argumentsList[index + 1] ?? host;
      index += 1;
    } else if (argument === "--port") {
      port = Number.parseInt(argumentsList[index + 1] ?? String(port), 10);
      index += 1;
    } else if (argument === "--stdio") {
      stdio = true;
    }
  }
  if (stdio) {
    runStdioRpc();
  }
  await runServer({ root, host, port });
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
      root = argumentsList[index + 1] ?? root;
      index += 1;
    } else if (argument === "--peer") {
      const peer = argumentsList[index + 1];
      if (peer) {
        peers.push(peer);
      }
      index += 1;
    }
  }
  if (peers.length === 0) {
    throw PrayError.unsupported("sync requires at least one --peer URL");
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
      options.fromLock = argumentsList[index + 1];
      index += 1;
    } else if (argument === "--version") {
      options.version = argumentsList[index + 1];
      index += 1;
    } else if (argument === "--note") {
      options.note = argumentsList[index + 1];
      index += 1;
    } else if (argument === "--url") {
      options.url = argumentsList[index + 1];
      index += 1;
    } else if (argument === "--accepted") {
      options.accepted = true;
    } else if (argument === "--rejected") {
      options.rejected = true;
    } else if (!options.packageName) {
      options.packageName = argument;
    }
  }
  await submitConfession(defaultManifestPath(), options);
}
