const WORKFLOW_COMMANDS = [
  "install [--locked|--frozen|--offline]  resolve, render, and write Prayfile.lock",
  "plan [--remote]                        preview materialization changes",
  "apply                                  apply the current plan",
  "verify [--strict]                      check rendered output against the lockfile",
  "drift [--semantic]                     compare lockfile to current resolution",
  "render [--check]                       render targets without updating the lockfile",
  "format|fmt                             rewrite Prayfile to recommended destination DSL",
];

const PACKAGE_COMMANDS = [
  "add <name> [constraint] [--path PATH]  declare a package in Prayfile",
  "remove <name>                          remove a package from Prayfile",
  "update [package] [--major] [--latest] [--dry-run] [--json]",
  "unlock <package>                       clear a locked package pin",
  "vendor                                 copy resolved packages locally",
  "clean [--unused]                       remove local state or unused registry entries",
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
  "outdated [--remote]                    show constraint vs resolved versions",
  "explain <package>                      show why a package was selected",
  "tree                                   print the dependency tree",
];

const META_COMMANDS = [
  "init [--targets tool_a,tool_b]         create a starter Prayfile",
  "prayer init                            scaffold a prayer package",
  "repo init                              scaffold a distribution root",
  "manifest                               print canonical Prayfile JSON",
  "package                                build a distributable prayer archive",
  "upgrade                                install the latest pray CLI release",
  "version | -V | --version               print the pray CLI version",
];

const GLOBAL_OPTIONS = [
  "--no-input            disable prompts",
  "--rm                  use an ephemeral home directory",
  "--trust [--global]    import trust on first use",
];

const COMMAND_HELP: Record<string, string> = {
  install:
    "resolve packages, render targets, and update Prayfile.lock\n\n" +
    "Usage: pray install [--locked|--frozen|--offline]\n\n" +
    "--locked   require an existing lockfile\n" +
    "--frozen   require lockfile to match Prayfile exactly\n" +
    "--offline  use cache only",
  verify:
    "check rendered files against Prayfile.lock\n\n" +
    "Usage: pray verify [--strict]\n\n" +
    "Without --strict, orphan-marker warnings print to stderr but exit 0.\n" +
    "With --strict, any finding fails with exit code 6.",
  drift:
    "report differences between lockfile and current resolution\n\n" +
    "Usage: pray drift [--semantic]\n\n" +
    "Exits with code 6 when drift is found.",
  render:
    "render targets without updating the lockfile\n\n" +
    "Usage: pray render [--check]",
  format:
    "rewrite Prayfile to recommended destination DSL\n\n" +
    "Usage: pray format\n       pray fmt",
  fmt:
    "rewrite Prayfile to recommended destination DSL\n\n" +
    "Usage: pray format\n       pray fmt",
  update:
    "refresh package versions within constraints\n\n" +
    "Usage: pray update [package] [--major] [--latest] [--dry-run] [--json]",
  plan: "preview install/apply changes\n\nUsage: pray plan [--remote]",
  outdated:
    "show constraint vs resolved versions\n\n" +
    "Usage: pray outdated [--remote]",
  apply: "materialize the current resolution plan\n\nUsage: pray apply",
  add: "declare a package in Prayfile\n\nUsage: pray add <name> [constraint] [--path PATH]",
  remove: "remove a package from Prayfile\n\nUsage: pray remove <name>",
  unlock: "clear a locked package pin\n\nUsage: pray unlock <package>",
  vendor: "copy resolved packages locally\n\nUsage: pray vendor",
  clean:
    "remove local cache and vendor trees, or only unused registry entries\n\nUsage: pray clean [--unused]",
  login:
    "authenticate to a registry server\n\n" +
    "Usage: pray login --server URL --email EMAIL (--passkey-key PATH --credential-id ID | --ssh-agent --public-key PATH)",
  upgrade:
    "install the latest pray CLI release\n\n" +
    "Usage: pray upgrade\n\n" +
    "Runs `npm install -g pray-cli@latest`.",
  trust:
    "manage client trust policy for remote sources\n\n" +
    "Usage: pray trust <subcommand>\n\n" +
    "Subcommands: list, show, add-key, remove-key, set-signed, set-allow, import-repo, import-registry, check",
  init: "create a starter Prayfile\n\nUsage: pray init [--targets tool_a,tool_b]",
  publish:
    "upload packages to a registry or local root\n\n" +
    "Usage: pray publish --root PATH [--server URL ...]",
  serve:
    "run a local registry server\n\n" +
    "Usage: pray serve [--root PATH] [--host HOST] [--port PORT] [--stdio]",
  sync:
    "sync packages with peer registries\n\n" +
    "Usage: pray sync [--root PATH] [--peer URL ...]",
  confess:
    "record an acceptance or rejection for a package confession\n\n" +
    "Usage: pray confess <package> | --from-lock SPAN_ID [--accepted|--rejected]",
  list: "list declared packages\n\nUsage: pray list",
  explain: "show why a package was selected\n\nUsage: pray explain <package>",
  tree: "print the dependency tree\n\nUsage: pray tree",
  prayer: "scaffold a prayer package\n\nUsage: pray prayer init",
  repo: "scaffold a distribution root\n\nUsage: pray repo init",
  manifest: "print canonical Prayfile JSON\n\nUsage: pray manifest",
  package: "build a distributable prayer archive\n\nUsage: pray package",
  version:
    "print the pray CLI version\n\n" +
    "Usage: pray version\n       pray -V | --version",
  help:
    "show help for pray or one command\n\n" +
    "Usage: pray help [command]\n       pray [command] --help",
};

function printCommandGroups(): string {
  const groups: Array<[string, string[]]> = [
    ["Workflow", WORKFLOW_COMMANDS],
    ["Packages", PACKAGE_COMMANDS],
    ["Distribution", DISTRIBUTION_COMMANDS],
    ["Trust", TRUST_COMMANDS],
    ["Inspect", INSPECT_COMMANDS],
    ["Meta", META_COMMANDS],
  ];
  return groups
    .map(([title, lines]) =>
      [`${title}:`, ...lines.map((line) => `  ${line}`)].join("\n"),
    )
    .join("\n\n");
}

export function conciseHelpText(): string {
  return [
    "Usage: pray [OPTIONS] <COMMAND>",
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
    "",
    "Options:",
    ...GLOBAL_OPTIONS.map((line) => `  ${line}`),
    "",
    "See 'pray help <command>' or 'pray <command> --help' for details on a command.",
    "",
  ].join("\n");
}

export const HELP_TEXT = conciseHelpText();

export function commandHelpText(command: string): string | undefined {
  return COMMAND_HELP[command];
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
      process.stdout.write(`${text}\n`);
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
      process.stdout.write(`${text}\n`);
      return "printed";
    }
    return "not_help";
  }

  return "not_help";
}
