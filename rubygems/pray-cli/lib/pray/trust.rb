# frozen_string_literal: true

require "toml-rb"
require "fileutils"

module Pray
  TrustRule = Struct.new(
    :match_prefix, :allow, :require_signed_commit,
    :allowed_signing_keys, :allowed_host_keys, :allowed_publishers
  ) do
    def initialize(
      match_prefix: nil, allow: true, require_signed_commit: false,
      allowed_signing_keys: [], allowed_host_keys: [], allowed_publishers: []
    )
      super
    end
  end

  TrustPolicy = Struct.new(:default_rule, :rules) do
    def initialize(default_rule: TrustRule.new, rules: [])
      super
    end
  end

  CompromisedKeyEntry = Struct.new(:key, :reason, :reference, :reported_at, keyword_init: true)

  module Trust
    module_function

    def trust_home
      return ENV["PRAY_HOME"] if ENV["PRAY_HOME"]

      home = ENV["HOME"]
      raise Error.manifest("HOME is not set; set PRAY_HOME to configure trust policy") unless home

      File.join(home, ".pray")
    end

    def trust_policy_path(home = trust_home)
      File.join(home, "trust.toml")
    end

    def load_policy(home = trust_home)
      path = trust_policy_path(home)
      return nil unless File.file?(path)

      parse_policy(TomlRB.load_file(path))
    rescue TomlRB::ParseError => error
      raise Error.parse("trust policy", "#{path}: #{error.message}")
    end

    def load_policy_or_default(home = trust_home)
      load_policy(home) || TrustPolicy.new
    end

    def save_policy(policy, home = trust_home)
      path = trust_policy_path(home)
      FileUtils.mkdir_p(File.dirname(path))
      File.write(path, "#{format_policy(policy)}\n")
    end

    def parse_policy(data)
      TrustPolicy.new(
        default_rule: parse_rule(data["default"] || {}),
        rules: Array(data["rules"]).map { |entry| parse_rule(entry) }
      )
    end

    def parse_rule(data)
      TrustRule.new(
        match_prefix: data["match_prefix"],
        allow: data.fetch("allow", true),
        require_signed_commit: data.fetch("require_signed_commit", false),
        allowed_signing_keys: Array(data["allowed_signing_keys"]),
        allowed_host_keys: Array(data["allowed_host_keys"]),
        allowed_publishers: Array(data["allowed_publishers"])
      )
    end

    def best_rule(policy, source_url)
      selected = nil
      selected_length = 0
      policy.rules.each do |rule|
        prefix = rule.match_prefix
        next if prefix.nil? || prefix.empty?
        next unless source_url.start_with?(prefix)
        next unless prefix.length > selected_length

        selected = rule
        selected_length = prefix.length
      end
      selected || policy.default_rule
    end

    def mutable_rule_for_match_prefix!(policy, match_prefix)
      existing = policy.rules.find { |rule| rule.match_prefix == match_prefix }
      return existing if existing

      rule = TrustRule.new(match_prefix: match_prefix)
      policy.rules << rule
      rule
    end

    def normalize_key(value)
      value.strip.upcase
    end

    def fingerprint_matches?(allowed, candidate)
      normalize_key(allowed) == candidate || candidate.end_with?(normalize_key(allowed))
    end

    def prepare_source_host_keys(sources)
      policy = load_policy_or_default
      sources.each_with_object({}) do |source, host_keys|
        next unless source.kind == "pray_ssh"

        rule = best_rule(policy, source.url)
        fingerprint = rule.allowed_host_keys.first
        host_keys[source.name] = fingerprint if fingerprint
      end
    end

    def verify_publisher_fingerprint!(source_url, selected)
      return unless selected.signature
      return unless selected.signer_fingerprint

      policy = load_policy_or_default
      rule = best_rule(policy, source_url)
      return if rule.allowed_publishers.empty?

      normalized = normalize_key(selected.signer_fingerprint)
      trusted = rule.allowed_publishers.any? do |publisher|
        fingerprint_matches?(publisher, normalized)
      end
      return if trusted

      raise Error.integrity(
        "publisher fingerprint #{selected.signer_fingerprint} is not trusted for #{source_url}"
      )
    end

    def format_policy(policy)
      lines = ["[default]"]
      lines.concat(format_rule_lines(policy.default_rule))
      policy.rules.each do |rule|
        lines << ""
        lines << "[[rules]]"
        lines << "match_prefix = #{rule.match_prefix.inspect}" if rule.match_prefix
        lines.concat(format_rule_lines(rule))
      end
      lines.join("\n")
    end

    def format_rule_lines(rule)
      [
        "allow = #{rule.allow}",
        "require_signed_commit = #{rule.require_signed_commit}",
        "allowed_signing_keys = #{toml_array(rule.allowed_signing_keys)}",
        "allowed_host_keys = #{toml_array(rule.allowed_host_keys)}",
        "allowed_publishers = #{toml_array(rule.allowed_publishers)}"
      ]
    end

    def toml_array(values)
      "[#{values.map(&:inspect).join(", ")}]"
    end

    def format_rule_block(scope, rule)
      lines = [
        scope,
        "  allow: #{rule.allow}",
        "  require_signed_commit: #{rule.require_signed_commit}",
        format_keyed_list("allowed_signing_keys", rule.allowed_signing_keys),
        format_keyed_list("allowed_host_keys", rule.allowed_host_keys),
        format_keyed_list("allowed_publishers", rule.allowed_publishers)
      ]
      "#{lines.join("\n")}\n"
    end

    def format_keyed_list(name, values)
      return "  #{name}: []" if values.empty?

      ["  #{name}:", *values.map { |value| "    - #{value}" }].join("\n")
    end

    def append_missing!(target, keys)
      added = 0
      keys.each do |key|
        normalized = normalize_key(key)
        next if normalized.empty?
        next if target.any? { |existing| normalize_key(existing) == normalized }

        target << normalized
        added += 1
      end
      added
    end
  end
end

require_relative "trust_ops"
