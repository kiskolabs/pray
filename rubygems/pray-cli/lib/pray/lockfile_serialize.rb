# frozen_string_literal: true

module Pray
  module LockfileSerialize
    module_function

    def lockfile_to_toml(lockfile)
      serialize_lockfile_text(lockfile.canonicalized)
    end

    def serialize_lockfile_text(lockfile)
      lines = [
        "prayfile_lock = #{format_string(lockfile.prayfile_lock)}",
        "spec = #{format_string(lockfile.spec)}",
        "generated_by = #{format_string(lockfile.generated_by)}",
        "manifest_hash = #{format_string(lockfile.manifest_hash)}",
        ""
      ]

      lockfile.source.each { |entry| append_section(lines, format_source(entry)) }
      lockfile.package.each { |entry| append_section(lines, format_package(entry)) }
      lockfile.target.each { |entry| append_section(lines, format_target(entry)) }
      lockfile.managed_span.each { |entry| append_section(lines, format_managed_span(entry)) }

      lines.pop while lines.last == ""
      "#{lines.join("\n")}\n"
    end

    def append_section(lines, section_lines)
      return if section_lines.empty?

      lines.concat(section_lines, [""])
    end

    def format_source(entry)
      lines = ["[[source]]"]
      scalars = [
        ["name", entry.name],
        ["kind", entry.kind],
        ["url", entry.url]
      ]
      scalars << ["revision", entry.revision] if entry.revision
      scalars << ["host_key_fingerprint", entry.host_key_fingerprint] if entry.host_key_fingerprint
      append_scalars(lines, scalars)
      lines
    end

    def format_package(entry)
      lines = ["[[package]]"]
      scalars = [
        ["name", entry.name],
        ["version", entry.version]
      ]
      scalars << ["source", entry.source] unless entry.source.nil?
      scalars.concat(
        [
          ["path", entry.path],
          ["tree_hash", entry.tree_hash],
          ["artifact_hash", entry.artifact_hash],
          ["artifact", entry.artifact]
        ]
      )
      append_scalars(lines, scalars)
      lines << "exports = #{format_string_array(entry.exports)}"
      lines << "dependencies = #{format_string_array(entry.dependencies)}"
      lines << "signer_fingerprint = #{format_string(entry.signer_fingerprint)}" if entry.signer_fingerprint
      lines
    end

    def format_target(entry)
      lines = ["[[target]]"]
      append_scalars(lines, [["name", entry.name]])
      lines << "outputs = #{format_string_array(entry.outputs)}"
      lines
    end

    def format_managed_span(entry)
      lines = ["[[managed_span]]"]
      append_scalars(
        lines,
        [
          ["id", entry.id],
          ["target", entry.target],
          ["open_line", entry.open_line],
          ["close_line", entry.close_line],
          ["ideal_checksum", entry.ideal_checksum],
          ["package", entry.package],
          ["export", entry.export],
          ["source_checksum", entry.source_checksum],
          ["silenced", entry.silenced]
        ]
      )
      lines
    end

    def append_scalars(lines, entries)
      entries.each do |key, value|
        lines << case value
                 when String
                   "#{key} = #{format_string(value)}"
                 when Integer
                   "#{key} = #{value}"
                 when TrueClass, FalseClass
                   "#{key} = #{value}"
                 else
                   raise ArgumentError, "unsupported lockfile scalar type for #{key}"
                 end
      end
    end

    def format_string(value)
      escaped = value.gsub("\\", "\\\\").gsub("\"", "\\\"")
      "\"#{escaped}\""
    end

    def format_string_array(values)
      return "[]" if values.empty?
      return "[#{format_string(values.first)}]" if values.length == 1

      items = values.map { |value| "    #{format_string(value)}" }.join(",\n")
      "[\n#{items},\n]"
    end
  end
end
