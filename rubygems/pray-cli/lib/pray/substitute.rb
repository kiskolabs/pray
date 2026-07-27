# frozen_string_literal: true

module Pray
  module Substitute
    PLACEHOLDER_PREFIX = "((pray:"
    PLACEHOLDER_SUFFIX = "))"

    module_function

    def pray_symbol_key?(key)
      return false if key.nil? || key.empty?

      key.match?(/\A[A-Za-z0-9._\/-]+\z/)
    end

    def substitute_pray_symbols(text, symbols)
      lookup = symbols.is_a?(Hash) ? symbols : symbols.to_h
      output = +""
      rest = text

      loop do
        start = rest.index(PLACEHOLDER_PREFIX)
        unless start
          output << rest
          return output
        end

        output << rest[0...start]
        after_prefix = rest[(start + PLACEHOLDER_PREFIX.length)..]
        end_index = after_prefix.index(PLACEHOLDER_SUFFIX)
        raise Error.render("unclosed ((pray:...) placeholder") unless end_index

        path = after_prefix[0...end_index]
        unless pray_symbol_key?(path)
          raise Error.render("invalid ((pray:...)) path `#{path}`")
        end

        value = lookup[path]
        unless value
          raise Error.render(
            "unknown pray symbol `#{path}`; declare it in `pray do ... end`"
          )
        end

        output << value
        rest = after_prefix[(end_index + PLACEHOLDER_SUFFIX.length)..]
      end
    end
  end
end
