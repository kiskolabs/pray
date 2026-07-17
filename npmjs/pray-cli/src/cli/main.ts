import { PrayError } from "../errors.js";
import { readLockfile } from "../lockfile/index.js";
import { PACKAGE_VERSION } from "../lockfile/types.js";
import { initDistributionRoot } from "../publish/index.js";
import { renderProject } from "../render/project.js";
import { defaultResolveOptions } from "../resolve/context.js";
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
import { runInit, runPrayerInit } from "./commands/init.js";
import { runExplain, runList, runOutdated } from "./commands/inspect.js";
import { runAdd, runRemove, runUnlock } from "./commands/packages.js";
import { runFormat, runPackage } from "./commands/workspace.js";
import { conciseHelpText, maybePrintHelp } from "./help.js";
import {
  initializeInvocation,
  lockfilePath,
  resolveCurrentProject,
  resolveCurrentProjectWithGitRefreshFallback,
} from "./invocation.js";
import { materializeProject, printManifest } from "./materialize.js";
import { unknownCommandMessage } from "./suggest.js";

export async function runCli(argumentsList: string[]): Promise<number> {
  try {
    const filteredArguments = [...argumentsList];
    if (filteredArguments.includes("--no-input")) {
      process.env.PRAY_NO_INPUT = "1";
      while (true) {
        const index = filteredArguments.indexOf("--no-input");
        if (index < 0) {
          break;
        }
        filteredArguments.splice(index, 1);
      }
    }

    const helpResult = maybePrintHelp(filteredArguments);
    if (helpResult === "printed") {
      return 0;
    }
    if (helpResult === "not_help" && filteredArguments[0] === "help") {
      throw PrayError.usage(unknownCommandMessage(filteredArguments[1] ?? ""));
    }
    if (
      helpResult === "not_help" &&
      filteredArguments.some(
        (argument) => argument === "--help" || argument === "-h",
      )
    ) {
      throw PrayError.usage(`unknown command: ${filteredArguments[0] ?? ""}`);
    }

    if (filteredArguments.length === 0) {
      process.stdout.write(conciseHelpText());
      return 0;
    }

    const [command, ...rest] = initializeInvocation(filteredArguments);
    switch (command) {
      case "version":
      case "-V":
      case "--version":
        process.stdout.write(`pray ${PACKAGE_VERSION} (typescript)\n`);
        return 0;
      case "help":
      case "-h":
      case "--help":
        process.stdout.write(conciseHelpText());
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
        const project = await resolveCurrentProjectWithGitRefreshFallback(
          defaultResolveOptions(),
          true,
        );
        const rendered = renderProject(project);
        for (const target of rendered) {
          process.stdout.write(`would render ${target.path}\n`);
        }
        return 0;
      }
      case "render":
        if (rest.includes("--check")) {
          const project = await resolveCurrentProject();
          renderProject(project);
          process.stdout.write("render check: ok\n");
          return 0;
        }
        await materializeProject();
        return 0;
      case "verify": {
        const project = await resolveCurrentProject();
        const lockfile = readLockfile(lockfilePath());
        verifyProject(project, lockfile, rest.includes("--strict"));
        process.stdout.write("verify: ok\n");
        return 0;
      }
      case "drift": {
        const project = await resolveCurrentProject();
        const lockfile = readLockfile(lockfilePath());
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
        const project = await resolveCurrentProject();
        vendorProject(project);
        return 0;
      }
      case "clean": {
        const project = await resolveCurrentProject();
        cleanProjectCaches(project.projectRoot);
        return 0;
      }
      case "tree": {
        const project = await resolveCurrentProject();
        process.stdout.write(`${renderDependencyTree(project).join("\n")}\n`);
        return 0;
      }
      default:
        throw PrayError.usage(unknownCommandMessage(command ?? ""));
    }
  } catch (error) {
    if (error instanceof PrayError) {
      process.stderr.write(`${error.toString()}\n`);
      return error.exitCode();
    }
    const message = error instanceof Error ? error.message : String(error);
    process.stderr.write(`${message}\n`);
    return 1;
  }
}
