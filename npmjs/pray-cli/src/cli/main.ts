/**
 * TypeScript `pray` CLI for Prayfile. Depends on `pray-core` / `@kiskolabs/pray-core`
 * for resolve, lock, render, verify, and publish. Library imports use that package,
 * not this one.
 *
 * @example
 * ```ts
 * import { runCli } from "@kiskolabs/pray-cli";
 *
 * const code = await runCli(["version"]);
 * ```
 *
 * @module
 */

import { runLoginCommand } from "pray-core/auth/login.js";
import { PrayError } from "pray-core/errors.js";
import { readLockfile } from "pray-core/lockfile/index.js";
import { PACKAGE_VERSION } from "pray-core/lockfile/types.js";
import { materializeProject, printManifest } from "pray-core/materialize.js";
import { configureOutput } from "pray-core/output.js";
import {
  activeInvocationContext,
  setActiveInvocationContext,
} from "pray-core/project-context/runtime.js";
import { initDistributionRoot } from "pray-core/publish/index.js";
import { maybeDeclarePublishRemote } from "pray-core/publish/remote-init.js";
import { ensureRenderedTargetsCurrent } from "pray-core/render/check.js";
import { renderProject } from "pray-core/render/project.js";
import { runTransaction } from "pray-core/transaction/index.js";
import { renderDependencyTree } from "pray-core/tree/index.js";
import { runTrustCommand } from "pray-core/trust/index.js";
import {
  cleanProjectCaches,
  cleanUnusedRegistryCache,
  vendorProject,
} from "pray-core/vendor/index.js";
import { driftProject, verifyProject } from "pray-core/verify/project.js";
import { parseCleanArguments } from "./commands/clean.js";
import {
  runConfess,
  runPublish,
  runServe,
  runSync,
} from "./commands/distribution.js";
import { runInit, runPrayerInit } from "./commands/init.js";
import { runExplain, runList, runOutdated } from "./commands/inspect.js";
import { runAdd, runRemove, runUnlock } from "./commands/packages.js";
import { runPlanCommand } from "./commands/plan.js";
import { runUpdateCommand } from "./commands/update.js";
import { runFormat, runPackage } from "./commands/workspace.js";
import { conciseHelpText, maybePrintHelp } from "./help.js";
import {
  initializeInvocation,
  lockfilePath,
  projectRoot,
  resolveCurrentProject,
} from "./invocation.js";
import { unknownCommandMessage } from "./suggest.js";
import { runUpgradeCommand } from "./upgrade.js";

function rejectUnknownArguments(
  command: string,
  argumentsList: string[],
  allowed: string[],
): void {
  const unknown = argumentsList.find((argument) => !allowed.includes(argument));
  if (unknown !== undefined) {
    throw PrayError.usage(`unknown ${command} argument: ${unknown}`);
  }
}

/** Run the `pray` CLI with the given argv-style argument list; returns the process exit code. */
export async function runCli(argumentsList: string[]): Promise<number> {
  const previousContext = activeInvocationContext();
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
    let quiet = false;
    let verbose = 0;
    for (let index = filteredArguments.length - 1; index >= 0; index -= 1) {
      const argument = filteredArguments[index];
      if (argument === "--quiet" || argument === "-q") {
        quiet = true;
        filteredArguments.splice(index, 1);
      } else if (argument === "--verbose" || argument === "-v") {
        verbose += 1;
        filteredArguments.splice(index, 1);
      }
    }
    configureOutput(quiet, verbose);

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
    const execute = async (): Promise<number> => {
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
          runPrayerInit(rest.slice(1));
          return 0;
        case "repo":
          if (rest[0] !== "init") {
            throw PrayError.unsupported("repo requires init");
          }
          initDistributionRoot(process.cwd());
          maybeDeclarePublishRemote(
            process.cwd(),
            process.cwd().endsWith("prayers")
              ? process.cwd()
              : `${process.cwd()}/prayers`,
          );
          process.stdout.write("created distribution root\n");
          return 0;
        case "add":
          await runAdd(rest);
          return 0;
        case "remove":
          await runRemove(rest[0]);
          return 0;
        case "update":
          await runUpdateCommand(rest);
          return 0;
        case "unlock":
          await runUnlock(rest[0]);
          return 0;
        case "install":
        case "apply":
          rejectUnknownArguments(command, rest, [
            "--frozen",
            "--locked",
            "--offline",
          ]);
          await materializeProject({
            frozen: rest.includes("--frozen"),
            locked: rest.includes("--locked"),
            offline: rest.includes("--offline"),
          });
          return 0;
        case "plan":
          await runPlanCommand(rest);
          return 0;
        case "render":
          rejectUnknownArguments(command, rest, ["--check"]);
          if (rest.includes("--check")) {
            const project = await resolveCurrentProject();
            ensureRenderedTargetsCurrent(project, renderProject(project));
            process.stdout.write("render check: ok\n");
            return 0;
          }
          await materializeProject();
          return 0;
        case "verify": {
          rejectUnknownArguments(command, rest, ["--strict"]);
          const project = await resolveCurrentProject();
          const lockfile = readLockfile(lockfilePath());
          verifyProject(project, lockfile, rest.includes("--strict"));
          process.stdout.write("verify: ok\n");
          return 0;
        }
        case "drift": {
          rejectUnknownArguments(command, rest, ["--semantic"]);
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
        case "fmt":
          await runFormat();
          return 0;
        case "package":
          await runPackage();
          return 0;
        case "publish":
          await runPublish(rest);
          return 0;
        case "login":
          await runLoginCommand(rest, projectRoot());
          return 0;
        case "upgrade":
          runUpgradeCommand();
          return 0;
        case "serve":
          await runServe(rest);
          return 0;
        case "sync":
          await runSync(rest);
          return 0;
        case "trust":
          await runTrustCommand(rest, projectRoot());
          return 0;
        case "confess":
          await runConfess(rest);
          return 0;
        case "list":
          await runList();
          return 0;
        case "outdated":
          await runOutdated(rest);
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
          const options = parseCleanArguments(rest);
          if (options.unused) {
            cleanUnusedRegistryCache(projectRoot());
          } else {
            cleanProjectCaches(projectRoot());
          }
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
    };
    return await ([
      "update",
      "unlock",
      "add",
      "remove",
      "render",
      "plan",
      "verify",
      "drift",
    ].includes(command ?? "") &&
    !(command === "render" && rest.includes("--check"))
      ? runTransaction(projectRoot(), execute)
      : execute());
  } catch (error) {
    if (error instanceof PrayError) {
      process.stderr.write(`${error.toString()}\n`);
      return error.exitCode();
    }
    const message = error instanceof Error ? error.message : String(error);
    process.stderr.write(`${message}\n`);
    return 1;
  } finally {
    setActiveInvocationContext(previousContext);
  }
}
