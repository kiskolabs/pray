# frozen_string_literal: true

module Pray
  module Literal
    LiteralValue = Struct.new(:kind, :value, keyword_init: true) do
      def string? = kind == :string || kind == :symbol
      def as_string = string? ? value : nil
      def as_bool = kind == :bool ? value : nil
      def as_integer = kind == :integer ? value : nil
      def as_array = kind == :array ? value : nil
      def as_map = kind == :map ? value : nil
    end

    module_function

    def split_top_level(input, separator)
      output = []
      start = 0
      depth = 0
      quote = nil
      escaped = false

      input.each_char.with_index do |character, index|
        if quote
          if escaped
            escaped = false
          elsif character == "\\"
            escaped = true
          elsif character == quote
            quote = nil
          end
          next
        end

        case character
        when '"', "'"
          quote = character
        when "[", "{", "("
          depth += 1
        when "]", "}", ")"
          depth -= 1
        else
          if character == separator && depth.zero?
            segment = input[start...index].strip
            output << segment unless segment.empty?
            start = index + 1
          end
        end
      end

      tail = input[start..].strip
      output << tail unless tail.empty?
      output
    end

    def find_top_level(input, token)
      depth = 0
      quote = nil
      escaped = false
      index = 0

      while index < input.length
        character = input[index]
        if quote
          if escaped
            escaped = false
          elsif character == "\\"
            escaped = true
          elsif character == quote
            quote = nil
          end
          index += 1
          next
        end

        case character
        when '"', "'"
          quote = character
        when "[", "{", "("
          depth += 1
        when "]", "}", ")"
          depth -= 1
        else
          return index if depth.zero? && input[index..].start_with?(token)
        end
        index += 1
      end
      nil
    end

    def is_balanced?(input)
      depth = 0
      quote = nil
      escaped = false

      input.each_char do |character|
        if quote
          if escaped
            escaped = false
          elsif character == "\\"
            escaped = true
          elsif character == quote
            quote = nil
          end
          next
        end

        case character
        when '"', "'"
          quote = character
        when "[", "{", "("
          depth += 1
        when "]", "}", ")"
          depth -= 1
        end
      end

      depth.zero? && quote.nil?
    end

    def parse_literal(input)
      parser = Parser.new(input)
      value = parser.parse_value
      parser.skip_whitespace
      unless parser.finished?
        raise Error.parse("literal", "unexpected trailing input near #{parser.remaining.inspect}")
      end
      value
    end

    def parse_literal_map(input)
      value = parse_literal(input)
      raise Error.parse("literal", "expected map literal, found #{value.inspect}") unless value.kind == :map

      value.value
    end

    def parse_literal_array(input)
      value = parse_literal(input)
      raise Error.parse("literal", "expected array literal, found #{value.inspect}") unless value.kind == :array

      value.value
    end

    def prepare_parser_lines(text)
      text.lines.map { |line| prepare_parser_line(line) }
    end

    def prepare_parser_line(line)
      strip_line_comment(line).rstrip
    end

    def strip_line_comment(line)
      quote = nil
      escaped = false
      line.each_char.with_index do |character, index|
        if quote
          if escaped
            escaped = false
          elsif character == "\\"
            escaped = true
          elsif character == quote
            quote = nil
          end
          next
        end

        case character
        when '"', "'"
          quote = character
        when "#"
          return line[0...index]
        end
      end
      line
    end

    class Parser
      def initialize(input)
        @input = input
        @cursor = 0
      end

      def finished? = @cursor >= @input.length
      def remaining = @input[@cursor..]

      def skip_whitespace
        while (character = peek) && character.match?(/\s/)
          @cursor += 1
        end
      end

      def peek = remaining[0]

      def next_character
        character = peek
        @cursor += 1 if character
        character
      end

      def parse_value
        skip_whitespace
        case peek
        when '"', "'"
          parse_string
        when ":"
          parse_symbol
        when "["
          parse_array
        when "{"
          parse_map
        when "-", "0".."9"
          parse_integer_or_identifier
        when nil
          raise Error.parse("literal", "unexpected end of input")
        else
          parse_identifier
        end
      end

      def parse_string
        quote = next_character
        output = +""
        escaped = false
        while (character = next_character)
          if escaped
            output << case character
                      when "n" then "\n"
                      when "r" then "\r"
                      when "t" then "\t"
                      when "\\" then "\\"
                      when "\"", "'" then character
                      else character
                      end
            escaped = false
            next
          end
          if character == "\\"
            escaped = true
            next
          end
          if character == quote
            return LiteralValue.new(kind: :string, value: output)
          end
          output << character
        end
        raise Error.parse("literal", "unterminated string literal")
      end

      def parse_symbol
        next_character
        output = +""
        while (character = peek) && character.match?(%r{[[:alnum:]_\-.\\/]})
          output << character
          next_character
        end
        raise Error.parse("literal", "empty symbol") if output.empty?

        LiteralValue.new(kind: :symbol, value: output)
      end

      def parse_array
        next_character
        values = []
        loop do
          skip_whitespace
          break if peek == "]" && next_character

          values << parse_value
          skip_whitespace
          case peek
          when ","
            next_character
          when "]"
            next_character
            break
          else
            raise Error.parse("literal", "expected ',' or ']'")
          end
        end
        LiteralValue.new(kind: :array, value: values)
      end

      def parse_map
        next_character
        entries = {}
        loop do
          skip_whitespace
          break if peek == "}" && next_character

          key = parse_map_key
          skip_whitespace
          if remaining.start_with?("=>")
            @cursor += 2
          elsif peek == ":"
            next_character
          else
            raise Error.parse("literal", "expected ':' or '=>' after map key")
          end
          entries[key] = parse_value
          skip_whitespace
          case peek
          when ","
            next_character
          when "}"
            next_character
            break
          else
            raise Error.parse("literal", "expected ',' or '}'")
          end
        end
        LiteralValue.new(kind: :map, value: entries)
      end

      def parse_map_key
        skip_whitespace
        case peek
        when '"', "'"
          value = parse_string
          value.value
        when ":"
          parse_symbol.value
        when "a".."z", "A".."Z", "_"
          parse_identifier_name
        else
          raise Error.parse("literal", "invalid map key")
        end
      end

      def parse_integer_or_identifier
        start = @cursor
        next_character if peek == "-"
        while peek&.match?(/[0-9_]/)
          next_character
        end
        if peek == "."
          return parse_identifier_from(start)
        end
        text = @input[start...@cursor].delete("_")
        LiteralValue.new(kind: :integer, value: Integer(text))
      rescue ArgumentError => error
        raise Error.parse("literal", error.message)
      end

      def parse_identifier
        identifier = parse_identifier_name
        case identifier
        when "true" then LiteralValue.new(kind: :bool, value: true)
        when "false" then LiteralValue.new(kind: :bool, value: false)
        when "nil" then LiteralValue.new(kind: :null, value: nil)
        else LiteralValue.new(kind: :string, value: identifier)
        end
      end

      def parse_identifier_name
        raise Error.parse("literal", "expected identifier") unless identifier_start?(peek)

        start = @cursor
        next_character
        while peek && identifier_continue?(peek)
          next_character
        end
        @input[start...@cursor]
      end

      def parse_identifier_from(start)
        while peek && (identifier_continue?(peek) || peek == ".")
          next_character
        end
        LiteralValue.new(kind: :string, value: @input[start...@cursor])
      end

      def identifier_start?(character)
        character && (character.match?(/[[:alpha:]]/) || character == "_")
      end

      def identifier_continue?(character)
        character.match?(%r{[[:alnum:]_\-.\\/]})
      end
    end
  end
end
