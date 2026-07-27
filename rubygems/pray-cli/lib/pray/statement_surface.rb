# frozen_string_literal: true

module Pray
  module StatementSurface
    module_function

    def expand_statement_surface(statement)
      trimmed = statement.strip
      return [] if trimmed.empty?

      Literal.split_top_level(trimmed, ";").flat_map { |segment| expand_one_surface(segment) }
    end

    def expand_one_surface(statement)
      trimmed = statement.strip
      return [] if trimmed.empty?

      braced = expand_brace_block(trimmed)
      return braced if braced

      [normalize_keyword_call(trimmed)]
    end

    def expand_brace_block(statement)
      keyword = leading_identifier(statement)
      return nil unless keyword

      after_keyword = statement[keyword.length..].lstrip
      # Only keyword{…}, keyword(…){…}, or keyword "…"{…} — not spec.exports = {…}.
      header = split_brace_header(after_keyword)
      return nil unless header

      args, after_open = header
      close_offset = matching_close_brace(after_open)
      return nil unless close_offset
      return nil unless after_open[(close_offset + 1)..].to_s.strip.empty?

      body = after_open[0...close_offset].strip
      return nil unless Literal.is_balanced?(body)

      header_args = unwrap_outer_parens(args)
      open = header_args.empty? ? "#{keyword} do" : "#{keyword} #{header_args} do"
      output = [open]
      output.concat(expand_statement_surface(body)) unless body.empty?
      output << "end"
      output
    end

    def split_brace_header(after_keyword)
      if after_keyword.start_with?("{")
        return ["", after_keyword[1..]]
      end
      if after_keyword.start_with?("(")
        close = matching_close_paren(after_keyword)
        return nil unless close

        trailing = after_keyword[(close + 1)..].to_s.lstrip
        return nil unless trailing.start_with?("{")

        return [after_keyword[1...close].strip, trailing[1..]]
      end
      first = after_keyword[0]
      return nil unless ['"', "'", ":"].include?(first)

      brace_offset = find_top_level_char(after_keyword, "{")
      return nil unless brace_offset

      [after_keyword[0...brace_offset].strip, after_keyword[(brace_offset + 1)..]]
    end

    def normalize_keyword_call(statement)
      spaced = normalize_spaced_block_opener(statement)
      return spaced if spaced

      keyword = leading_identifier(statement)
      return statement unless keyword

      after_keyword = statement[keyword.length..].lstrip
      return statement unless after_keyword.start_with?("(")

      close = matching_close_paren(after_keyword)
      return statement unless close

      inner = after_keyword[1...close].strip
      trailing = after_keyword[(close + 1)..].to_s.strip
      trailing.empty? ? "#{keyword} #{inner}" : "#{keyword} #{inner} #{trailing}"
    end

    def normalize_spaced_block_opener(statement)
      trimmed = statement.strip
      %w[pray template].each do |keyword|
        next unless trimmed.start_with?(keyword)

        rest = trimmed[keyword.length..].lstrip
        return "#{keyword} do" if rest == "do"
      end
      nil
    end

    def split_symbol_assignment(statement)
      trimmed = statement.strip
      call = split_symbol_call(trimmed)
      return call if call

      match = trimmed.match(/\A(\S+)\s+(.+)\z/)
      return nil unless match

      key = match[1].strip
      value = match[2].strip
      return nil if key.empty? || value.empty?

      [key, value]
    end

    def split_symbol_call(statement)
      key = leading_identifier(statement)
      return nil unless key

      after_key = statement[key.length..].lstrip
      return nil unless after_key.start_with?("(") && after_key.end_with?(")")
      return nil unless matching_close_paren(after_key) == after_key.length - 1

      inner = after_key[1...-1].strip
      return nil if inner.empty?

      [key, inner]
    end

    def leading_identifier(input)
      trimmed = input.lstrip
      end_index = 0
      while end_index < trimmed.length
        character = trimmed[end_index]
        break unless character.match?(/[A-Za-z0-9_]/)

        end_index += 1
      end
      return nil if end_index.zero?

      ident = trimmed[0...end_index]
      return nil unless ident[0].match?(/[A-Za-z]/)

      ident
    end

    def unwrap_outer_parens(input)
      trimmed = input.strip
      if trimmed.start_with?("(") && matching_close_paren(trimmed) == trimmed.length - 1
        return trimmed[1...-1].strip
      end

      trimmed
    end

    def matching_close_paren(input)
      matching_close_delimited(input, "(", ")")
    end

    def matching_close_brace(input)
      depth = 1
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
        when "{"
          depth += 1
        when "}"
          depth -= 1
          return index if depth.zero?
        end
      end
      nil
    end

    def matching_close_delimited(input, open, close)
      return nil unless input.start_with?(open)

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

        if character == open
          depth += 1
        elsif character == close
          depth -= 1
          return index if depth.zero?
        end
      end
      nil
    end

    def find_top_level_char(input, needle)
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
        when "(", "[", "{"
          return index if depth.zero? && character == needle

          depth += 1
        when ")", "]", "}"
          depth -= 1
        else
          return index if depth.zero? && character == needle
        end
      end
      nil
    end

    class Reader
      def initialize
        @pending = []
      end

      def push_raw(statement)
        @pending.concat(StatementSurface.expand_statement_surface(statement))
      end

      def next
        @pending.shift
      end
    end
  end
end
