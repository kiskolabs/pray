# frozen_string_literal: true

module Pray
  module ManifestMethods
    module_function

    PACKAGE_KEYWORDS = %w[pray use include agent package].freeze

    def rewrite_constraint_on_line(line, constraint)
      indent = line[/\A\s*/]
      trimmed = line.lstrip
      after_keyword = skip_package_keyword(trimmed)
      raise Error.manifest("package declaration is missing a keyword") unless after_keyword

      after_keyword = after_keyword.lstrip
      parsed_name = parse_quoted(after_keyword)
      raise Error.manifest("package declaration is missing a quoted name") unless parsed_name

      name, after_name = parsed_name
      quoted_constraint = "\"#{constraint}\""
      keyword_and_name = trimmed[0, trimmed.length - after_name.length]
      remainder = after_name.lstrip
      return "#{indent}#{keyword_and_name}, #{quoted_constraint}" if remainder.empty?
      unless remainder.start_with?(",")
        raise Error.manifest("package #{name} declaration is missing a comma after the name")
      end

      after_comma = remainder[1..].lstrip
      if after_comma.start_with?("\"", "'")
        parsed_constraint = parse_quoted(after_comma)
        unless parsed_constraint
          raise Error.manifest("package #{name} declaration has an unclosed constraint")
        end

        return "#{indent}#{keyword_and_name}, #{quoted_constraint}#{parsed_constraint[1]}"
      end

      "#{indent}#{keyword_and_name}, #{quoted_constraint}, #{after_comma}"
    end

    def skip_package_keyword(input)
      PACKAGE_KEYWORDS.each do |keyword|
        next unless input.start_with?(keyword)

        rest = input[keyword.length..]
        next_character = rest[0]
        whitespace_or_quote = next_character.nil? ||
          next_character.match?(/\s/) ||
          next_character == "\"" ||
          next_character == "'"
        return rest if whitespace_or_quote
      end
      nil
    end

    def parse_quoted(input)
      quote = input[0]
      return unless quote == "\"" || quote == "'"

      rest = input[1..]
      ending = rest.index(quote)
      return unless ending

      [rest[0, ending], rest[(ending + 1)..]]
    end
  end
end
