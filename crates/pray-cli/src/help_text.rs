pub(crate) const WORKFLOW_COMMANDS: &[&str] = &[
    "install [--locked|--frozen|--offline|--strict]  keep locked versions, re-embed listed locals",
    "plan [--remote]                        dry-run of install: dest after patch",
    "apply                                  apply the current plan",
    "verify [--strict]                      dest managed spans versus Prayfile.lock",
    "drift [--semantic]                     dest versus lock, then dest versus a fresh render",
    "render [--check]                       write dest and Prayfile.lock from current inputs",
    "format|fmt                             rewrite Prayfile to recommended destination DSL",
];

pub(crate) const PACKAGE_COMMANDS: &[&str] = &[
    "add <name> [constraint] [--path PATH]  declare a package in Prayfile",
    "remove <name>                          remove a package from Prayfile",
    "update [package] [--major|--latest|--dry-run|--json]  re-resolve packages and git sources within constraints",
    "unlock <package>                       clear a locked package pin",
    "vendor                                 copy resolved packages locally",
    "clean [--unused]                       remove local state or unused registry entries",
];

#[cfg(feature = "auth")]
pub(crate) const DISTRIBUTION_COMMANDS: &[&str] = &[
    "publish [--root PATH|--server URL|--to NAME] [--dry-run] [--signing-key PATH] [--resign]",
    "yank <package> <version> [--root PATH | --to NAME] [--undo]",
    "token create|revoke [--root PATH | --to NAME] ...",
    "login --server URL --email EMAIL",
    "serve [--root PATH | --to NAME] [--host HOST] [--port PORT] [--stdio] [--allow-open-push]",
    "sync [--root PATH] [--peer URL ...]",
    "confess <package> | --from-lock SPAN_ID [--accepted|--rejected]",
];

