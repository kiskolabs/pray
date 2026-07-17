# frozen_string_literal: true

module Pray
  module Dotenv
    module_function

    def load_dotenv_variables(project_root_hint)
      path = File.join(project_root_hint, ".env")
      return {} unless File.file?(path)

      text = File.read(path)
      parse_dotenv_text(text)
    rescue
      {}
    end

    def parse_dotenv_text(text)
      variables = {}
      text.each_line do |line|
        trimmed = line.strip
        next if trimmed.empty? || trimmed.start_with?("#")

        assignment = trimmed.delete_prefix("export ").strip
        key, value = assignment.split("=", 2)
        next unless value

        key = key.strip
        next if key.empty?
        next unless key.start_with?("PRAY_")

        variables[key] = parse_dotenv_value(value.strip)
      end
      variables
    end

    def parse_dotenv_value(value)
      if value.length >= 2
        quote = value[0]
        if (quote == '"' || quote == "'") && value[-1] == quote
          return value[1...-1]
        end
      end
      value
    end
  end
end
