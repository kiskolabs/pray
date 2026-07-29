# frozen_string_literal: true

module Pray
  module CLI
    module Help
      WORKFLOW_COMMANDS = [
        "install [--locked|--frozen|--offline]  resolve, render, and write Prayfile.lock",
        "plan [--remote]                        preview materialization changes",
        "apply                                  apply the current plan",
        "verify [--strict]                      check rendered output against the lockfile",
        "drift [--semantic]                     compare lockfile to current resolution",
        "render [--check]                       render targets without updating the lockfile",
        "format, fmt                            rewrite Prayfile to recommended destination DSL"
      ].freeze

      PACKAGE_COMMANDS = [
        "add <name> [constraint] [--path PATH]  declare a package in Prayfile",
        "remove <name>                          remove a package from Prayfile",
        "update [package] [--major] [--latest] [--dry-run] [--json]",
        "unlock <package>                       clear a locked package pin",
        "vendor                                 copy resolved packages locally",
        "clean                                  remove local cache and vendor trees"
      ].freeze

      DISTRIBUTION_COMMANDS = [
        "publish --root PATH [--server URL ...]",
        "login --server URL --email EMAIL",
        "serve [--root PATH] [--host HOST] [--port PORT] [--stdio]",
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
        "prayer init                            scaffold a prayer package",
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
          resolve packages, render targets, and update Prayfile.lock

          Usage: pray install [--locked|--frozen|--offline]

          --locked   require an existing lockfile
          --frozen   require lockfile to match Prayfile exactly
          --offline  use cache only
        TEXT
        "verify" => <<~TEXT.strip,
          check rendered files against Prayfile.lock

          Usage: pray verify [--strict]

          Without --strict, orphan-marker warnings print to stderr but exit 0.
          With --strict, any finding fails with exit code 6.
        TEXT
        "drift" => <<~TEXT.strip,
          report differences between lockfile and current resolution

          Usage: pray drift [--semantic]

          Exits with code 6 when drift is found.
        TEXT
        "render" => <<~TEXT.strip,
          render targets without updating the lockfile

          Usage: pray render [--check]
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
          refresh package versions within constraints

          Usage: pray update [package] [--major] [--latest] [--dry-run] [--json]
        TEXT
        "plan" => <<~TEXT.strip,
          preview install/apply changes

          Usage: pray plan [--remote]
        TEXT
        "apply" => "materialize the current resolution plan\n\nUsage: pray apply",
        "add" => <<~TEXT.strip,
          declare a package in Prayfile

          Usage: pray add <name> [constraint] [--path PATH]
        TEXT
        "remove" => "remove a package from Prayfile\n\nUsage: pray remove <name>",
        "unlock" => "clear a locked package pin\n\nUsage: pray unlock <package>",
        "vendor" => "copy resolved packages locally\n\nUsage: pray vendor",
        "clean" => "remove local cache and vendor trees\n\nUsage: pray clean",
        "publish" => <<~TEXT.strip,
          upload packages to a registry or local root

          Usage: pray publish --root PATH [--server URL ...]
        TEXT
        "login" => <<~TEXT.strip,
          authenticate to a registry server

          Usage: pray login --server URL --email EMAIL
        TEXT
        "serve" => <<~TEXT.strip,
          run a local registry server

          Usage: pray serve [--root PATH] [--host HOST] [--port PORT] [--stdio]
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
        "prayer" => "scaffold a prayer package\n\nUsage: pray prayer init",
        "repo" => "scaffold a distribution root\n\nUsage: pray repo init",
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