#[cfg(not(feature = "auth"))]
pub(crate) const DISTRIBUTION_COMMANDS: &[&str] = &[
    "publish [--root PATH|--server URL|--to NAME] [--dry-run] [--signing-key PATH] [--resign]",
    "yank <package> <version> [--root PATH | --to NAME] [--undo]",
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
    "prayer init [name] [--path DIR]       scaffold a local prayer",
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
            "keep locked package versions and re-embed listed local compose sources\n\n\
             Usage: pray install [--locked|--frozen|--offline|--strict]\n\n\
             Install renders dest and writes Prayfile.lock. Package versions stay at the lock.\n\
             Listed local compose files are re-embedded and their span checksums refresh.\n\
             A path package with spec.upstream copies upstream files when the path tree has no content yet.\n\
             Use pray update to re-resolve package versions and git sources.\n\
             Use pray plan to preview a write.\n\n\
             --locked   require an existing lockfile\n\
             --frozen   fail if dest or lock would change\n\
             --offline  use cache only\n\
             --strict   fail if a locked package version is yanked",
        ),
        "plan" => Some(
            "preview what install or apply would write after patch\n\n\
             Usage: pray plan [--remote]\n\n\
             This is the dry-run of a write. It does not write dest or the lock.",
        ),
        "apply" => Some("materialize the current resolution plan\n\nUsage: pray apply"),
        "verify" => Some(
            "compare dest managed spans versus Prayfile.lock\n\n\
             Usage: pray verify [--strict]\n\n\
             Read-only integrity check. It does not compare dest to a fresh render from current sources.\n\
             A changed local compose file is clean here when dest and lock still match.\n\
             Use pray drift to compare dest with a fresh render. Use pray plan to preview a write.\n\n\
             Without --strict, orphan-marker warnings print to stderr but exit 0.\n\
             With --strict, any finding fails with exit code 6.",
        ),
        "drift" => Some(
            "compare dest versus Prayfile.lock, then dest versus a fresh render\n\n\
             Usage: pray drift [--semantic]\n\n\
             Read-only finding report from current packages and listed local files. Not a line diff.\n\
             Exits with code 6 when drift is found.\n\
             --semantic prints package version arrows only.\n\
             Use pray plan to preview a write. Use pray verify for dest versus lock alone.",
        ),
        "render" => Some(
            "write dest and Prayfile.lock from current inputs\n\n\
             Usage: pray render [--check]\n\n\
             Without --check, render writes dest and the lock. Use pray plan to preview a write.\n\
             --check writes nothing. It fails unless dest already matches a fresh render from current inputs.\n\
             That is a dest identity gate, not a preview.",
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
            "re-resolve package versions and git sources within current constraints\n\n\
             Usage: pray update [package] [--major] [--latest] [--dry-run] [--json]\n\n\
             With no package name, update re-resolves the whole graph within constraints\n\
             and refreshes git source revisions. A package name unlocks that package only.\n\
             Path-fork packages can refresh from upstream.\n\
             Listed local compose files are re-embedded by pray install, not by update.\n\n\
             --major requires a package name.\n\
             --latest rewrites Prayfile constraints and exact spec.upstream pins to admit the latest versions, then refreshes those trees.\n\
             --latest --dry-run previews those versions and checks destination conflicts.\n\
             If a destination conflicts, inspect it and move it aside before retrying.\n\
             --json prints machine-readable output.\n\
             For files from an older pray, install the original package version first.",
        ),
        "unlock" => Some("clear a locked package pin\n\nUsage: pray unlock <package>"),
        "vendor" => Some("copy resolved packages locally\n\nUsage: pray vendor"),
        "clean" => Some(
            "remove local cache and vendor trees, or only unused registry entries\n\n\
             Usage: pray clean [--unused]",
        ),
        "publish" => Some(
            "upload path packages to a registry or local root\n\n\
             Usage: pray publish [--root PATH] [--server URL ...] [--to NAME] [--dry-run] [--signing-key PATH] [--resign]\n\n\
             Prayfile publish remotes supply dests when flags are omitted.\n\
             Prefer --signing-key PATH or PRAY_SIGNING_KEY (32-byte ed25519 seed).\n\
             Without a signing key, publish records a legacy content digest.\n\
             --resign uses that key for unchanged versions in a local root.",
        ),
        "yank" => Some(
            "mark or unmark a published version as yanked in a distribution root\n\n\
             Usage: pray yank <package> <version> [--root PATH | --to NAME] [--undo]\n\n\
             Yank flips metadata only; artifact bytes stay immutable.\n\
             New resolves skip yanked versions. Locked installs may continue with a warning;\n\
             use pray install --strict to refuse them.",
        ),
        #[cfg(feature = "auth")]
        "token" => Some(
            "mint or revoke scoped publish tokens for a distribution root\n\n\
             Usage: pray token create [--root PATH | --to NAME] --email EMAIL [--scope publish]\n\
                    pray token revoke [--root PATH | --to NAME] TOKEN\n\n\
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
             Usage: pray serve [--root PATH | --to NAME] [--host HOST] [--port PORT] [--stdio] [--allow-open-push]",
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
             Usage: pray outdated [--remote]\n\n\
             Path-fork files that differ from the locked upstream are listed.",
        ),
        "explain" => Some(
            "show why a package was selected\n\n\
             Usage: pray explain <package>",
        ),
        "tree" => Some("print the dependency tree\n\nUsage: pray tree"),
        "init" => Some("create a starter Prayfile\n\nUsage: pray init [--targets tool_a,tool_b]"),
        "prayer" => Some(
            "scaffold a local prayer under a path source\n\n\
             Usage: pray prayer init [name] [--path DIR]\n\n\
             With a Prayfile, writes <name>/ under the path source directory without a version.\n\
             The default directory is prayers/. Use --path to choose another.\n\
             Declares pray \"<source>/<name>\" once, such as pray \"local/project\", inside compose when a compose block exists.\n\
             The default name is project. The name v1 is reserved for the distribution layout.\n\
             Without a Prayfile, writes a versioned package spec in the current directory.\n\
             Add spec.version before pray package or pray publish.\n\
             A compose file such as .agents/project.md remains a shortcut for one local file.",
        ),
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
