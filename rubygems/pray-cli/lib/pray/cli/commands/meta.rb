# frozen_string_literal: true

module Pray
  module CLI
    def version_command
      puts "pray #{Pray::VERSION}"
    end

    def help_command
      puts <<~HELP
        pray <command>
          version | -V | --version
          manifest
          init [--targets tool_a,tool_b]
          prayer init
          repo init
          install [--locked] [--frozen]
          add <name> [constraint] [--path PATH]
          remove <name>
          update <name>
          unlock <name>
          render [--check]
          plan
          apply
          verify [--strict]
          drift [--semantic]
          format
          package
          publish [--root PATH] [--server URL]
          serve [--root PATH] [--host HOST] [--port PORT]
          explain <name>
          outdated
          list
          tree
          clean
          trust list|show [SOURCE_URL]
          help
      HELP
    end

    def clean_command
      remove_path_if_exists(".pray/cache")
      remove_path_if_exists(".pray/vendor")
      remove_path_if_exists(".pray/state.json")
    end
  end
end
