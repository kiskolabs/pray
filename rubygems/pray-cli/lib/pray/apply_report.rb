# frozen_string_literal: true

module Pray
  module ApplyReport
    module_function

    def local_summary_lines(previous, project)
      previous_checksums = previous_local_checksums(previous)
      project.local_files.filter_map do |local|
        next if local.content.empty? && local.optional

        checksum = local.source_checksum
        previous_checksum = previous_checksums[local.manifest_path]
        if previous_checksum.nil?
          "Installing #{local.manifest_path} (#{checksum})"
        elsif previous_checksum == checksum
          "Using #{local.manifest_path} (#{checksum} checked)"
        else
          "Updating #{local.manifest_path} (#{checksum} was #{previous_checksum})"
        end
      end
    end

    def outdated_local_lines(previous, project)
      previous_checksums = previous_local_checksums(previous)
      project.local_files.filter_map do |local|
        next if local.content.empty? && local.optional

        checksum = local.source_checksum
        previous_checksum = previous_checksums[local.manifest_path]
        if previous_checksum && previous_checksum != checksum
          "#{local.manifest_path} #{previous_checksum} -> #{checksum}"
        elsif previous && previous_checksum.nil?
          "#{local.manifest_path} (new) -> #{checksum}"
        end
      end
    end

    def previous_local_checksums(previous)
      return {} unless previous

      previous.managed_span.each_with_object({}) do |span, checksums|
        next unless span.package == LOCAL_EMBED_PACKAGE

        checksums[span.export] = span.source_checksum
      end
    end
  end
end
