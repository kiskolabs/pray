# frozen_string_literal: true

require_relative "error"
require_relative "hashing"
require_relative "git_run"
require_relative "git_store"
require_relative "git_materialize"
require_relative "git_legacy"

module Pray
  module GitCache
    module_function

    def ensure_git_repository(project_root, clone_url, refresh:, pinned_revision:, sparse_subdir:, offline: false)
      db, revision = GitStore.ensure_global_git_db(
        clone_url,
        pinned_revision: pinned_revision,
        refresh: refresh,
        offline: offline,
        working_directory: project_root
      )
      GitLegacy.quarantine_legacy_project_git_checkouts(project_root, clone_url)
      catalog = git_source_cache_directory(project_root, clone_url, sparse_subdir)
      GitMaterialize.materialize_catalog_tree(
        db,
        catalog,
        revision,
        sparse_subdir,
        refresh,
        offline: offline
      )
      [catalog, revision]
    end

    def git_source_cache_directory(project_root, clone_url, subdir = nil)
      identity = (subdir && !subdir.empty?) ? "#{clone_url}\n#{subdir}" : clone_url
      File.join(project_root, ".pray", "cache", "git", cache_key(identity))
    end

    def git_source_cached_repository(project_root, clone_url)
      expected = GitStore.global_git_cache_directory(clone_url)
      return nil unless expected
      return expected if GitStore.global_git_cache_ready?(expected)

      stub_git_dir(git_source_cache_directory(project_root, clone_url), expected) ||
        find_stub_git_dir(project_root, expected)
    end

    def git_checkout?(path)
      File.exist?(File.join(path, ".git"))
    end

    def cache_key(text)
      Hashing.sha256_prefixed(text)[7, 16]
    end

    def stub_git_dir(catalog, expected)
      marker = File.join(catalog, GitMaterialize::GIT_DIR_MARKER)
      return nil unless File.file?(marker)

      git_dir = File.read(marker).strip
      return nil if git_dir.empty? || !File.exist?(git_dir)
      return nil unless paths_equal?(git_dir, expected)

      git_dir
    end

    def find_stub_git_dir(project_root, expected)
      cache_root = File.join(project_root, ".pray", "cache", "git")
      return nil unless File.directory?(cache_root)

      Dir.children(cache_root).each do |name|
        path = File.join(cache_root, name)
        next if git_checkout?(path)

        git_dir = stub_git_dir(path, expected)
        return git_dir if git_dir
      end
      nil
    end

    def paths_equal?(left, right)
      return true if left == right

      File.realpath(left) == File.realpath(right)
    rescue Errno::ENOENT
      false
    end
    private_class_method :stub_git_dir, :find_stub_git_dir, :paths_equal?
  end
end
