# frozen_string_literal: true

require "json"

module Pray
  module Trust
    module_function

    def list_policy(scope:, source_url: nil, home: trust_home)
      policy = load_policy_or_default(home)
      source_url ? list_policy_for_source(policy, scope, source_url) : list_policy_all(policy, scope)
    end

    def show_policy_toml(home = trust_home)
      format_policy(load_policy_or_default(home))
    end

    def add_allowed_signing_key(key, match_prefix: nil, home: trust_home)
      normalized = normalize_key(key)
      raise Error.unsupported("signing key is empty") if normalized.empty?

      policy = load_policy_or_default(home)
      rule = match_prefix ? mutable_rule_for_match_prefix!(policy, match_prefix) : policy.default_rule
      unless rule.allowed_signing_keys.any? { |existing| normalize_key(existing) == normalized }
        rule.allowed_signing_keys << normalized
      end
      save_policy(policy, home)
    end

    def remove_allowed_signing_key(key, match_prefix: nil, home: trust_home)
      normalized = normalize_key(key)
      raise Error.unsupported("signing key is empty") if normalized.empty?

      policy = load_policy_or_default(home)
      rule = match_prefix ? mutable_rule_for_match_prefix!(policy, match_prefix) : policy.default_rule
      before = rule.allowed_signing_keys.length
      rule.allowed_signing_keys.reject! { |existing| normalize_key(existing) == normalized }
      if rule.allowed_signing_keys.length == before
        raise Error.unsupported(
          "signing key not found in allowed_signing_keys for #{match_prefix || "<default>"}"
        )
      end
      save_policy(policy, home)
    end

    def set_require_signed_commit(match_prefix, enabled, home: trust_home)
      raise Error.unsupported("match-prefix is empty") if match_prefix.to_s.strip.empty?

      policy = load_policy_or_default(home)
      mutable_rule_for_match_prefix!(policy, match_prefix).require_signed_commit = enabled
      save_policy(policy, home)
    end

    def set_allow(match_prefix, allow, home: trust_home)
      raise Error.unsupported("match-prefix is empty") if match_prefix.to_s.strip.empty?

      policy = load_policy_or_default(home)
      mutable_rule_for_match_prefix!(policy, match_prefix).allow = allow
      save_policy(policy, home)
    end

    def import_registry(source_url, match_prefix: nil, include_host_key: false, home: trust_home)
      if source_url.start_with?("pray+ssh://", "ssh+pray://")
        raise Error.unsupported("pray_ssh registry import is not implemented yet in pray-cli Ruby")
      end
      raise Error.unsupported("--host-key requires pray_ssh sources") if include_host_key

      publishers = fetch_ssh_publisher_fingerprints(source_url)
      raise Error.unsupported("no v1/ssh_publishers.json found for #{source_url}") if publishers.nil?
      if publishers.empty?
        raise Error.unsupported(
          "v1/ssh_publishers.json for #{source_url} lists no publisher fingerprints"
        )
      end

      prefix = match_prefix || source_url
      policy = load_policy_or_default(home)
      rule = mutable_rule_for_match_prefix!(policy, prefix)
      publishers_added = append_missing!(rule.allowed_publishers, publishers)
      save_policy(policy, home)
      {publishers_added: publishers_added, host_keys_added: 0}
    end

    def import_repo(source_url, repository, match_prefix: nil, home: trust_home)
      keys = repository_signing_keys(repository)
      if keys.empty?
        raise Error.unsupported(
          "no commit signing key/fingerprint found for HEAD in #{repository}"
        )
      end

      prefix = match_prefix || source_url
      policy = load_policy_or_default(home)
      rule = mutable_rule_for_match_prefix!(policy, prefix)
      added = append_missing!(rule.allowed_signing_keys, keys)
      save_policy(policy, home)
      added
    end

    def list_policy_all(policy, scope)
      lines = []
      if %i[all global].include?(scope)
        lines << format_rule_block("scope: global", policy.default_rule).rstrip
      end
      if %i[all local].include?(scope)
        if policy.rules.empty?
          lines << "scope: local\n  (no rules)"
        else
          policy.rules.sort_by { |rule| rule.match_prefix.to_s }.each do |rule|
            prefix = rule.match_prefix || "-"
            lines << format_rule_block("scope: local (#{prefix})", rule).rstrip
          end
        end
      end
      lines.join("\n\n")
    end

    def list_policy_for_source(policy, scope, source_url)
      lines = ["source: #{source_url}", ""]
      if %i[all global].include?(scope)
        lines << format_rule_block("scope: global", policy.default_rule).rstrip
        lines << ""
      end
      if %i[all local].include?(scope)
        matched = policy.rules.select do |rule|
          prefix = rule.match_prefix
          prefix && !prefix.empty? && source_url.start_with?(prefix)
        end
        matched.sort_by! { |rule| -(rule.match_prefix&.length || 0) }
        lines << if matched.empty?
          "scope: local\n  (no matching rules)"
        else
          matched.map { |rule|
            format_rule_block("scope: local (#{rule.match_prefix})", rule).rstrip
          }.join("\n")
        end
        lines << ""
      end
      if scope == :all
        effective = best_rule(policy, source_url)
        lines << if effective.match_prefix
          "effective_scope: local (#{effective.match_prefix})"
        else
          "effective_scope: global"
        end
      end
      lines.join("\n").strip
    end

    def fetch_ssh_publisher_fingerprints(source_url)
      body = read_distribution_bytes(source_url, "v1/ssh_publishers.json")
      return nil unless body

      data = JSON.parse(body)
      Array(data["publishers"]).filter_map do |entry|
        fingerprint = entry["fingerprint"].to_s
        next if fingerprint.empty?

        normalize_key(fingerprint)
      end
    rescue JSON::ParserError => error
      raise Error.parse("ssh publishers", error.message)
    end

    def read_distribution_bytes(source_url, relative_path)
      if source_url.start_with?("http://", "https://")
        begin
          return Registry.send(:http_get, Registry.send(:join_url, source_url, relative_path))
        rescue Error
          return nil
        end
      end

      root = source_url.delete_prefix("file://")
      path = File.join(root, relative_path)
      return nil unless File.file?(path)

      File.read(path)
    end

    def repository_signing_keys(repository)
      keys = []
      key = git_format(repository, "%GK")
      keys << normalize_key(key) unless key.empty?
      fingerprint = git_format(repository, "%GF")
      keys << normalize_key(fingerprint) unless fingerprint.empty?
      keys.uniq
    end

    def git_format(repository, format)
      output = IO.popen(
        ["git", "-C", repository, "log", "-1", "--format=#{format}"],
        err: File::NULL, &:read
      )
      return "" unless $?.success?

      output.to_s.strip
    end
  end
end

require_relative "trust_feed"
