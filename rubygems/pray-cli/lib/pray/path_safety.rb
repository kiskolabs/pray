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
  end
end
