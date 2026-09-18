const WORKFLOW_COMMANDS = [
  "install [--locked|--frozen|--offline]  keep locked versions, re-embed listed locals",
  "plan [--remote]                        dry-run of install: dest after patch",
  "apply                                  apply the current plan",
  "verify [--strict]                      dest managed spans versus Prayfile.lock",
  "drift [--semantic]                     dest versus lock, then dest versus a fresh render",
  "render [--check]                       write dest and Prayfile.lock from current inputs",
  "format|fmt                             rewrite Prayfile to recommended destination DSL",
];

const PACKAGE_COMMANDS = [
  "add <name> [constraint] [--path PATH]  declare a package in Prayfile",
  "remove <name>                          remove a package from Prayfile",
  "update [package] [--major|--latest|--dry-run|--json]  re-resolve packages and git sources within constraints",
  "unlock <package>                       clear a locked package pin",
  "vendor                                 copy resolved packages locally",
  "clean [--unused]                       remove local state or unused registry entries",
];

const DISTRIBUTION_COMMANDS = [
  "publish [--root PATH|--server URL|--to NAME] [--dry-run]",
  "login --server URL --email EMAIL",
  "serve [--root PATH | --to NAME] [--host HOST] [--port PORT] [--stdio]",
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
  "prayer init [name] [--path DIR]       scaffold a local prayer",
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
    "keep locked package versions and re-embed listed local compose sources\n\n" +
    "Usage: pray install [--locked|--frozen|--offline]\n\n" +
    "Install renders dest and writes Prayfile.lock. Package versions stay at the lock.\n" +
    "Listed local compose files are re-embedded and their span checksums refresh.\n" +
    "A path package with spec.upstream copies upstream files when the path tree has no content yet.\n" +
    "Use pray update to re-resolve package versions and git sources.\n" +
    "Use pray plan to preview a write.\n\n" +
    "--locked   require an existing lockfile\n" +
    "--frozen   fail if dest or lock would change\n" +
    "--offline  use cache only",
  verify:
    "compare dest managed spans versus Prayfile.lock\n\n" +
    "Usage: pray verify [--strict]\n\n" +
    "Read-only integrity check. It does not compare dest to a fresh render from current sources.\n" +
    "A changed local compose file is clean here when dest and lock still match.\n" +
    "Use pray drift to compare dest with a fresh render. Use pray plan to preview a write.\n\n" +
    "Without --strict, orphan-marker warnings print to stderr but exit 0.\n" +
    "With --strict, any finding fails with exit code 6.",
  drift:
    "compare dest versus Prayfile.lock, then dest versus a fresh render\n\n" +
    "Usage: pray drift [--semantic]\n\n" +
    "Read-only finding report from current packages and listed local files. Not a line diff.\n" +
    "Exits with code 6 when drift is found.\n" +
    "--semantic prints package version arrows only.\n" +
    "Use pray plan to preview a write. Use pray verify for dest versus lock alone.",
  render:
    "write dest and Prayfile.lock from current inputs\n\n" +
    "Usage: pray render [--check]\n\n" +
    "Without --check, render writes dest and the lock. Use pray plan to preview a write.\n" +
    "--check writes nothing. It fails unless dest already matches a fresh render from current inputs.\n" +
    "That is a dest identity gate, not a preview.",
  format:
    "rewrite Prayfile to recommended destination DSL\n\n" +
    "Usage: pray format\n       pray fmt",
  fmt:
    "rewrite Prayfile to recommended destination DSL\n\n" +
    "Usage: pray format\n       pray fmt",
  update:
    "re-resolve package versions and git sources within current constraints\n\n" +
    "Usage: pray update [package] [--major] [--latest] [--dry-run] [--json]\n\n" +
    "With no package name, update re-resolves the whole graph within constraints\n" +
    "and refreshes git source revisions. A package name unlocks that package only.\n" +
    "Path-fork packages can refresh from upstream.\n" +
    "Listed local compose files are re-embedded by pray install, not by update.\n\n" +
    "--major requires a package name.\n" +
    "--latest rewrites Prayfile constraints and exact spec.upstream pins to admit the latest versions, then refreshes those trees.\n" +
    "--latest --dry-run previews those versions and checks destination conflicts.\n" +
    "If a destination conflicts, inspect it and move it aside before retrying.\n" +
    "--json prints machine-readable output.\n" +
    "For files from an older pray, install the original package version first.",
  plan:
    "preview what install or apply would write after patch\n\n" +
    "Usage: pray plan [--remote]\n\n" +
    "This is the dry-run of a write. It does not write dest or the lock.",
  outdated:
    "show constraint vs resolved versions\n\n" +
    "Usage: pray outdated [--remote]\n\n" +
    "Path-fork files that differ from the locked upstream are listed.",
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
    "upload path packages to a registry or local root\n\n" +
    "Usage: pray publish [--root PATH] [--server URL ...] [--to NAME] [--dry-run]\n\n" +
    "Prayfile publish remotes supply dests when flags are omitted.",
  serve:
    "run a local registry server\n\n" +
    "Usage: pray serve [--root PATH | --to NAME] [--host HOST] [--port PORT] [--stdio]",
  sync:
    "sync packages with peer registries\n\n" +
    "Usage: pray sync [--root PATH] [--peer URL ...]",
  confess:
    "record an acceptance or rejection for a package confession\n\n" +
    "Usage: pray confess <package> | --from-lock SPAN_ID [--accepted|--rejected]",
  list: "list declared packages\n\nUsage: pray list",
  explain: "show why a package was selected\n\nUsage: pray explain <package>",
  tree: "print the dependency tree\n\nUsage: pray tree",
  prayer:
    "scaffold a local prayer under a path source\n\n" +
    "Usage: pray prayer init [name] [--path DIR]\n\n" +
    "With a Prayfile, writes <name>/ under the path source directory without a version.\n" +
    "The default directory is prayers/. Use --path to choose another.\n" +
    'Declares pray "<source>/<name>" once, such as pray "local/project", inside compose when a compose block exists.\n' +
    "The default name is project. The name v1 is reserved for the distribution layout.\n" +
    "Without a Prayfile, writes a versioned package spec in the current directory.\n" +
    "Add spec.version before pray package or pray publish.\n" +
    "A compose file such as .agents/project.md remains a shortcut for one local file.",
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
