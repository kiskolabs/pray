# frozen_string_literal: true

require_relative "error"
require_relative "hashing"
require_relative "git_run"
require_relative "git_store"
require_relative "git_materialize"

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
      catalog = git_source_cache_directory(project_root, clone_url, sparse_subdir)
      GitMaterialize.materialize_catalog_tree(db, catalog, revision, sparse_subdir, refresh)
      [catalog, revision]
    end

    def git_source_cache_directory(project_root, clone_url, subdir = nil)
      identity = (subdir && !subdir.empty?) ? "#{clone_url}\n#{subdir}" : clone_url
      File.join(project_root, ".pray", "cache", "git", cache_key(identity))
    end

    def git_source_cached_repository(project_root, clone_url)
      global_cache = GitStore.global_git_cache_directory(clone_url)
      return global_cache if global_cache && GitStore.global_git_cache_ready?(global_cache)

      shared = git_source_cache_directory(project_root, clone_url)
      return shared if git_checkout?(shared)

      cache_root = File.join(project_root, ".pray", "cache", "git")
      return nil unless File.directory?(cache_root)

      Dir.children(cache_root).each do |name|
        path = File.join(cache_root, name)
        return path if git_checkout?(path) && origin_matches?(path, clone_url)
      end
      nil
    end

    def git_checkout?(path)
      File.exist?(File.join(path, ".git"))
    end

    def cache_key(text)
      Hashing.sha256_prefixed(text)[7, 16]
    end

    def origin_matches?(repository, clone_url)
      output, status = GitRun.capture_git(repository, "remote", "get-url", "origin")
      return false unless status.success?

      origin = output.strip.delete_prefix("git+")
      origin == clone_url
    end
  end
end
