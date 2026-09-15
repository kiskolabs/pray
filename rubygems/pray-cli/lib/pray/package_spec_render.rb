# frozen_string_literal: true

module Pray
  module PackageSpecRender
    module_function

    def fork_spec_after_refresh(local, new_upstream, local_prayspec_file, clean_replica, merged_content_paths)
      spec = local.dup
      spec.files = [local_prayspec_file, *merged_content_paths]
      if clean_replica
        spec.exports = new_upstream.exports.dup
        spec.templates = new_upstream.templates.dup
      end
      if spec.upstream
        spec.upstream = PackageUpstream.new(
          name: new_upstream.name,
          constraint: Upstream.next_upstream_constraint(spec.upstream.constraint, new_upstream.version)
        )
      end
      spec
    end

    def render_package_spec(spec)
      lines = ["Package::Specification.new do |spec|"]
      push_identity_fields(lines, spec)
      push_content_fields(lines, spec)
      push_relation_fields(lines, spec)
      lines << "end"
      lines << ""
      lines.join("\n")
    end

    def push_identity_fields(lines, spec)
      push_assignment(lines, "name", spec.name)
      push_assignment(lines, "version", spec.version)
      push_optional(lines, "summary", spec.summary)
      push_optional(lines, "description", spec.description)
      push_array(lines, "authors", spec.authors) unless spec.authors.nil? || spec.authors.empty?
      push_optional(lines, "license", spec.license)
      push_optional(lines, "homepage", spec.homepage)
      push_optional(lines, "source_code_uri", spec.source_code_uri)
      push_optional(lines, "changelog_uri", spec.changelog_uri)
      push_optional(lines, "prayfile_version", spec.prayfile_version)
    end

    def push_content_fields(lines, spec)
      push_array(lines, "files", spec.files)
      push_exports(lines, spec.exports)
      push_named_paths(lines, "skills", spec.skills)
      push_named_paths(lines, "templates", spec.templates)
      push_string_map(lines, "adapters", spec.adapters)
      push_array(lines, "targets", spec.targets) unless spec.targets.nil? || spec.targets.empty?
    end

    def push_relation_fields(lines, spec)
      (spec.dependencies || []).each do |dependency|
        method = dependency.optional ? "add_optional_dependency" : "add_dependency"
        lines << "  spec.#{method} #{quote(dependency.name)}, #{quote(dependency.constraint)}"
      end
      unless spec.metadata.nil? || spec.metadata.empty?
        lines << "  spec.metadata = #{literal_map(spec.metadata)}"
      end
      return unless spec.upstream

      lines << "  spec.upstream #{quote(spec.upstream.name)}, #{quote(spec.upstream.constraint)}"
    end

    def push_assignment(lines, field, value)
      lines << "  spec.#{field} = #{quote(value)}"
    end

    def push_optional(lines, field, value)
      push_assignment(lines, field, value) unless value.nil?
    end

    def push_array(lines, field, values)
      lines << "  spec.#{field} = [#{string_array(values)}]"
    end

    def push_exports(lines, exports)
      return if exports.nil? || exports.empty?

      lines << "  spec.exports = {"
      exports.each do |name, export|
        fields = ["type: #{quote(export.kind)}", "path: #{quote(export.path)}"]
        fields << "summary: #{quote(export.summary)}" if export.summary
        fields << "only: [#{string_array(export.only)}]" unless export.only.nil? || export.only.empty?
        fields << "except: [#{string_array(export.except)}]" unless export.except.nil? || export.except.empty?
        fields << "default_path: #{quote(export.default_path)}" if export.default_path
        lines << "    #{quote(name)} => { #{fields.join(", ")} },"
      end
      lines << "  }"
    end

    def push_named_paths(lines, field, entries)
      return if entries.nil? || entries.empty?

      lines << "  spec.#{field} = {"
      entries.each do |name, entry|
        summary = entry.summary ? ", summary: #{quote(entry.summary)}" : ""
        lines << "    #{quote(name)} => { path: #{quote(entry.path)}#{summary} },"
      end
      lines << "  }"
    end

    def push_string_map(lines, field, entries)
      return if entries.nil? || entries.empty?

      values = entries.map { |name, value| "#{quote(name)} => #{quote(value)}" }.join(", ")
      lines << "  spec.#{field} = { #{values} }"
    end

    def string_array(values)
      Array(values).map { |value| quote(value) }.join(", ")
    end

    def quote(value)
      escaped = value.to_s.gsub("\\", "\\\\").gsub('"', '\\"').gsub("\n", "\\n").gsub("\r", "\\r").gsub("\t", "\\t")
      %("#{escaped}")
    end

    def literal_map(entries)
      values = entries.map { |name, value| "#{quote(name)} => #{literal(value)}" }.join(", ")
      "{ #{values} }"
    end

    def literal(value)
      case value.kind
      when :string then quote(value.value)
      when :symbol then ":#{value.value}"
      when :bool then value.value.to_s
      when :null then "nil"
      when :integer then value.value.to_s
      when :array then "[#{value.value.map { |entry| literal(entry) }.join(", ")}]"
      when :map then literal_map(value.value)
      else quote(value.value)
      end
    end
    private_class_method :push_identity_fields, :push_content_fields, :push_relation_fields,
      :push_assignment, :push_optional, :push_array, :push_exports,
      :push_named_paths, :push_string_map, :string_array, :quote, :literal_map, :literal
  end
end
