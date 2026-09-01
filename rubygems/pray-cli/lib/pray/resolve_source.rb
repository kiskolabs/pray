# frozen_string_literal: true

module Pray
  module ResolveSource
    module_function

    def implied_source_name(declaration, sources)
      return declaration.source if declaration.source

      namespace = package_namespace(declaration.name)
      return namespace if namespace && sources.key?(namespace)

      case sources.length
      when 0 then nil
      when 1 then sources.keys.first
      else
        raise Error.resolution(
          "package #{declaration.name} requires source: when multiple sources are declared " \
          "and the package namespace does not match a source"
        )
      end
    end

    def package_namespace(name)
      name.include?("/") ? name.split("/", 2).first : nil
    end
  end
end
