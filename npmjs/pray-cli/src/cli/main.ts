import { PrayError } from "../errors.js";
import { readLockfile } from "../lockfile/index.js";
import {
  defaultLockfilePath,
  defaultManifestPath,
} from "../lockfile/paths.js";
import { PACKAGE_VERSION } from "../lockfile/types.js";
import { initDistributionRoot } from "../publish/index.js";
import { renderProject } from "../render/project.js";
import { resolveProject } from "../resolve/project.js";
import { loginCommand } from "../sync/index.js";
import { renderDependencyTree } from "../tree/index.js";
import { runTrustCommand } from "../trust/index.js";
import { cleanProjectCaches, vendorProject } from "../vendor/index.js";
import { driftProject, verifyProject } from "../verify/project.js";
import {
  runConfess,
  runPublish,
  runServe,
  runSync,
} from "./commands/distribution.js";
import { runExplain, runList, runOutdated } from "./commands/inspect.js";
import { runInit, runPrayerInit } from "./commands/init.js";
import { runAdd, runRemove, runUnlock } from "./commands/packages.js";
import { runFormat, runPackage } from "./commands/workspace.js";
import { HELP_TEXT } from "./help.js";
import { materializeProject, printManifest } from "./materialize.js";

export async function runCli(argumentsList: string[]): Promise<number> {
  try {
    if (argumentsList.length === 0) {
      process.stdout.write(HELP_TEXT);
      return 0;
    }

    const [command, ...rest] = argumentsList;
    switch (command) {
      case "version":
      case "-V":
      case "--version":
        process.stdout.write(`pray ${PACKAGE_VERSION} (typescript)\n`);
        return 0;
      case "help":
      case "-h":
      case "--help":
        process.stdout.write(HELP_TEXT);
        return 0;
      case "manifest":
        await printManifest();
        return 0;
      case "init":
        runInit(rest);
        return 0;
      case "prayer":
        if (rest[0] !== "init") {
          throw PrayError.unsupported("prayer requires init");
        }
        runPrayerInit();
        return 0;
      case "repo":
        if (rest[0] !== "init") {
          throw PrayError.unsupported("repo requires init");
        }
        initDistributionRoot(process.cwd());
        process.stdout.write("created distribution root\n");
        return 0;
      case "add":
        await runAdd(rest);
        return 0;
      case "remove":
        await runRemove(rest[0]);
        return 0;
      case "update":
        await materializeProject({
          offline: rest.includes("--offline"),
          refreshSourceRevisions: true,
        });
        return 0;
      case "unlock":
        await runUnlock(rest[0]);
        return 0;
      case "install":
      case "apply":
        await materializeProject({
          frozen: rest.includes("--frozen"),
          locked: rest.includes("--locked"),
          offline: rest.includes("--offline"),
        });
        return 0;
      case "plan": {
        const project = await resolveProject(defaultManifestPath());
        const rendered = renderProject(project);
        for (const target of rendered) {
          process.stdout.write(`would render ${target.path}\n`);
        }
        return 0;
      }
      case "render":
        if (rest.includes("--check")) {
          const project = await resolveProject(defaultManifestPath());
          renderProject(project);
          process.stdout.write("render check: ok\n");
          return 0;
        }
        await materializeProject();
        return 0;
      case "verify": {
        const project = await resolveProject(defaultManifestPath());
        const lockfile = readLockfile(defaultLockfilePath(project.projectRoot));
        verifyProject(project, lockfile, rest.includes("--strict"));
        process.stdout.write("verify: ok\n");
        return 0;
      }
      case "drift": {
        const project = await resolveProject(defaultManifestPath());
        const lockfile = readLockfile(defaultLockfilePath(project.projectRoot));
        if (rest.includes("--semantic")) {
          verifyProject(project, lockfile, false);
        } else {
          driftProject(project, lockfile);
        }
        process.stdout.write("drift: ok\n");
        return 0;
      }
      case "format":
        await runFormat();
        return 0;
      case "package":
        await runPackage();
        return 0;
      case "publish":
        await runPublish(rest);
        return 0;
      case "login":
        await loginCommand();
        return 0;
      case "serve":
        await runServe(rest);
        return 0;
      case "sync":
        await runSync(rest);
        return 0;
      case "trust":
        runTrustCommand(rest);
        return 0;
      case "confess":
        await runConfess(rest);
        return 0;
      case "list":
        await runList();
        return 0;
      case "outdated":
        await runOutdated();
        return 0;
      case "explain":
        await runExplain(rest[0]);
        return 0;
      case "vendor": {
        const project = await resolveProject(defaultManifestPath());
        vendorProject(project);
        return 0;
      }
      case "clean": {
        const project = await resolveProject(defaultManifestPath());
        cleanProjectCaches(project.projectRoot);
        return 0;
      }
      case "tree": {
        const project = await resolveProject(defaultManifestPath());
        process.stdout.write(`${renderDependencyTree(project).join("\n")}\n`);
        return 0;
      }
      default:
        throw PrayError.unsupported(`unknown command: ${command}`);
    }
  } catch (error) {
    if (error instanceof PrayError) {
      process.stderr.write(`${error.message}\n`);
      return error.exitCode();
    }
    const message = error instanceof Error ? error.message : String(error);
    process.stderr.write(`${message}\n`);
    return 1;
  }
}
