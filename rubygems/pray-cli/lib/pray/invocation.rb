# frozen_string_literal: true

require "pathname"

module Pray
  module Invocation
    module_function

    def initialize(arguments)
      options, remaining = parse_global_options(arguments)
      self.context = ProjectContext.from_options(options)
      remaining
    end

    def context
      Thread.current[:pray_invocation_context]
    end

    def context=(value)
      Thread.current[:pray_invocation_context] = value
    end

    def invocation_context
      context || ProjectContext.from_current_directory
    end

    def manifest_path
      invocation_context.manifest_path
    end

    def lockfile_path
      invocation_context.lockfile_path
    end

    def project_root
      invocation_context.project_root
    end

    def resolve_current_project(options = ResolveOptions.new)
      context = invocation_context
      resolved_options = options.dup
      resolved_options.environment ||= context.environment
      Resolve.resolve_project_in_context(context.manifest_path, context.project_root, resolved_options)
    end

    def resolve_current_project_with_git_refresh_fallback(options = ResolveOptions.new, allow_git_refresh_fallback: false)
      resolve_current_project(options)
    rescue Error => error
      if allow_git_refresh_fallback &&
         !options.offline &&
         !options.refresh &&
         Resolve.resolution_may_benefit_from_git_source_refresh?(error)
        refreshed = options.dup
        refreshed.refresh = true
        resolve_current_project(refreshed)
      else
        raise
      end
    end

    def parse_global_options(arguments)
      options = ProjectInvocationOptions.new
      remaining = []
      index = 0
      while index < arguments.length
        argument = arguments[index]
        case argument
        when "--path"
          index += 1
          options.project_root = require_option_value("--path", arguments[index])
        when "--file-path"
          index += 1
          options.manifest_path = require_option_value("--file-path", arguments[index])
        when "--env", "--environment"
          index += 1
          options.environment = require_environment_value(argument, arguments[index])
        else
          if top_level_command?(argument) || argument.start_with?("-")
            remaining.concat(arguments[index..])
            break
          end
          remaining.concat(arguments[index..])
          break
        end
        index += 1
      end
      [options, remaining]
    end

    def require_option_value(flag, value)
      raise Error.usage("#{flag} requires a value") if value.nil? || value.empty?

      value
    end

    def require_environment_value(flag, value)
      raise Error.usage("#{flag} requires a value") if value.nil?

      value
    end

    def top_level_command?(token)
      %w[
        manifest init prayer repo install add remove update unlock render plan apply verify drift
        format package publish login serve confess list outdated explain vendor clean tree sync trust
        version -V --version -h --help help
      ].include?(token)
    end
  end
end
