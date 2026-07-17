# frozen_string_literal: true

require "toml-rb"

module Pray
  TrustRule = Struct.new(
    :match_prefix, :allow, :allowed_host_keys, :allowed_publishers
  ) do
    def initialize(
      match_prefix: nil, allow: true, allowed_host_keys: [], allowed_publishers: []
    )
      super
    end
  end

  TrustPolicy = Struct.new(:default_rule, :rules) do
    def initialize(default_rule: TrustRule.new, rules: [])
      super
    end
  end

  module Trust
    module_function

    def trust_home
      return ENV["PRAY_HOME"] if ENV["PRAY_HOME"]

      home = ENV["HOME"]
      raise Error.manifest("HOME is not set; set PRAY_HOME to configure trust policy") unless home

      File.join(home, ".pray")
    end

    def trust_policy_path
      File.join(trust_home, "trust.toml")
    end

    def load_policy
      path = trust_policy_path
      return nil unless File.file?(path)

      parse_policy(TomlRB.load_file(path))
    rescue TomlRB::ParseError => error
      raise Error.parse("trust policy", "#{path}: #{error.message}")
    end

    def load_policy_or_default
      load_policy || TrustPolicy.new
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

    def normalize_key(value)
      value.strip.upcase
    end

    def fingerprint_matches?(allowed, candidate)
      normalize_key(allowed) == candidate || candidate.end_with?(normalize_key(allowed))
    end

    def format_policy(policy)
      lines = ["[default]", format_rule_lines(policy.default_rule)]
      policy.rules.each do |rule|
        lines << ""
        lines << "[[rules]]"
        lines << "match_prefix = #{rule.match_prefix.inspect}" if rule.match_prefix
        lines.concat(format_rule_lines(rule))
      end
      lines.join("\n")
    end

    def format_rule_lines(rule)
      entries = []
      entries << "allow = #{rule.allow}" unless rule.allow
      entries << "allowed_host_keys = #{rule.allowed_host_keys.inspect}" unless rule.allowed_host_keys.empty?
      entries << "allowed_publishers = #{rule.allowed_publishers.inspect}" unless rule.allowed_publishers.empty?
      entries
    end
  end
end
