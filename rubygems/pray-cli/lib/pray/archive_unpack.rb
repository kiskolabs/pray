# frozen_string_literal: true

require "open3"
require "fileutils"
require "find"
require_relative "path_safety"
require_relative "resource_limits"
require_relative "tar_validation"

module Pray
  module ArchiveUnpack
    module_function

    def unpack_praypkg(artifact_bytes, output_directory)
      if artifact_bytes.bytesize > ResourceLimits::MAX_ARCHIVE_TOTAL_BYTES
        raise Error.integrity(
          "package archive exceeds #{ResourceLimits::MAX_ARCHIVE_TOTAL_BYTES} bytes"
        )
      end

      tar_bytes = decompress_zstd(artifact_bytes)
      if tar_bytes.bytesize > ResourceLimits::MAX_ARCHIVE_TOTAL_BYTES
        raise Error.integrity(
          "package archive exceeds #{ResourceLimits::MAX_ARCHIVE_TOTAL_BYTES} decompressed bytes"
        )
      end

      TarValidation.extract!(tar_bytes, output_directory)
      reject_unsafe_extracted_tree!(output_directory)
    end

    def with_binary_process_encoding
      previous = Encoding.default_internal
      verbose = $VERBOSE
      $VERBOSE = nil
      Encoding.default_internal = nil
      yield
    ensure
      Encoding.default_internal = previous
      $VERBOSE = verbose
    end

    def decompress_zstd(artifact_bytes)
      with_binary_process_encoding do
        tar_bytes, status = Open3.capture2("zstd", "-d", "-q", "-c", stdin_data: artifact_bytes)
        unless status.success?
          raise Error.unsupported("zstd is required to unpack package archives")
        end

        tar_bytes
      end
    end

    def reject_unsafe_extracted_tree!(output_directory)
      Find.find(output_directory) do |path|
        next if path == output_directory
        next if File.directory?(path)

        if File.symlink?(path)
          raise Error.integrity("unsupported package archive entry type")
        end
        unless PathSafety.path_under_root?(output_directory, path)
          raise Error.integrity("package path escapes package root: #{path}")
        end
      end
    end
  end
end
