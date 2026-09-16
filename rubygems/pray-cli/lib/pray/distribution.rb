# frozen_string_literal: true

require "json"
require "fileutils"

module Pray
  DISTRIBUTION_CONFIG_SPEC = "pray-distribution-config-1"
  TORRENT_PROTOCOL = "torrent"
  DISTRIBUTION_CONFIG_FIELDS = %w[spec protocols bootstrap_trackers enable_dht].freeze

  RegistryDistributionSettings = Struct.new(
    :spec, :protocols, :bootstrap_trackers, :enable_dht,
    keyword_init: true
  ) do
    def initialize(spec: DISTRIBUTION_CONFIG_SPEC, protocols: [], bootstrap_trackers: [], enable_dht: false)
      super
    end

    def allows_torrent?
      Array(protocols).include?(TORRENT_PROTOCOL)
    end
  end

  module Distribution
    module_function

    def default_settings
      RegistryDistributionSettings.new
    end

    def parse_settings(text)
      data = JSON.parse(text)
      unless data.is_a?(Hash)
        raise Error.parse("distribution config", "expected an object")
      end

      data.each_key do |key|
        next if DISTRIBUTION_CONFIG_FIELDS.include?(key)

        raise Error.parse("distribution config", "unsupported field: #{key}")
      end
      settings = RegistryDistributionSettings.new(
        spec: data.fetch("spec", DISTRIBUTION_CONFIG_SPEC),
        protocols: Array(data["protocols"]),
        bootstrap_trackers: Array(data["bootstrap_trackers"]),
        enable_dht: data["enable_dht"] == true
      )
      validate_settings!(settings)
      settings
    rescue JSON::ParserError => error
      raise Error.parse("distribution config", error.message)
    end

    def read_settings(root)
      path = File.join(root, "v1", "distribution.json")
      return default_settings unless File.file?(path)

      parse_settings(File.read(path))
    end

    def write_settings(root, settings = default_settings)
      validate_settings!(settings)
      path = File.join(root, "v1", "distribution.json")
      FileUtils.mkdir_p(File.dirname(path))
      payload = {"spec" => settings.spec, "protocols" => Array(settings.protocols)}
      File.write(path, JSON.pretty_generate(payload))
    end

    def validate_settings!(settings)
      unless settings.spec == DISTRIBUTION_CONFIG_SPEC
        raise Error.parse("distribution config", "unsupported distribution config spec: #{settings.spec}")
      end
      Array(settings.protocols).each do |name|
        next if name == TORRENT_PROTOCOL

        raise Error.parse("distribution config", "unsupported protocol: #{name}")
      end
      if settings.enable_dht
        raise Error.parse("distribution config", "DHT announce is not implemented")
      end
      return if Array(settings.bootstrap_trackers).empty? || settings.allows_torrent?

      raise Error.parse("distribution config", "bootstrap_trackers requires protocols to include torrent")
    end

    def fetch_settings(source_url)
      parse_settings(Registry.http_get(Registry.join_url(source_url, "v1/distribution.json")))
    rescue Error => error
      return default_settings if error.message.match?(/:\s*404\b/)

      raise
    end
  end
end
