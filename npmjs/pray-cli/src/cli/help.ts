const DOCS_URL = "https://github.com/kiskolabs/pray";

const WORKFLOW_COMMANDS = [
  "install [--locked|--frozen|--offline]  resolve, render, and write Prayfile.lock",
  "plan [--remote]                        preview materialization changes",
  "apply                                  apply the current plan",
  "verify [--strict]                      check rendered output against the lockfile",
  "drift [--semantic]                     compare lockfile to current resolution",
  "render [--check]                       render targets without updating the lockfile",
  "format                                 canonicalize Prayfile formatting",
];

const PACKAGE_COMMANDS = [
  "add <name> [constraint] [--path PATH]",
  "remove <name>",
  "update [package] [--major] [--latest] [--dry-run] [--json]",
  "unlock <package>",
  "vendor                                 copy resolved packages locally",
  "clean                                  remove local cache and vendor trees",
];

const DISTRIBUTION_COMMANDS = [
  "publish --root PATH [--server URL ...]",
  "login --server URL --email EMAIL",
  "serve [--root PATH] [--host HOST] [--port PORT] [--stdio]",
  "sync [--root PATH] [--peer URL ...]",
  "confess <package> | --from-lock SPAN_ID [--accepted|--rejected]",
];

const TRUST_COMMANDS = [
  "trust list|show|add-key|remove-key|set-signed|set-allow|import-repo|import-registry|check",
];

const INSPECT_COMMANDS = [
  "list                                   list declared packages",
  "outdated [--remote]                      show constraint vs resolved versions",
  "explain <package>                        show why a package was selected",
  "tree                                     print the dependency tree",
];

const META_COMMANDS = [
  "init [--targets tool_a,tool_b]",
  "prayer init                            scaffold a prayer package",
  "repo init                              scaffold a distribution root",
  "manifest                               print canonical Prayfile JSON",
  "package                                build a distributable prayer archive",
  "version | -V | --version",
];

const COMMAND_HELP: Record<string, string> = {
  install:
    "install — resolve packages, render targets, and update Prayfile.lock\n\n" +
    "Usage: pray install [--locked|--frozen|--offline]\n\n" +
    "--locked   require an existing lockfile\n" +
    "--frozen   require lockfile to match Prayfile exactly\n" +
    "--offline  use cache only",
  verify:
    "verify — check rendered files against Prayfile.lock\n\n" +
    "Usage: pray verify [--strict]\n\n" +
    "Without --strict, orphan-marker warnings print to stderr but exit 0.\n" +
    "With --strict, any finding fails with exit code 6.",
  drift:
    "drift — report differences between lockfile and current resolution\n\n" +
    "Usage: pray drift [--semantic]\n\n" +
    "Exits with code 6 when drift is found.",
  update:
    "update — refresh package versions within constraints\n\n" +
    "Usage: pray update [package] [--major] [--latest] [--dry-run] [--json]",
  plan: "plan — preview install/apply changes\n\nUsage: pray plan [--remote]",
  apply: "apply — materialize the current resolution plan\n\nUsage: pray apply",
  trust:
    "trust — manage client trust policy for remote sources\n\n" +
    "Usage: pray trust <subcommand>\n\n" +
    "Subcommands: list, show, add-key, remove-key, set-signed, set-allow, import-repo, import-registry, check",
  init: "init — create a starter Prayfile\n\nUsage: pray init [--targets tool_a,tool_b]",
  add: "add — declare a package in Prayfile\n\nUsage: pray add <name> [constraint] [--path PATH]",
  publish:
    "publish — upload packages to a registry or local root\n\n" +
    "Usage: pray publish --root PATH [--server URL ...]",
  serve:
    "serve — run a local registry server\n\n" +
    "Usage: pray serve [--root PATH] [--host HOST] [--port PORT] [--stdio]",
};

function printCommandGroups(): string {
  const lines = [
    "Workflow:",
    ...WORKFLOW_COMMANDS.map((line) => `  ${line}`),
    "Packages:",
    ...PACKAGE_COMMANDS.map((line) => `  ${line}`),
    "Distribution:",
    ...DISTRIBUTION_COMMANDS.map((line) => `  ${line}`),
    "Trust:",
    ...TRUST_COMMANDS.map((line) => `  ${line}`),
    "Inspect:",
    ...INSPECT_COMMANDS.map((line) => `  ${line}`),
    "Meta:",
    ...META_COMMANDS.map((line) => `  ${line}`),
    "Global flags: --no-input (disable prompts), --rm (ephemeral home), --trust [--global]",
  ];
  return `${lines.join("\n")}\n`;
}

export function conciseHelpText(): string {
  return [
    "pray — reproducible inference input for projects",
    "",
    "Declare shared instructions in Prayfile, lock versions, and render tool-specific output.",
    "",
    "Getting started:",
    "  pray init",
    "  pray install",
    "  pray plan",
    "  pray apply",
    "  pray verify",
    "",
    printCommandGroups(),
    "Run `pray help <command>` or `pray <command> --help` for one command.",
    `Documentation: ${DOCS_URL}`,
    "Exit codes: 0 success; 2 usage/parse; 3 resolution; 4 integrity; 5 render; 6 verify; 8 unsupported.",
    "",
  ].join("\n");
}

export const HELP_TEXT = conciseHelpText();

export function commandHelpText(command: string): string | undefined {
  const text = COMMAND_HELP[command];
  if (!text) {
    return undefined;
  }
  return `${text}\n\nDocumentation: ${DOCS_URL}\n`;
}

export type HelpDispatchResult = "printed" | "not_help";

export function maybePrintHelp(argumentsList: string[]): HelpDispatchResult {
  if (argumentsList.length === 0) {
    process.stdout.write(conciseHelpText());
    return "printed";
  }

  if (
    argumentsList.length === 1 &&
    (argumentsList[0] === "help" ||
      argumentsList[0] === "-h" ||
      argumentsList[0] === "--help")
  ) {
    process.stdout.write(conciseHelpText());
    return "printed";
  }

  if (argumentsList[0] === "help") {
    const target = argumentsList[1] ?? "";
    if (target === "" || target === "-h" || target === "--help") {
      process.stdout.write(conciseHelpText());
      return "printed";
    }
    const text = commandHelpText(target);
    if (text) {
      process.stdout.write(text);
      return "printed";
    }
    return "not_help";
  }

  const helpPosition = argumentsList.findIndex(
    (argument) => argument === "--help" || argument === "-h",
  );
  if (helpPosition >= 0) {
    if (helpPosition === 0) {
      process.stdout.write(conciseHelpText());
      return "printed";
    }
    const text = commandHelpText(argumentsList[0] ?? "");
    if (text) {
      process.stdout.write(text);
      return "printed";
    }
    return "not_help";
  }

  return "not_help";
}
