# frozen_string_literal: true

module RuboCop
  module Cop
    module Pray
      # Enforces a maximum line count per Ruby source file.
      # RuboCop core does not ship Metrics/FileLength; blank lines and comments count.
      class FileLength < Base
        MSG = "File has too many lines (%<current>d/%<max>d)."

        def on_new_investigation
          super

          max = cop_config["Max"]
          return if max.nil?

          # Match wc -l: count newline characters (shared loc-check script).
          current = processed_source.raw_source.count("\n")
          return if current <= max

          add_global_offense(format(MSG, current: current, max: max))
        end
      end
    end
  end
end
