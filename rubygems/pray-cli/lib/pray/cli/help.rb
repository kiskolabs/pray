# frozen_string_literal: true

module Pray
  module CLI
    module Help
      DOCS_URL = "https://github.com/kiskolabs/pray"

      WORKFLOW_COMMANDS = [
        "install [--locked|--frozen|--offline]  resolve, render, and write Prayfile.lock",
        "plan [--remote]                        preview materialization changes",
        "apply                                  apply the current plan",
        "verify [--strict]                      check rendered output against the lockfile",
        "drift [--semantic]                     compare lockfile to current resolution",
        "render [--check]                       render targets without updating the lockfile",
        "format                                 canonicalize Prayfile formatting"
      ].freeze

      PACKAGE_COMMANDS = [
        "add <name> [constraint] [--path PATH]",
        "remove <name>",
        "update [package] [--major] [--latest] [--dry-run] [--json]",
        "unlock <package>",
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
        "outdated [--remote]                      show constraint vs resolved versions",
        "explain <package>                        show why a package was selected",
        "tree                                     print the dependency tree"
      ].freeze

      META_COMMANDS = [
        "init [--targets tool_a,tool_b]",
        "prayer init                            scaffold a prayer package",
        "repo init                              scaffold a distribution root",
        "manifest                               print canonical Prayfile JSON",
        "package                                build a distributable prayer archive",
        "version | -V | --version"
      ].freeze

      COMMAND_HELP = {
        "install" => <<~TEXT.strip,
          install — resolve packages, render targets, and update Prayfile.lock

          Usage: pray install [--locked|--frozen|--offline]

          --locked   require an existing lockfile
          --frozen   require lockfile to match Prayfile exactly
          --offline  use cache only
        TEXT
        "verify" => <<~TEXT.strip,
          verify — check rendered files against Prayfile.lock

          Usage: pray verify [--strict]

          Without --strict, orphan-marker warnings print to stderr but exit 0.
          With --strict, any finding fails with exit code 6.
        TEXT
        "drift" => <<~TEXT.strip,
          drift — report differences between lockfile and current resolution

          Usage: pray drift [--semantic]

          Exits with code 6 when drift is found.
        TEXT
        "update" => <<~TEXT.strip,
          update — refresh package versions within constraints

          Usage: pray update [package] [--major] [--latest] [--dry-run] [--json]
        TEXT
        "plan" => <<~TEXT.strip,
          plan — preview install/apply changes

          Usage: pray plan [--remote]
        TEXT
        "apply" => "apply — materialize the current resolution plan\n\nUsage: pray apply",
        "trust" => <<~TEXT.strip,
          trust — manage client trust policy for remote sources

          Usage: pray trust <subcommand>

          Subcommands: list, show, add-key, remove-key, set-signed, set-allow, import-repo, import-registry, check
        TEXT
        "init" => <<~TEXT.strip,
          init — create a starter Prayfile

          Usage: pray init [--targets tool_a,tool_b]
        TEXT
        "add" => <<~TEXT.strip,
          add — declare a package in Prayfile

          Usage: pray add <name> [constraint] [--path PATH]
        TEXT
        "publish" => <<~TEXT.strip,
          publish — upload packages to a registry or local root

          Usage: pray publish --root PATH [--server URL ...]
        TEXT
        "serve" => <<~TEXT.strip
          serve — run a local registry server

          Usage: pray serve [--root PATH] [--host HOST] [--port PORT] [--stdio]
        TEXT
      }.freeze

      module_function

      def print_concise_help
        puts "pray — reproducible inference input for projects"
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
        puts "Run `pray help <command>` or `pray <command> --help` for one command."
        puts "Documentation: #{DOCS_URL}"
        puts "Exit codes: 0 success; 2 usage/parse; 3 resolution; 4 integrity; 5 render; 6 verify; 8 unsupported."
      end

      def print_command_help(command)
        text = COMMAND_HELP[command]
        return false unless text

        puts text
        puts
        puts "Documentation: #{DOCS_URL}"
        true
      end

      def print_command_groups
        puts "Workflow:"
        WORKFLOW_COMMANDS.each { |line| puts "  #{line}" }
        puts "Packages:"
        PACKAGE_COMMANDS.each { |line| puts "  #{line}" }
        puts "Distribution:"
        DISTRIBUTION_COMMANDS.each { |line| puts "  #{line}" }
        puts "Trust:"
        TRUST_COMMANDS.each { |line| puts "  #{line}" }
        puts "Inspect:"
        INSPECT_COMMANDS.each { |line| puts "  #{line}" }
        puts "Meta:"
        META_COMMANDS.each { |line| puts "  #{line}" }
        puts "Global flags: --no-input (disable prompts), --rm (ephemeral home), --trust [--global]"
      end
    end
  end
end
