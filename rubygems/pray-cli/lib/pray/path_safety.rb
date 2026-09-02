# frozen_string_literal: true

require "pathname"

module Pray
  module PathSafety
    module_function

    def path_under_root?(root, candidate)
      root_path = Pathname.new(File.expand_path(root)).cleanpath
      candidate_path = Pathname.new(File.expand_path(candidate)).cleanpath
      candidate_path == root_path || candidate_path.to_s.start_with?("#{root_path}#{File::SEPARATOR}")
    end

    def join_under_root(root, *segments)
      root_path = Pathname.new(File.expand_path(root)).cleanpath
      candidate = root_path.join(*segments).cleanpath
      return candidate.to_s if path_under_root?(root_path.to_s, candidate.to_s)

      nil
    end

    def reject_unsafe_package_name!(package_name)
      if package_name.nil? || package_name.empty? || package_name.include?("\0") || package_name.include?("\\") ||
          package_name.include?("..")
        raise Error.resolution("invalid package name: #{package_name.inspect}")
      end

      package_name
    end

    def sanitize_relative_path(path)
      cleaned = path.to_s.delete_prefix("/").tr("\\", "/")
      if cleaned.empty? || cleaned.include?("\0") || cleaned.split("/").include?("..")
        raise Error.resolution("unsafe relative path: #{path.inspect}")
      end

      cleaned
    end

    def validate_archive_member_path!(path)
      cleaned = path.to_s.delete_prefix("./")
      if cleaned.empty? || Pathname.new(cleaned).absolute? || cleaned.start_with?("/")
        raise Error.integrity("package path must be relative: #{path}")
      end

      cleaned.split("/").each do |part|
        next if part.empty? || part == "."

        if part == ".." || part.include?("\0")
          raise Error.integrity("package path escapes package root: #{path}")
        end
      end

      cleaned
    end

    def validate_project_relative_path!(value)
      path = value.to_s.strip
      windows_absolute = path.match?(/\A(?:[A-Za-z]:[\\\/]|[\\\/]{2})/)
      if path.empty? || Pathname.new(path).absolute? || windows_absolute
        raise Error.manifest("project path must be repository-relative: #{value}")
      end
      parts = path.tr("\\", "/").split("/")
      if parts.include?("..") || path.include?("\0")
        raise Error.manifest("project path escapes repository root: #{value}")
      end
      if parts.all? { |part| part.empty? || part == "." }
        raise Error.manifest("project path must be repository-relative: #{value}")
      end

      path
    end

    def validate_destination_path!(value)
      path = value.to_s.strip
      if path.start_with?("~")
        raise Error.manifest("project path must be repository-relative: #{value}")
      end

      validate_project_relative_path!(value)
    end
  end
end
