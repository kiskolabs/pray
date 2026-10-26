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

import { PrayError } from "pray-core/errors.js";
import { configureOutput } from "pray-core/output.js";
import {
  activeInvocationContext,
  setActiveInvocationContext,
} from "pray-core/project-context/runtime.js";
import { runTransaction } from "pray-core/transaction/index.js";
import { dispatchCommand } from "./dispatch.js";
import { conciseHelpText, maybePrintHelp } from "./help.js";
import { initializeInvocation, projectRoot } from "./invocation.js";
import { unknownCommandMessage } from "./suggest.js";

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
    const execute = async (): Promise<number> => dispatchCommand(command, rest);
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
