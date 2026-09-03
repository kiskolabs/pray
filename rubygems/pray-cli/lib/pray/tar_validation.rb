# frozen_string_literal: true

require "fileutils"
require_relative "path_safety"
require_relative "resource_limits"

module Pray
  module TarValidation
    BLOCK_BYTES = 512
    State = Struct.new(:paths, :entry_count, :total_bytes, :pending_path, keyword_init: true)

    module_function

    def extract!(bytes, output_directory)
      FileUtils.mkdir_p(output_directory)
      validate!(bytes, output_directory: output_directory)
    end

    def validate!(bytes, output_directory: nil)
      offset = 0
      state = State.new(paths: {}, entry_count: 0, total_bytes: 0)

      while offset + BLOCK_BYTES <= bytes.bytesize
        header = bytes.byteslice(offset, BLOCK_BYTES)
        return if header.bytes.all?(&:zero?)

        verify_checksum!(header)
        size = parse_octal(header.byteslice(124, 12), "entry size")
        data_start = offset + BLOCK_BYTES
        data_end = data_start + size
        raise Error.integrity("truncated package archive entry") if data_end > bytes.bytesize

        process_entry(header, bytes.byteslice(data_start, size), size, state, output_directory)
        offset = data_start + ((size + BLOCK_BYTES - 1) / BLOCK_BYTES * BLOCK_BYTES)
      end
      raise Error.integrity("package archive is missing its end marker")
    end

    def process_entry(header, data, size, state, output_directory)
      type = header.byteslice(156, 1)
      if type == "x"
        state.pending_path = pax_path(data)
        return
      end
      if type == "L"
        state.pending_path = c_string(data).delete_suffix("\n")
        return
      end

      raw_path = state.pending_path || header_path(header)
      state.pending_path = nil
      return if type == "5" && [".", "./"].include?(raw_path)

      path = PathSafety.validate_archive_member_path!(raw_path)
      return if File.basename(path).start_with?("._")
      if type == "5"
        write_extracted_directory(output_directory, path) if output_directory
        return
      end
      validate_regular_entry!(type, path, size, state)
      write_extracted_file(output_directory, path, data) if output_directory
    end

    def validate_regular_entry!(type, path, size, state)
      raise Error.integrity("unsupported package archive entry type") unless ["0", "\0"].include?(type)

      state.entry_count += 1
      if state.entry_count > ResourceLimits::MAX_ARCHIVE_ENTRIES
        raise Error.integrity("package archive exceeds #{ResourceLimits::MAX_ARCHIVE_ENTRIES} entries")
      end
      raise Error.integrity("duplicate package archive path: #{path}") if state.paths.key?(path)

      state.paths[path] = true
      if size > ResourceLimits::MAX_ARCHIVE_ENTRY_BYTES
        raise Error.integrity("package archive entry exceeds #{ResourceLimits::MAX_ARCHIVE_ENTRY_BYTES} bytes: #{path}")
      end
      state.total_bytes += size
      if state.total_bytes > ResourceLimits::MAX_ARCHIVE_TOTAL_BYTES
        raise Error.integrity("package archive exceeds #{ResourceLimits::MAX_ARCHIVE_TOTAL_BYTES} decompressed bytes")
      end
    end

    def write_extracted_directory(output_directory, path)
      FileUtils.mkdir_p(extracted_destination(output_directory, path))
    end

    def write_extracted_file(output_directory, path, data)
      destination = extracted_destination(output_directory, path)
      FileUtils.mkdir_p(File.dirname(destination))
      File.open(destination, File::WRONLY | File::CREAT | File::EXCL | File::BINARY) do |file|
        file.write(data)
      end
    end

    def extracted_destination(output_directory, path)
      destination = File.join(output_directory, path)
      unless PathSafety.path_under_root?(output_directory, destination)
        raise Error.integrity("package path escapes package root: #{path}")
      end

      destination
    end

    def verify_checksum!(header)
      # The checksum field is eight bytes. Writers fill it differently: six octal
      # digits then NUL and space, seven digits then NUL, or space padding. Read the
      # whole field and let parse_octal strip the terminator.
      stored = parse_octal(header.byteslice(148, 8), "checksum")
      sum = 0
      header.bytes.each_with_index do |byte, index|
        sum += (index >= 148 && index < 156) ? 32 : byte
      end
      return if sum == stored

      raise Error.integrity("invalid package archive checksum")
    end

    def header_path(header)
      name = c_string(header.byteslice(0, 100))
      prefix = c_string(header.byteslice(345, 155))
      prefix.empty? ? name : "#{prefix}/#{name}"
    end

    def c_string(bytes)
      bytes.to_s.split("\0", 2).first.to_s.force_encoding(Encoding::UTF_8).tap do |text|
        raise Error.integrity("invalid package archive path encoding") unless text.valid_encoding?
      end
    end

    def parse_octal(bytes, label)
      value = c_string(bytes).strip
      raise Error.integrity("invalid package archive #{label}") unless value.match?(/\A[0-7]+\z/)

      value.to_i(8)
    end

    def pax_path(data)
      cursor = 0
      path = nil
      while cursor < data.bytesize
        separator = data.index(" ".b, cursor)
        length = separator && Integer(data.byteslice(cursor, separator - cursor), exception: false)
        record_end = cursor + length.to_i
        if !length || length <= 0 || record_end > data.bytesize
          raise Error.integrity("invalid package archive pax header")
        end
        record = data.byteslice(separator + 1, record_end - separator - 2)
        equals = record.index("=".b)
        if equals && record.byteslice(0, equals) == "path"
          value = record.byteslice(equals + 1, record.bytesize - equals - 1).force_encoding(Encoding::UTF_8)
          raise Error.integrity("invalid package archive path encoding") unless value.valid_encoding?

          path = value
        end
        cursor = record_end
      end
      path
    end
  end
end
