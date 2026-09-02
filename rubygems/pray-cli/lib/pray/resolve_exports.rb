# frozen_string_literal: true

module Pray
  module ResolveExports
    module_function

    def select_exports(declaration, spec)
      unless declaration.exports.empty?
        declaration.exports.each do |export|
          unless spec.exports.key?(export)
            raise Error.resolution("package #{declaration.name} does not export #{export}")
          end
        end
        return declaration.exports
      end

      roles = declaration.roles || []
      return spec.exports.keys.sort if roles.empty? && declaration.file.nil?

      effective_roles = roles.dup
      effective_roles << "file" if declaration.file && !effective_roles.include?("file")

      selected = []
      effective_roles.each do |role|
        compatible = if role == "fragment"
          fragment_role_exports(spec)
        else
          spec.exports.filter_map do |name, export|
            name if Destination.export_kind_matches_role?(export.kind, role)
          end
        end
        case compatible.length
        when 1
          selected << compatible.first unless selected.include?(compatible.first)
        when 0
          raise Error.resolution(
            "package #{declaration.name} has no export compatible with #{role}"
          )
        else
          raise Error.resolution(
            "package #{declaration.name} has multiple exports compatible with #{role}; set export: \"name\""
          )
        end
      end
      selected
    end

    def load_export_bodies(file_bytes, spec, selected_exports)
      export_bodies = {}
      selected_exports.each do |export_name|
        entry = spec.exports[export_name]
        raise Error.resolution("package #{spec.name} is missing export #{export_name}") unless entry
        next unless %w[fragment file].include?(entry.kind)

        bytes = file_bytes[entry.path]
        unless bytes
          raise Error.integrity("package file missing for export #{export_name}: #{entry.path}")
        end

        text = bytes.dup.force_encoding(Encoding::UTF_8)
        unless text.valid_encoding?
          if entry.kind == "fragment"
            raise Error.integrity("package file is not valid utf-8 for export #{export_name}")
          end
          next
        end
        export_bodies[export_name] = Hashing.normalize_line_endings(text)
      end
      export_bodies
    end

    def fragment_role_exports(spec)
      fragments = spec.exports.filter_map { |name, export| name if export.kind == "fragment" }
      return fragments unless fragments.empty?

      spec.exports.filter_map { |name, export| name if export.kind == "file" }
    end
  end
end
