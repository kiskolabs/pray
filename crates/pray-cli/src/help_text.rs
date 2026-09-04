pub(crate) const WORKFLOW_COMMANDS: &[&str] = &[
    "install [--locked|--frozen|--offline|--strict]  resolve, render, and write Prayfile.lock",
    "plan [--remote]                        preview materialization changes",
    "apply                                  apply the current plan",
    "verify [--strict]                      check rendered output against the lockfile",
    "drift [--semantic]                     compare lockfile to current resolution",
    "render [--check]                       render targets without updating the lockfile",
    "format|fmt                             rewrite Prayfile to recommended destination DSL",
];

pub(crate) const PACKAGE_COMMANDS: &[&str] = &[
    "add <name> [constraint] [--path PATH]  declare a package in Prayfile",
    "remove <name>                          remove a package from Prayfile",
    "update [package] [--major] [--latest] [--dry-run] [--json]",
    "unlock <package>                       clear a locked package pin",
    "vendor                                 copy resolved packages locally",
    "clean [--unused]                       remove local state or unused registry entries",
];

#[cfg(feature = "auth")]
pub(crate) const DISTRIBUTION_COMMANDS: &[&str] = &[
    "publish --root PATH [--server URL ...] [--signing-key PATH]",
    "yank <package> <version> --root PATH [--undo]",
    "token create|revoke --root PATH ...",
    "login --server URL --email EMAIL",
    "serve [--root PATH] [--host HOST] [--port PORT] [--stdio] [--allow-open-push]",
    "sync [--root PATH] [--peer URL ...]",
    "confess <package> | --from-lock SPAN_ID [--accepted|--rejected]",
];

#[cfg(not(feature = "auth"))]
pub(crate) const DISTRIBUTION_COMMANDS: &[&str] = &[
    "publish --root PATH [--server URL ...] [--signing-key PATH]",
    "yank <package> <version> --root PATH [--undo]",
    "login --server URL --email EMAIL",
    "sync [--root PATH] [--peer URL ...]",
    "confess <package> | --from-lock SPAN_ID [--accepted|--rejected]",
];

pub(crate) const TRUST_COMMANDS: &[&str] =
    &["trust list|show|add-key|remove-key|set-signed|set-require-signed-packages|set-allow|import-repo|import-registry|check"];

pub(crate) const INSPECT_COMMANDS: &[&str] = &[
    "list                                   list declared packages",
    "search <query> [--source|--root|--url]  find packages in a distribution index",
    "outdated [--remote]                    show constraint vs resolved versions",
    "explain <package>                      show why a package was selected",
    "tree                                   print the dependency tree",
];

pub(crate) const META_COMMANDS: &[&str] = &[
    "init [--targets tool_a,tool_b]         create a starter Prayfile",
    "prayer init                            scaffold a prayer package",
    "repo init                              scaffold a distribution root",
    "manifest                               print canonical Prayfile JSON",
    "package                                build a distributable prayer archive",
    "completion bash|zsh|fish               print shell completion script",
    "upgrade                                install the latest pray CLI release",
    "version | -V | --version               print the pray CLI version",
];

pub(crate) const GLOBAL_OPTIONS: &[&str] = &[
    "--path PATH           project root (default: current directory)",
    "--file-path PATH      Prayfile path",
    "--env NAME            environment name",
    "--no-input            disable prompts",
    "--rm                  use an ephemeral home directory",
    "--trust [--global]    import trust on first use",
];

