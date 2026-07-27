# frozen_string_literal: true

module Pray
  module ManifestJson
    module_function

    def encode_compact(manifest)
      encode_object(manifest_fields(manifest))
    end

    def manifest_fields(manifest)
      fields = {
        "prayfile_version" => manifest.prayfile_version,
        "sources" => manifest.sources.map { |source| source_fields(source) },
        "targets" => manifest.targets.map { |target| target_fields(target) },
        "packages" => manifest.packages.map { |package| package_fields(package) },
        "local" => manifest.local.map { |entry| local_fields(entry) },
        "render" => render_fields(manifest.render)
      }
      fields["symbols"] = manifest.symbols.sort.to_h unless manifest.symbols.empty?
      fields
    end

    def source_fields(source)
      fields = {
        "name" => source.name,
        "kind" => source.kind,
        "url" => source.url
      }
      fields["subdir"] = source.subdir if source.subdir
      fields["rev"] = source.rev if source.rev
      fields["tag"] = source.tag if source.tag
      fields
    end

    def target_fields(target)
      {
        "name" => target.name,
        "outputs" => target.outputs,
        "skills" => target.skills,
        "commands" => target.commands,
        "rules" => target.rules,
        "max_bytes" => target.max_bytes,
        "mode" => target.mode || "legacy",
        "scoped" => target.scoped || false,
        "entries" => (target.entries || []).map { |entry| entry_fields(entry) }
      }
    end

    def entry_fields(entry)
      if entry.kind == "package"
        { "kind" => "package", "name" => entry.name }
      else
        { "kind" => "local", "path" => entry.path }
      end
    end

    def package_fields(package)
      fields = {
        "name" => package.name,
        "constraint" => package.constraint,
        "source" => package.source,
        "exports" => package.exports,
        "targets" => package.targets,
        "features" => package.features,
        "groups" => package.groups,
        "optional" => package.optional,
        "path" => package.path,
        "git" => package.git,
        "tag" => package.tag,
        "rev" => package.rev,
        "tarball" => package.tarball,
        "oci" => package.oci
      }
      fields["file"] = package.file if package.file
      fields["roles"] = package.roles unless package.roles.nil? || package.roles.empty?
      fields["bound"] = true if package.bound
      fields
    end

    def local_fields(entry)
      fields = {
        "path" => entry.path,
        "position" => entry.position,
        "optional" => entry.optional
      }
      fields["bound"] = true if entry.bound
      fields
    end

    def render_fields(render)
      {
        "mode" => render.mode,
        "conflict" => render.conflict,
        "churn" => render.churn,
        "header" => render.header,
        "section_markers" => render.section_markers,
        "line_endings" => render.line_endings
      }
    end

    def encode_object(hash)
      entries = hash.map { |key, value| "#{encode_string(key)}:#{encode_value(value)}" }
      "{#{entries.join(",")}}"
    end

    def encode_array(array)
      "[#{array.map { |value| encode_value(value) }.join(",")}]"
    end

    def encode_value(value)
      case value
      when Hash then encode_object(value)
      when Array then encode_array(value)
      when String then encode_string(value)
      when TrueClass, FalseClass then value.to_s
      when NilClass then "null"
      when Integer then value.to_s
      else
        raise Error.manifest("unsupported JSON value: #{value.inspect}")
      end
    end

    def encode_string(text)
      "\"#{text.gsub("\\", "\\\\").gsub("\"", "\\\"").gsub("\n", "\\n").gsub("\r", "\\r").gsub("\t", "\\t")}\""
    end
  end
end
