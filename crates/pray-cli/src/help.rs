use std::io::{self, Write};

const DOCS_URL: &str = "https://github.com/kiskolabs/pray";

pub fn print_concise_help() {
    let _ = writeln!(
        io::stdout(),
        "pray — reproducible inference input for projects"
    );
    let _ = writeln!(io::stdout());
    let _ = writeln!(
        io::stdout(),
        "Declare shared instructions in Prayfile, lock versions, and render tool-specific output."
    );
    let _ = writeln!(io::stdout());
    let _ = writeln!(io::stdout(), "Getting started:");
    let _ = writeln!(io::stdout(), "  pray init");
    let _ = writeln!(io::stdout(), "  pray install");
    let _ = writeln!(io::stdout(), "  pray plan");
    let _ = writeln!(io::stdout(), "  pray apply");
    let _ = writeln!(io::stdout(), "  pray verify");
    let _ = writeln!(io::stdout());
    print_command_groups();
    let _ = writeln!(io::stdout());
    let _ = writeln!(
        io::stdout(),
        "Run `pray help <command>` or `pray <command> --help` for one command."
    );
    let _ = writeln!(io::stdout(), "Documentation: {DOCS_URL}");
    let _ = writeln!(
        io::stdout(),
        "Exit codes: 0 success; 2 usage/parse; 3 resolution; 4 integrity; 5 render; 6 verify; 8 unsupported."
    );
}

pub fn print_command_help(command: &str) -> bool {
    if let Some(text) = command_help_text(command) {
        let _ = writeln!(io::stdout(), "{text}");
        let _ = writeln!(io::stdout());
        let _ = writeln!(io::stdout(), "Documentation: {DOCS_URL}");
        true
    } else {
        false
    }
}

fn print_command_groups() {
    let _ = writeln!(io::stdout(), "Workflow:");
    for line in WORKFLOW_COMMANDS {
        let _ = writeln!(io::stdout(), "  {line}");
    }
    let _ = writeln!(io::stdout(), "Packages:");
    for line in PACKAGE_COMMANDS {
        let _ = writeln!(io::stdout(), "  {line}");
    }
    let _ = writeln!(io::stdout(), "Distribution:");
    for line in DISTRIBUTION_COMMANDS {
        let _ = writeln!(io::stdout(), "  {line}");
    }
    let _ = writeln!(io::stdout(), "Trust:");
    for line in TRUST_COMMANDS {
        let _ = writeln!(io::stdout(), "  {line}");
    }
    let _ = writeln!(io::stdout(), "Inspect:");
    for line in INSPECT_COMMANDS {
        let _ = writeln!(io::stdout(), "  {line}");
    }
    let _ = writeln!(io::stdout(), "Meta:");
    for line in META_COMMANDS {
        let _ = writeln!(io::stdout(), "  {line}");
    }
    let _ = writeln!(
        io::stdout(),
        "Global flags: --path PATH, --file-path PATH, --env NAME, --no-input (disable prompts), --rm (ephemeral home), --trust [--global]"
    );
}

const WORKFLOW_COMMANDS: &[&str] = &[
    "install [--locked|--frozen|--offline]  resolve, render, and write Prayfile.lock",
    "plan [--remote]                        preview materialization changes",
    "apply                                  apply the current plan",
    "verify [--strict]                      check rendered output against the lockfile",
    "drift [--semantic]                     compare lockfile to current resolution",
    "render [--check]                       render targets without updating the lockfile",
    "format                                 canonicalize Prayfile formatting",
];

const PACKAGE_COMMANDS: &[&str] = &[
    "add <name> [constraint] [--path PATH]",
    "remove <name>",
    "update [package] [--major] [--latest] [--dry-run] [--json]",
    "unlock <package>",
    "vendor                                 copy resolved packages locally",
    "clean                                  remove local cache and vendor trees",
];

const DISTRIBUTION_COMMANDS: &[&str] = &[
    "publish --root PATH [--server URL ...]",
    "login --server URL --email EMAIL",
    "serve [--root PATH] [--host HOST] [--port PORT] [--stdio]",
    "sync [--root PATH] [--peer URL ...]",
    "confess <package> | --from-lock SPAN_ID [--accepted|--rejected]",
];

const TRUST_COMMANDS: &[&str] =
    &["trust list|show|add-key|remove-key|set-signed|set-allow|import-repo|import-registry|check"];

const INSPECT_COMMANDS: &[&str] = &[
    "list                                   list declared packages",
    "outdated [--remote]                      show constraint vs resolved versions",
    "explain <package>                        show why a package was selected",
    "tree                                     print the dependency tree",
];

const META_COMMANDS: &[&str] = &[
    "init [--targets tool_a,tool_b]",
    "prayer init                            scaffold a prayer package",
    "repo init                              scaffold a distribution root",
    "manifest                               print canonical Prayfile JSON",
    "package                                build a distributable prayer archive",
    "version | -V | --version",
];

fn command_help_text(command: &str) -> Option<&'static str> {
    match command {
        "install" => Some(
            "install — resolve packages, render targets, and update Prayfile.lock\n\n\
             Usage: pray install [--locked|--frozen|--offline]\n\n\
             --locked   require an existing lockfile\n\
             --frozen   require lockfile to match Prayfile exactly\n\
             --offline  use cache only",
        ),
        "verify" => Some(
            "verify — check rendered files against Prayfile.lock\n\n\
             Usage: pray verify [--strict]\n\n\
             Without --strict, orphan-marker warnings print to stderr but exit 0.\n\
             With --strict, any finding fails with exit code 6.",
        ),
        "drift" => Some(
            "drift — report differences between lockfile and current resolution\n\n\
             Usage: pray drift [--semantic]\n\n\
             Exits with code 6 when drift is found.",
        ),
        "update" => Some(
            "update — refresh package versions within constraints\n\n\
             Usage: pray update [package] [--major] [--latest] [--dry-run] [--json]",
        ),
        "plan" => Some(
            "plan — preview install/apply changes\n\n\
             Usage: pray plan [--remote]",
        ),
        "apply" => Some("apply — materialize the current resolution plan\n\nUsage: pray apply"),
        "trust" => Some(
            "trust — manage client trust policy for remote sources\n\n\
             Usage: pray trust <subcommand>\n\n\
             Subcommands: list, show, add-key, remove-key, set-signed, set-allow, \
             import-repo, import-registry, check",
        ),
        "init" => Some(
            "init — create a starter Prayfile\n\n\
             Usage: pray init [--targets tool_a,tool_b]",
        ),
        "add" => Some(
            "add — declare a package in Prayfile\n\n\
             Usage: pray add <name> [constraint] [--path PATH]",
        ),
        "publish" => Some(
            "publish — upload packages to a registry or local root\n\n\
             Usage: pray publish --root PATH [--server URL ...]",
        ),
        "serve" => Some(
            "serve — run a local registry server\n\n\
             Usage: pray serve [--root PATH] [--host HOST] [--port PORT] [--stdio]",
        ),
        _ => None,
    }
}
