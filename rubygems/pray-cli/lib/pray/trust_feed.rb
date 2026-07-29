# frozen_string_literal: true

require "net/http"
require "uri"

module Pray
  module Trust
    DEFAULT_COMPROMISED_KEYS_SOURCE =
      "https://raw.githubusercontent.com/bmx-rs/trust-lists/main/compromised-keys.toml"

    module_function

    def check_compromised(source = nil, home: trust_home)
      source_description, body = fetch_compromised_feed(source)
      entries = parse_compromised_feed(body, source_description)
      [source_description, compromised_hits(home, entries)]
    end

    def parse_compromised_feed(body, source_hint)
      if source_hint.downcase.end_with?(".txt")
        parse_compromised_txt(body)
      else
        parse_compromised_toml(body)
      end
    end

    def fetch_compromised_feed(source)
      url = source || DEFAULT_COMPROMISED_KEYS_SOURCE
      if url.start_with?("http://", "https://")
        uri = URI(url)
        response = Net::HTTP.start(
          uri.hostname, uri.port, use_ssl: uri.scheme == "https",
          open_timeout: 10, read_timeout: 30
        ) { |http| http.get(uri.request_uri) }
        unless response.is_a?(Net::HTTPSuccess)
          raise Error.resolution("HTTP request failed for #{url}: #{response.code}")
        end

        return [url, response.body]
      end

      path = File.expand_path(url)
      [path, File.read(path)]
    end

    def parse_compromised_toml(body)
      data = TomlRB.parse(body)
      Array(data["keys"]).filter_map do |entry|
        key = normalize_key(entry["value"].to_s)
        next if key.empty?

        CompromisedKeyEntry.new(
          key: key, reason: entry["reason"],
          reference: entry["reference"], reported_at: entry["reported_at"]
        )
      end
    rescue TomlRB::ParseError
      []
    end

    def parse_compromised_txt(body)
      body.each_line.filter_map do |raw_line|
        line = raw_line.strip
        next if line.empty? || line.start_with?("#")

        head, reason = line.split("#", 2)
        key = normalize_key(head.to_s.split(/\s+/).first.to_s)
        next if key.empty?

        CompromisedKeyEntry.new(
          key: key,
          reason: reason&.strip&.empty? ? nil : reason&.strip,
          reference: nil, reported_at: nil
        )
      end
    end

    def compromised_hits(home, entries)
      trusted = trusted_keys_by_scope(home)
      return [] if trusted.empty?

      by_key = entries.group_by(&:key)
      trusted.filter_map do |key, scopes|
        matches = by_key[key]
        next unless matches

        [key, scopes, matches]
      end
    end

    def trusted_keys_by_scope(home)
      policy = load_policy(home)
      return {} unless policy

      output = Hash.new { |hash, key| hash[key] = [] }
      policy.default_rule.allowed_signing_keys.each do |key|
        normalized = normalize_key(key)
        output[normalized] << "global/default" unless normalized.empty?
      end
      policy.rules.each do |rule|
        scope = "local:#{rule.match_prefix || "-"}"
        rule.allowed_signing_keys.each do |key|
          normalized = normalize_key(key)
          output[normalized] << scope unless normalized.empty?
        end
      end
      output
    end
  end
end
