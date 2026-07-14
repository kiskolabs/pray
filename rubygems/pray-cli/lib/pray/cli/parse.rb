# frozen_string_literal: true

module Pray
  module CLI
    def parse_command(arguments)
      arguments = arguments.dup
      flags = {
        check: extract_flag!(arguments, "--check"),
        strict: extract_flag!(arguments, "--strict"),
        semantic: extract_flag!(arguments, "--semantic"),
        locked: false,
        frozen: false,
        offline: false,
        targets: []
      }

      arguments.each do |argument|
        case argument
        when "--locked" then flags[:locked] = true
        when "--frozen" then flags[:locked] = flags[:frozen] = true
        when "--offline" then flags[:offline] = true
        end
      end
      arguments.reject! { |argument| %w[--locked --frozen --offline].include?(argument) }

      command = arguments.shift
      raise Error.usage("pray requires a command; run pray --help") unless command

      case command
      when "manifest" then [:manifest]
      when "init"
        targets = []
        while (argument = arguments.shift)
          if argument == "--targets"
            targets = arguments.shift.to_s.split(",").map(&:strip).reject(&:empty?)
          end
        end
        [:init, targets]
      when "prayer"
        raise Error.unsupported("prayer requires init") unless arguments.shift == "init"

        [:prayer_init]
      when "repo"
        raise Error.unsupported("repo requires init") unless arguments.shift == "init"

        [:repo_init]
      when "install" then [:install, flags]
      when "add" then [:add, parse_add_arguments(arguments)]
      when "remove" then [:remove, arguments.shift]
      when "update" then [:update, arguments]
      when "unlock" then [:unlock, arguments.shift]
      when "render" then [:render, flags]
      when "plan" then [:plan, arguments]
      when "apply" then [:apply]
      when "verify" then [:verify, flags]
      when "drift" then [:drift, flags]
      when "format" then [:format]
      when "package" then [:package]
      when "publish" then [:publish, parse_publish_arguments(arguments)]
      when "login" then [:unsupported, "login"]
      when "serve" then [:serve, parse_serve_arguments(arguments)]
      when "confess" then [:unsupported, "confess"]
      when "list" then [:list]
      when "outdated" then [:outdated, arguments]
      when "explain" then [:explain, arguments.shift]
      when "vendor" then [:vendor]
      when "clean" then [:clean]
      when "tree" then [:tree]
      when "sync" then [:unsupported, "sync"]
      when "trust" then parse_trust_command(arguments)
      when "version", "-V", "--version" then [:version]
      else
        raise Error.usage(Suggest.unknown_command_message(command))
      end
    end

    def extract_flag!(arguments, flag)
      found = arguments.include?(flag)
      arguments.reject! { |argument| argument == flag }
      found
    end

    def parse_add_arguments(arguments)
      name = arguments.shift
      raise Error.unsupported("add requires a package name") unless name

      constraint = nil
      path = nil
      while (argument = arguments.shift)
        case argument
        when "--path"
          path = arguments.shift
        else
          constraint ||= argument
        end
      end
      { name: name, constraint: constraint, path: path }
    end

    def parse_publish_arguments(arguments)
      roots = []
      servers = []
      while (argument = arguments.shift)
        case argument
        when "--root"
          roots << arguments.shift
        when "--server"
          servers << arguments.shift
        else
          raise Error.unsupported("unexpected publish argument: #{argument}")
        end
      end
      raise Error.unsupported("publish requires at least one --root PATH or --server URL") if roots.empty? && servers.empty?

      { roots: roots, servers: servers }
    end

    def parse_serve_arguments(arguments)
      options = { root: ".", host: "127.0.0.1", port: 7429, stdio: false }
      while (argument = arguments.shift)
        case argument
        when "--root" then options[:root] = arguments.shift
        when "--host" then options[:host] = arguments.shift
        when "--port" then options[:port] = Integer(arguments.shift)
        when "--stdio" then options[:stdio] = true
        else
          raise Error.unsupported("unexpected serve argument: #{argument}")
        end
      end
      options
    end

    def parse_trust_command(arguments)
      subcommand = arguments.shift || "list"
      case subcommand
      when "list" then [:trust_list]
      when "show" then [:trust_show, arguments.shift]
      else
        raise Error.unsupported("trust #{subcommand} is not implemented yet in pray-cli Ruby")
      end
    end
  end
end
