# frozen_string_literal: true

require "fileutils"
require "set"

require_relative "cli/parse"
require_relative "cli/helpers"
require_relative "cli/commands/init"
require_relative "cli/commands/workflow"
require_relative "cli/commands/packages"
require_relative "cli/commands/distribution"
require_relative "cli/commands/trust"
require_relative "cli/commands/meta"

module Pray
  module CLI
    MANIFEST_PATH = "Prayfile"
    LOCKFILE_PATH = "Prayfile.lock"

    def run(arguments)
      command = parse_command(arguments)
      dispatch(command)
    end

    def dispatch(command)
      case command
      in [:manifest] then manifest_command
      in [:init, targets] then init_command(targets)
      in [:prayer_init] then prayer_init_command
      in [:repo_init] then repo_init_command
      in [:install, flags] then install_command(flags)
      in [:add, add_args] then add_command(**add_args)
      in [:remove, name] then remove_command(name)
      in [:update, arguments] then update_command(arguments)
      in [:unlock, name] then unlock_command(name)
      in [:render, flags] then render_command(flags)
      in [:plan, arguments] then plan_command(arguments)
      in [:apply] then install_command({ locked: false, frozen: false, offline: false })
      in [:verify, flags] then verify_command(flags)
      in [:drift, flags] then drift_command(flags)
      in [:format] then format_command
      in [:package] then package_command
      in [:publish, publish_args] then publish_command(**publish_args)
      in [:unsupported, name] then raise Error.unsupported("#{name} is not implemented yet in pray-cli Ruby")
      in [:serve, serve_args] then serve_command(**serve_args)
      in [:list] then list_command
      in [:outdated, arguments] then outdated_command(arguments)
      in [:explain, name] then explain_command(name)
      in [:vendor] then raise Error.unsupported("vendor is not implemented yet in pray-cli Ruby")
      in [:clean] then clean_command
      in [:tree] then tree_command
      in [:trust_list] then trust_list_command
      in [:trust_show, source_url] then trust_show_command(source_url)
      in [:version] then version_command
      in [:help] then help_command
      end
    end

    extend self
  end
end
