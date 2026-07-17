# frozen_string_literal: true

require "pathname"
require "rubocop"
require_relative "../../pray/require_graph"

module RuboCop
  module Cop
    module Pray
      class CircularRequire < Base
        include RangeHelp

        MSG = "Circular require_relative dependency: %<cycle>s"

        def on_new_investigation
          cycles_for_file(processed_source.file_path).each do |cycle|
            add_offense(
              source_range,
              message: format(MSG, cycle: format_cycle(cycle))
            )
          end
        end

        class << self
          def reset!
            @cycles_by_file = nil
          end

          private

          def cycles_by_file
            @cycles_by_file ||= index_cycles
          end

          def index_cycles
            lib_root = Pathname.new(Dir.pwd).join("lib")
            index = Hash.new { |hash, key| hash[key] = [] }

            RuboCop::Pray::RequireGraph.cycles(lib_root).each do |cycle|
              leader = cycle.first.to_s
              index[leader] << cycle
            end

            index
          end
        end

        private

        def cycles_for_file(file_path)
          normalized_path = RuboCop::Pray::RequireGraph.expand_path(file_path)
          self.class.__send__(:cycles_by_file)[normalized_path]
        end

        def source_range
          range_between(0, 0)
        end

        def format_cycle(cycle)
          root = Pathname.new(Dir.pwd)
          cycle.map { |path| Pathname.new(path).relative_path_from(root).to_s }.join(" -> ")
        end
      end
    end
  end
end
