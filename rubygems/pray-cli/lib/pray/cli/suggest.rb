# frozen_string_literal: true

module Pray
  module CLI
    module Suggest
      TOP_LEVEL_COMMANDS = %w[
        add apply clean confess drift explain format help init install list login manifest
        outdated package plan prayer publish remove render repo serve sync tree trust unlock
        update vendor verify version
      ].freeze

      module_function

      def unknown_command_message(command)
        message = "unknown command: #{command}"
        suggestion = suggest_command(command, TOP_LEVEL_COMMANDS)
        suggestion ? "#{message}\nDid you mean `#{suggestion}`?" : message
      end

      def suggest_command(input, candidates)
        maximum_distance = input.length <= 3 ? 1 : 2
        candidates
          .map { |candidate| [candidate, levenshtein_distance(input, candidate)] }
          .select { |(_, distance)| distance <= maximum_distance }
          .min_by { |(_, distance)| distance }
          &.first
      end

      def levenshtein_distance(left, right)
        left_chars = left.chars
        right_chars = right.chars
        left_length = left_chars.length
        right_length = right_chars.length
        return right_length if left_length.zero?
        return left_length if right_length.zero?

        previous_row = (0..right_length).to_a
        current_row = Array.new(right_length + 1, 0)

        left_chars.each_with_index do |left_character, left_index|
          current_row[0] = left_index + 1
          right_chars.each_with_index do |right_character, right_index|
            substitution_cost = left_character == right_character ? 0 : 1
            current_row[right_index + 1] = [
              previous_row[right_index + 1] + 1,
              current_row[right_index] + 1,
              previous_row[right_index] + substitution_cost
            ].min
          end
          previous_row, current_row = current_row, previous_row
        end

        previous_row[right_length]
      end
    end
  end
end
