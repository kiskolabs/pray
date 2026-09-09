# frozen_string_literal: true

require "perfect_toml"

module Pray
  module Config
    PrayConfig = Struct.new(:local) do
      def initialize(local: PrayLocalConfig.new)
        super
      end
    end

    PrayLocalConfig = Struct.new(:package, :source) do
      def initialize(package: {}, source: {})
        super
      end
    end

    module_function

    def load_user_config
      path = user_config_path
      return PrayConfig.new unless path && File.file?(path)

      data = PerfectTOML.parse(File.read(path, encoding: "UTF-8"))
      PrayConfig.new(
        local: PrayLocalConfig.new(
          package: data.dig("local", "package") || {},
          source: data.dig("local", "source") || {}
        )
      )
    rescue PerfectTOML::ParseError => error
      raise Error.parse("config", "#{path}: #{error.message}")
    end

    def user_config_path
      if (path = ENV["PRAY_CONFIG"])
        return path
      end

      if (home = ENV["PRAY_HOME"])
        candidate = File.join(home, "config.toml")
        return candidate if File.file?(candidate)
      end

      home = ENV["HOME"]
      return nil unless home

      File.join(home, ".config", "pray", "config.toml")
    end
  end
end
