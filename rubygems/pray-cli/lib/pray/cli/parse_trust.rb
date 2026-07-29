# frozen_string_literal: true

module Pray
  module CLI
    def parse_trust_command(arguments)
      subcommand = arguments.shift
      raise Error.unsupported("trust requires a subcommand") unless subcommand

      case subcommand
      when "list" then [:trust_list, parse_trust_list_arguments(arguments)]
      when "show"
        raise Error.unsupported("unknown trust show argument: #{arguments.first}") if arguments.any?

        [:trust_show]
      when "add-key" then [:trust_add_key, parse_trust_key_arguments(arguments, "add-key")]
      when "remove-key", "revoke"
        [:trust_remove_key, parse_trust_key_arguments(arguments, "remove-key")]
      when "set-signed" then [:trust_set_signed, parse_trust_set_signed_arguments(arguments)]
      when "set-allow" then [:trust_set_allow, parse_trust_set_allow_arguments(arguments)]
      when "import-repo" then [:trust_import_repo, parse_trust_import_repo_arguments(arguments)]
      when "import-registry"
        [:trust_import_registry, parse_trust_import_registry_arguments(arguments)]
      when "check"
        source = arguments.shift
        raise Error.unsupported("trust check accepts at most one SOURCE argument") if arguments.any?

        [:trust_check, source]
      else
        raise Error.unsupported("unknown trust command: #{subcommand}")
      end
    end

    def parse_trust_list_arguments(arguments)
      scope = :all
      source_url = nil
      arguments.each do |argument|
        case argument
        when "--global" then scope = :global
        when "--local" then scope = :local
        when /\A-/ then raise Error.unsupported("unknown trust list argument: #{argument}")
        else source_url = argument
        end
      end
      {scope: scope, source_url: source_url}
    end

    def parse_trust_key_arguments(arguments, label)
      key = arguments.shift
      raise Error.unsupported("trust #{label} requires KEY") unless key

      match_prefix = nil
      while (argument = arguments.shift)
        if argument == "--match-prefix"
          match_prefix = arguments.shift
          raise Error.unsupported("--match-prefix requires VALUE") unless match_prefix
        else
          raise Error.unsupported("unknown trust #{label} argument: #{argument}")
        end
      end
      {key: key, match_prefix: match_prefix}
    end

    def parse_trust_set_signed_arguments(arguments)
      match_prefix = nil
      enabled = true
      while (argument = arguments.shift)
        case argument
        when "--match-prefix"
          match_prefix = arguments.shift
          raise Error.unsupported("--match-prefix requires VALUE") unless match_prefix
        when "--enabled"
          value = arguments.shift
          raise Error.unsupported("--enabled requires true|false") unless value

          enabled = parse_bool_flag(value)
        else
          raise Error.unsupported("unknown trust set-signed argument: #{argument}")
        end
      end
      raise Error.unsupported("trust set-signed requires --match-prefix PREFIX") unless match_prefix

      {match_prefix: match_prefix, enabled: enabled}
    end

    def parse_trust_set_allow_arguments(arguments)
      match_prefix = nil
      allow = true
      while (argument = arguments.shift)
        case argument
        when "--match-prefix"
          match_prefix = arguments.shift
          raise Error.unsupported("--match-prefix requires VALUE") unless match_prefix
        when "--allow"
          value = arguments.shift
          raise Error.unsupported("--allow requires true|false") unless value

          allow = parse_bool_flag(value)
        else
          raise Error.unsupported("unknown trust set-allow argument: #{argument}")
        end
      end
      raise Error.unsupported("trust set-allow requires --match-prefix PREFIX") unless match_prefix

      {match_prefix: match_prefix, allow: allow}
    end

    def parse_trust_import_repo_arguments(arguments)
      source_url = arguments.shift
      raise Error.unsupported("trust import-repo requires SOURCE_URL") unless source_url

      match_prefix = nil
      while (argument = arguments.shift)
        if argument == "--match-prefix"
          match_prefix = arguments.shift
          raise Error.unsupported("--match-prefix requires VALUE") unless match_prefix
        else
          raise Error.unsupported("unknown trust import-repo argument: #{argument}")
        end
      end
      {source_url: source_url, match_prefix: match_prefix}
    end

    def parse_trust_import_registry_arguments(arguments)
      source_url = arguments.shift
      raise Error.unsupported("trust import-registry requires SOURCE_URL") unless source_url

      match_prefix = nil
      include_host_key = source_url.start_with?("pray+ssh://", "ssh+pray://")
      while (argument = arguments.shift)
        case argument
        when "--match-prefix"
          match_prefix = arguments.shift
          raise Error.unsupported("--match-prefix requires VALUE") unless match_prefix
        when "--host-key" then include_host_key = true
        when "--no-host-key" then include_host_key = false
        else
          raise Error.unsupported("unknown trust import-registry argument: #{argument}")
        end
      end
      {source_url: source_url, match_prefix: match_prefix, include_host_key: include_host_key}
    end

    def parse_bool_flag(value)
      case value.downcase
      when "true", "1", "yes", "on" then true
      when "false", "0", "no", "off" then false
      else
        raise Error.unsupported("expected true|false, got #{value}")
      end
    end
  end
end
