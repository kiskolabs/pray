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

    def quiet?
      env_truthy?("PRAY_QUIET")
    end

    def verbose?
      level = ENV.fetch("PRAY_VERBOSE", "0").to_i
      level > 0
    end

    def configure_output!(quiet:, verbose:)
      ENV["PRAY_QUIET"] = "1" if quiet
      if verbose > 0
        ENV["PRAY_VERBOSE"] = verbose.to_s
      end
    end

    def println_stdout(message)
      return if quiet?

      puts message
    end

    def println_verbose(message)
      return if quiet? || !verbose?

      puts message
    end

    def env_truthy?(name)
      value = ENV.fetch(name, nil)
      return false if value.nil?

      %w[1 true yes on].include?(value.strip.downcase)
    end
  end
end
