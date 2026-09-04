# frozen_string_literal: true

require "fileutils"

module Pray
  module CacheClean
    module_function

    def clean_unused_registry_cache(project_root)
      lockfile = Pray.read_lockfile(File.join(project_root, "Prayfile.lock"))
      registry_root = File.expand_path(".pray/cache/registry", project_root)
      retained = lockfile.package.filter_map do |package|
        package_path = File.expand_path(package.path, project_root)
        package_path if PathSafety.path_under_root?(registry_root, package_path)
      end

      begin
        metadata = File.lstat(registry_root)
      rescue Errno::ENOENT
        return
      end
      if metadata.symlink? || !metadata.directory?
        remove_path(registry_root)
        return
      end
      prune_directory(registry_root, retained, remove_when_empty: false)
    end

    def prune_directory(path, retained, remove_when_empty:)
      Dir.each_child(path) do |name|
        entry = File.join(path, name)
        protects_entry = retained.include?(entry)
        leads_to_retained = retained.any? { |kept| PathSafety.path_under_root?(entry, kept) }
        next if protects_entry

        metadata = File.lstat(entry)
        if leads_to_retained && metadata.directory? && !metadata.symlink?
          prune_directory(entry, retained, remove_when_empty: true)
        else
          remove_path(entry)
        end
      end
      Dir.rmdir(path) if remove_when_empty && Dir.empty?(path)
    end

    def remove_path(path)
      metadata = File.lstat(path)
      if metadata.directory? && !metadata.symlink?
        FileUtils.rm_rf(path)
      else
        File.unlink(path)
      end
    rescue Errno::ENOENT
      nil
    end
  end
end
