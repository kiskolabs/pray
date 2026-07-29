# frozen_string_literal: true

module Pray
  module ManifestMethods
    module_function

    def format_package_declaration(package)
      parts = ["pray \"#{package.name}\""]
      parts << "\"#{package.constraint}\"" unless package.constraint == "*"
      parts << "path: \"#{package.path}\"" if package.path
      parts << "source: \"#{package.source}\"" if package.source
      parts << "git: \"#{package.git}\"" if package.git
      parts << "tag: \"#{package.tag}\"" if package.tag
      parts << "rev: \"#{package.rev}\"" if package.rev
      parts << "tarball: \"#{package.tarball}\"" if package.tarball
      parts << "oci: \"#{package.oci}\"" if package.oci
      parts << "file: \"#{package.file}\"" if package.file
      unless package.exports.empty?
        parts << if package.exports.length == 1
          "export: \"#{package.exports.first}\""
        else
          "exports: [#{format_string_keyword_list(package.exports)}]"
        end
      end
      parts << "targets: [#{format_string_keyword_list(package.targets)}]" unless package.targets.empty?
      parts << "features: [#{format_string_keyword_list(package.features)}]" unless package.features.empty?
      parts << "optional: true" if package.optional
      parts.join(", ")
    end

    def replace_package_declaration(text, package)
      name = package.name
      prefixes = [
        "pray \"#{name}\"", "pray '#{name}'",
        "use \"#{name}\"", "include \"#{name}\"",
        "agent \"#{name}\"", "agent '#{name}'",
        "package \"#{name}\"", "package '#{name}'"
      ]
      lines = text.lines.map(&:chomp)
      index = lines.index { |line|
        trimmed = line.lstrip
        prefixes.any? { |prefix| trimmed.start_with?(prefix) }
      }
      raise Error.manifest("package #{name} not found in manifest") unless index

      lines[index] = format_package_declaration(package)
      output = lines.join("\n")
      output += "\n" if text.end_with?("\n") && !output.end_with?("\n")
      output
    end

    def format_string_keyword_list(values)
      values.map { |value| "\"#{value}\"" }.join(", ")
    end
  end
end
