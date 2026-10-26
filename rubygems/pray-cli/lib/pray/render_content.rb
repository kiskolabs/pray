# frozen_string_literal: true

module Pray
  module Render
    class ContentBuilder
      attr_reader :next_line_number

      def initialize
        @content = +""
        @next_line_number = 1
      end

      def append_line(line)
        @content << line << "\n"
        @next_line_number += 1
      end

      def append_empty_line
        @content << "\n"
      end

      def append_body(body)
        trimmed = body.sub(/\n+\z/, "")
        return if trimmed.empty?

        trimmed.each_line(chomp: true) { |line| append_line(line) }
      end

      def finish
        @content.sub!(/\n\n+\z/, "\n")
        @content << "\n" unless @content.end_with?("\n")
        @content
      end
    end
  end
end