pub(crate) fn command_help_text(command: &str) -> Option<&'static str> {
    match command {
        "install" => Some(
            "resolve packages, render targets, and update Prayfile.lock\n\n\
             Usage: pray install [--locked|--frozen|--offline|--strict]\n\n\
             --locked   require an existing lockfile\n\
             --frozen   require lockfile to match Prayfile exactly\n\
             --offline  use cache only\n\
             --strict   fail if a locked package version is yanked",
        ),
        "plan" => Some("preview install/apply changes\n\nUsage: pray plan [--remote]"),
        "apply" => Some("materialize the current resolution plan\n\nUsage: pray apply"),
        "verify" => Some(
            "check rendered files against Prayfile.lock\n\n\
             Usage: pray verify [--strict]\n\n\
             Without --strict, orphan-marker warnings print to stderr but exit 0.\n\
             With --strict, any finding fails with exit code 6.",
        ),
        "drift" => Some(
            "report differences between lockfile and current resolution\n\n\
             Usage: pray drift [--semantic]\n\n\
             Exits with code 6 when drift is found.",
        ),
        "render" => Some(
            "render targets without updating the lockfile\n\n\
             Usage: pray render [--check]",
        ),
        "format" | "fmt" => Some(
            "rewrite Prayfile to recommended destination DSL\n\n\
             Usage: pray format\n       pray fmt",
        ),
        "add" => Some(
            "declare a package in Prayfile\n\n\
             Usage: pray add <name> [constraint] [--path PATH]",
        ),
        "remove" => Some("remove a package from Prayfile\n\nUsage: pray remove <name>"),
        "update" => Some(
            "refresh package versions within constraints\n\n\
             Usage: pray update [package] [--major] [--latest] [--dry-run] [--json]",
        ),
        "unlock" => Some("clear a locked package pin\n\nUsage: pray unlock <package>"),
        "vendor" => Some("copy resolved packages locally\n\nUsage: pray vendor"),
        "clean" => Some(
            "remove local cache and vendor trees, or only unused registry entries\n\n\
             Usage: pray clean [--unused]",
        ),
        "publish" => Some(
            "upload packages to a registry or local root\n\n\
             Usage: pray publish --root PATH [--server URL ...] [--signing-key PATH]\n\n\
             Prefer --signing-key PATH or PRAY_SIGNING_KEY (32-byte ed25519 seed).\n\
             Without a signing key, publish records a legacy content digest.",
        ),
        "yank" => Some(
            "mark or unmark a published version as yanked in a distribution root\n\n\
             Usage: pray yank <package> <version> --root PATH [--undo]\n\n\
             Yank flips metadata only; artifact bytes stay immutable.\n\
             New resolves skip yanked versions. Locked installs may continue with a warning;\n\
             use pray install --strict to refuse them.",
        ),
        #[cfg(feature = "auth")]
        "token" => Some(
            "mint or revoke scoped publish tokens for a distribution root\n\n\
             Usage: pray token create --root PATH --email EMAIL [--scope publish]\n\
                    pray token revoke --root PATH TOKEN\n\n\
             Use the printed token as PRAY_PUBLISH_TOKEN for pray publish --server.",
        ),
        "search" => Some(
            "search a distribution index for package names\n\n\
             Usage: pray search <query> [--source NAME | --root PATH | --url URL]\n\n\
             Matches package names (substring, case-insensitive). Optional summaries come from\n\
             package metadata when available. No ranking.",
        ),
        "login" => Some(
            "authenticate to a registry server\n\n\
             Usage: pray login --server URL --email EMAIL \\\n\
                    (--passkey-key PATH --credential-id ID | --ssh-agent --public-key PATH)",
        ),
        #[cfg(feature = "auth")]
        "serve" => Some(
            "run a local registry server\n\n\
             Usage: pray serve [--root PATH] [--host HOST] [--port PORT] [--stdio] [--allow-open-push]",
        ),
        "sync" => Some(
            "sync packages with peer registries\n\n\
             Usage: pray sync [--root PATH] [--peer URL ...]",
        ),
        "confess" => Some(
            "record an acceptance or rejection for a package confession\n\n\
             Usage: pray confess <package> | --from-lock SPAN_ID [--accepted|--rejected]",
        ),
        "trust" => Some(
            "manage client trust policy for remote sources\n\n\
             Usage: pray trust <subcommand>\n\n\
             Subcommands: list, show, add-key, remove-key, set-signed, \
             set-require-signed-packages, set-allow, import-repo, import-registry, check",
        ),
        "list" => Some("list declared packages\n\nUsage: pray list"),
        "outdated" => Some(
            "show constraint vs resolved versions\n\n\
             Usage: pray outdated [--remote]",
        ),
        "explain" => Some(
            "show why a package was selected\n\n\
             Usage: pray explain <package>",
        ),
        "tree" => Some("print the dependency tree\n\nUsage: pray tree"),
        "init" => Some("create a starter Prayfile\n\nUsage: pray init [--targets tool_a,tool_b]"),
        "prayer" => Some("scaffold a prayer package\n\nUsage: pray prayer init"),
        "repo" => Some("scaffold a distribution root\n\nUsage: pray repo init"),
        "manifest" => Some("print canonical Prayfile JSON\n\nUsage: pray manifest"),
        "package" => Some("build a distributable prayer archive\n\nUsage: pray package"),
        "upgrade" => Some(
            "install the latest pray CLI release\n\n\
             Usage: pray upgrade\n\n\
             Runs `cargo install pray-cli --locked --force`.",
        ),
        "version" => Some("print the pray CLI version\n\nUsage: pray version\n       pray -V | --version"),
        "completion" => Some(
            "print a shell completion script\n\n\
             Usage: pray completion bash|zsh|fish\n\n\
             Redirect stdout into your shell completion directory.",
        ),
        "help" => Some(
            "show help for pray or one command\n\n\
             Usage: pray help [command]\n       pray [command] --help",
        ),
        _ => None,
    }
}
