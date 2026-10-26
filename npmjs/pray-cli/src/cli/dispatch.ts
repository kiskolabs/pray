import { runLoginCommand } from "pray-core/auth/login.js";
import { PrayError } from "pray-core/errors.js";
import { readLockfile } from "pray-core/lockfile/index.js";
import { PACKAGE_VERSION } from "pray-core/lockfile/types.js";
import { materializeProject, printManifest } from "pray-core/materialize.js";
import { initDistributionRoot } from "pray-core/publish/index.js";
import { maybeDeclarePublishRemote } from "pray-core/publish/remote-init.js";
import { ensureRenderedTargetsCurrent } from "pray-core/render/check.js";
import { renderProject } from "pray-core/render/project.js";
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
import { conciseHelpText } from "./help.js";
import {
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

/** Dispatch one resolved CLI command; returns the process exit code. */
export async function dispatchCommand(
  command: string | undefined,
  rest: string[],
): Promise<number> {
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
}
