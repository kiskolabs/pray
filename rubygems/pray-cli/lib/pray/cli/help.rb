# frozen_string_literal: true

module Pray
  module CLI
    module Help
      WORKFLOW_COMMANDS = [
        "install [--locked|--frozen|--offline]  keep locked versions, re-embed listed locals",
        "plan [--remote]                        dry-run of install: dest after patch",
        "apply                                  apply the current plan",
        "verify [--strict]                      dest managed spans versus Prayfile.lock",
        "drift [--semantic]                     dest versus lock, then dest versus a fresh render",
        "render [--check]                       write dest from current inputs",
        "format, fmt                            rewrite Prayfile to recommended destination DSL"
      ].freeze

      PACKAGE_COMMANDS = [
        "add <name> [constraint] [--path PATH]  declare a package in Prayfile",
        "remove <name>                          remove a package from Prayfile",
        "update [package]                       re-resolve packages and git sources within constraints",
        "unlock <package>                       clear a locked package pin",
        "vendor                                 copy resolved packages locally",
        "clean                                  remove local cache and vendor trees"
      ].freeze

      DISTRIBUTION_COMMANDS = [
        "publish [--root PATH|--server URL|--to NAME] [--dry-run]",
        "login --server URL --email EMAIL",
        "serve [--root PATH | --to NAME] [--host HOST] [--port PORT] [--stdio]",
        "sync [--root PATH] [--peer URL ...]",
        "confess <package> | --from-lock SPAN_ID [--accepted|--rejected]"
      ].freeze

      TRUST_COMMANDS = [
        "trust list|show|add-key|remove-key|set-signed|set-allow|import-repo|import-registry|check"
      ].freeze

      INSPECT_COMMANDS = [
        "list                                   list declared packages",
        "outdated [--remote]                    show constraint vs resolved versions",
        "explain <package>                      show why a package was selected",
        "tree                                   print the dependency tree"
      ].freeze

      META_COMMANDS = [
        "init [--targets tool_a,tool_b]         create a starter Prayfile",
        "prayer init [name] [--path DIR]       scaffold a local prayer",
        "repo init                              scaffold a distribution root",
        "manifest                               print canonical Prayfile JSON",
        "package                                build a distributable prayer archive",
        "version | -V | --version               print the pray CLI version"
      ].freeze

      GLOBAL_OPTIONS = [
        "--no-input            disable prompts",
        "--rm                  use an ephemeral home directory",
        "--trust [--global]    import trust on first use"
      ].freeze

      COMMAND_HELP = {
        "install" => <<~TEXT.strip,
          keep locked package versions and re-embed listed local compose sources

          Usage: pray install [--locked|--frozen|--offline]

          Install renders dest and writes Prayfile.lock. Package versions stay at the lock.
          Listed local compose files are re-embedded and their span checksums refresh.
          A path package with spec.upstream copies upstream files when the path tree has no content yet.
          Use pray update to re-resolve package versions and git sources.
          Use pray plan to preview a write.

          --locked   require an existing lockfile
          --frozen   fail if dest or lock would change
          --offline  use cache only
        TEXT
        "verify" => <<~TEXT.strip,
          compare dest managed spans versus Prayfile.lock

          Usage: pray verify [--strict]

          Read-only integrity check. It does not compare dest to a fresh render from current sources.
          A changed local compose file is clean here when dest and lock still match.
          Use pray drift to compare dest with a fresh render. Use pray plan to preview a write.

          Without --strict, orphan-marker warnings print to stderr but exit 0.
          With --strict, any finding fails with exit code 6.
        TEXT
        "drift" => <<~TEXT.strip,
          compare dest versus Prayfile.lock, then dest versus a fresh render

          Usage: pray drift [--semantic]

          Read-only finding report from current packages and listed local files. Not a line diff.
          Exits with code 6 when drift is found.
          Use pray plan to preview a write. Use pray verify for dest versus lock alone.
        TEXT
        "render" => <<~TEXT.strip,
          write dest from current inputs

          Usage: pray render [--check]

          Without --check, render writes dest. It does not write Prayfile.lock.
          Use pray plan to preview a write.
          --check writes nothing. It fails unless dest already matches a fresh render from current inputs.
        TEXT
        "format" => <<~TEXT.strip,
          rewrite Prayfile to recommended destination DSL

          Usage: pray format
                 pray fmt
        TEXT
        "fmt" => <<~TEXT.strip,
          rewrite Prayfile to recommended destination DSL

          Usage: pray format
                 pray fmt
        TEXT
        "update" => <<~TEXT.strip,
          re-resolve package versions and git sources within current constraints

          Usage: pray update [package] [--latest] [--latest --dry-run]

          With no package name, update re-resolves the whole graph within constraints
          and refreshes git source revisions. A package name unlocks that package only.
          Path-fork packages can refresh from upstream.
          Listed local compose files are re-embedded by pray install, not by update.

          --latest adjusts constraints to allow the latest package versions.
          --latest also rewrites exact spec.upstream pins in path packages, then refreshes those trees.
          --latest --dry-run prints the planned rewrite and does not write.
        TEXT
        "plan" => <<~TEXT.strip,
          preview what install or apply would write after patch

          Usage: pray plan [--remote]

          This is the dry-run of a write. It does not write dest or the lock.
        TEXT
        "apply" => "materialize the current resolution plan\n\nUsage: pray apply",
        "add" => <<~TEXT.strip,
          declare a package in Prayfile

          Usage: pray add <name> [constraint] [--path PATH]
        TEXT
        "remove" => "remove a package from Prayfile\n\nUsage: pray remove <name>",
        "unlock" => "clear a locked package pin\n\nUsage: pray unlock <package>",
        "vendor" => "copy resolved packages locally\n\nUsage: pray vendor",
        "clean" => "remove local cache and vendor trees, or only unused registry entries\n\nUsage: pray clean [--unused]",
        "publish" => <<~TEXT.strip,
          upload path packages to a registry or local root

          Usage: pray publish [--root PATH] [--server URL ...] [--to NAME] [--dry-run]

          Prayfile publish remotes supply dests when flags are omitted.
        TEXT
        "login" => <<~TEXT.strip,
          authenticate to a registry server

          Usage: pray login --server URL --email EMAIL
        TEXT
        "serve" => <<~TEXT.strip,
          run a local registry server

          Usage: pray serve [--root PATH | --to NAME] [--host HOST] [--port PORT] [--stdio]
        TEXT
        "sync" => <<~TEXT.strip,
          sync packages with peer registries

          Usage: pray sync [--root PATH] [--peer URL ...]
        TEXT
        "confess" => <<~TEXT.strip,
          record an acceptance or rejection for a package confession

          Usage: pray confess <package> | --from-lock SPAN_ID [--accepted|--rejected]
        TEXT
        "trust" => <<~TEXT.strip,
          manage client trust policy for remote sources

          Usage: pray trust <subcommand>

          Subcommands: list, show, add-key, remove-key, set-signed, set-allow, import-repo, import-registry, check
        TEXT
        "list" => "list declared packages\n\nUsage: pray list",
        "outdated" => <<~TEXT.strip,
          show constraint vs resolved versions

          Usage: pray outdated [--remote]

          Path-fork files that differ from the locked upstream are listed.
        TEXT
        "explain" => <<~TEXT.strip,
          show why a package was selected

          Usage: pray explain <package>
        TEXT
        "tree" => "print the dependency tree\n\nUsage: pray tree",
        "init" => <<~TEXT.strip,
          create a starter Prayfile

          Usage: pray init [--targets tool_a,tool_b]
        TEXT
        "prayer" => <<~TEXT.strip,
          scaffold a local prayer under a path source

          Usage: pray prayer init [name] [--path DIR]

          With a Prayfile, writes <name>/ under the path source directory without a version.
          The default directory is prayers/. Use --path to choose another.
          Declares pray "<source>/<name>" once, such as pray "local/project", inside compose when a compose block exists.
          The default name is project. The name v1 is reserved for the distribution layout.
          Named prayers may sit beside prayers/v1/. Keep sources under prayers/<name>/.
          Without a Prayfile, writes a versioned package spec in the current directory.
          Add spec.version before pray package or pray publish.
          A compose file such as .agents/project.md remains a shortcut for one local file.
        TEXT
        "repo" => <<~TEXT.strip,
          scaffold a distribution root

          Usage: pray repo init

          Writes prayers/v1/. Git install discovers that catalog. Named prayers may sit beside v1/.
          Keep sources under prayers/<name>/.
        TEXT
        "manifest" => "print canonical Prayfile JSON\n\nUsage: pray manifest",
        "package" => "build a distributable prayer archive\n\nUsage: pray package",
        "version" => <<~TEXT.strip,
          print the pray CLI version

          Usage: pray version
                 pray -V | --version
        TEXT
        "help" => <<~TEXT.strip
          show help for pray or one command

          Usage: pray help [command]
                 pray [command] --help
        TEXT
      }.freeze

      module_function

      def print_concise_help
        puts "Usage: pray [OPTIONS] <COMMAND>"
        puts
        puts "Declare shared instructions in Prayfile, lock versions, and render tool-specific output."
        puts
        puts "Getting started:"
        puts "  pray init"
        puts "  pray install"
        puts "  pray plan"
        puts "  pray apply"
        puts "  pray verify"
        puts
        print_command_groups
        puts
        puts "Options:"
        GLOBAL_OPTIONS.each { |line| puts "  #{line}" }
        puts
        puts "See 'pray help <command>' or 'pray <command> --help' for details on a command."
      end

      def print_command_help(command)
        text = COMMAND_HELP[command]
        return false unless text

        puts text
        true
      end

      def print_command_groups
        print_group("Workflow", WORKFLOW_COMMANDS)
        puts
        print_group("Packages", PACKAGE_COMMANDS)
        puts
        print_group("Distribution", DISTRIBUTION_COMMANDS)
        puts
        print_group("Trust", TRUST_COMMANDS)
        puts
        print_group("Inspect", INSPECT_COMMANDS)
        puts
        print_group("Meta", META_COMMANDS)
      end

      def print_group(title, lines)
        puts "#{title}:"
        lines.each { |line| puts "  #{line}" }
      end
    end
  end
end
