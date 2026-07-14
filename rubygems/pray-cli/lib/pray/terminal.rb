# frozen_string_literal: true

module Pray
  module Terminal
    module_function

    def color_enabled?
      return false if env_truthy?("PRAY_NO_COLOR") || no_color_requested?

      term = ENV.fetch("TERM", nil)
      term.nil? || term != "dumb"
    end

    def no_color_requested?
      value = ENV.fetch("NO_COLOR", nil)
      !value.nil? && !value.empty?
    end

    def no_input_requested?
      env_truthy?("PRAY_NO_INPUT")
    end

    def env_truthy?(name)
      value = ENV.fetch(name, nil)
      return false if value.nil?

      %w[1 true yes on].include?(value.strip.downcase)
    end
  end
end
