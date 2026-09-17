# frozen_string_literal: true

require "open3"
require "fileutils"
require "pathname"
require_relative "error"
require_relative "hashing"

module Pray
  module GitCache
    module_function

    def ensure_git_repository(project_root, clone_url, refresh:, pinned_revision:, sparse_subdir:)
      shared = git_source_cache_directory(project_root, clone_url)
      ensure_shared_git_repository(project_root, clone_url, shared, refresh: refresh, pinned_revision: pinned_revision)
      checkout = git_source_cache_directory(project_root, clone_url, sparse_subdir)
      if checkout != shared
        ensure_linked_worktree(shared, checkout)
        if pinned_revision
          checkout_git_revision(checkout, clone_url, pinned_revision, refresh)
        elsif refresh
          run_git(checkout, "reset", "--hard", git_head_revision(shared))
        end
        apply_sparse_checkout(checkout, sparse_subdir) if sparse_subdir
        return [checkout, git_head_revision(checkout)]
      end

      [shared, git_head_revision(shared)]
    end

    def git_source_cache_directory(project_root, clone_url, subdir = nil)
      identity = (subdir && !subdir.empty?) ? "#{clone_url}\n#{subdir}" : clone_url
      File.join(project_root, ".pray", "cache", "git", cache_key(identity))
    end

    def git_source_cached_repository(project_root, clone_url)
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

    def ensure_shared_git_repository(project_root, clone_url, shared, refresh:, pinned_revision:)
      if File.directory?(File.join(shared, ".git"))
        if pinned_revision
          checkout_git_revision(shared, clone_url, pinned_revision, refresh)
        elsif refresh
          refresh_git_worktree(shared, clone_url)
        end
        refresh_global_from_project(clone_url, shared) if refresh
        return
      end

      FileUtils.rm_rf(shared) if File.exist?(shared)
      FileUtils.mkdir_p(File.dirname(shared))
      seeded = seed_git_cache_from_global(clone_url, shared, project_root)
      if seeded
        run_git(shared, "remote", "set-url", "origin", clone_url)
      else
        run_git(project_root, "clone", "--depth", "1", clone_url, shared)
        mirror_git_cache_to_global(clone_url, shared)
      end
      if pinned_revision
        checkout_git_revision(shared, clone_url, pinned_revision, true)
      elsif refresh && seeded
        refresh_git_worktree(shared, clone_url)
      end
      refresh_global_from_project(clone_url, shared) if refresh && seeded
    end

    def ensure_linked_worktree(shared, checkout)
      return if same_object_store?(shared, checkout)

      FileUtils.rm_rf(checkout) if File.exist?(checkout)
      FileUtils.mkdir_p(File.dirname(checkout))
      run_git(shared, "worktree", "add", "--detach", checkout)
    end

    def same_object_store?(shared, checkout)
      return false unless git_checkout?(checkout)

      git_common_dir(shared) == git_common_dir(checkout)
    end

    def git_common_dir(repository)
      output, status = Open3.capture2e("git", "-C", repository, "rev-parse", "--git-common-dir")
      raise Error.resolution(command_error("git rev-parse --git-common-dir", output)) unless status.success?

      reported = output.strip
      path = Pathname.new(reported).absolute? ? reported : File.expand_path(reported, repository)
      File.realpath(path)
    end

    def origin_matches?(repository, clone_url)
      output, status = Open3.capture2e("git", "-C", repository, "remote", "get-url", "origin")
      return false unless status.success?

      origin = output.strip.delete_prefix("git+")
      origin == clone_url
    end

    def global_cache_root
      return ENV["PRAY_CACHE"] if ENV["PRAY_CACHE"]
      return File.join(ENV["PRAY_HOME"], "cache") if ENV["PRAY_HOME"]

      home = ENV["HOME"]
      home ? File.join(home, ".cache", "pray") : nil
    end

    def global_git_cache_directory(clone_url)
      root = global_cache_root
      root ? File.join(root, "git", cache_key(clone_url)) : nil
    end

    def global_git_cache_ready?(global_cache)
      File.directory?(File.join(global_cache, ".git")) || File.file?(File.join(global_cache, "HEAD"))
    end

    def seed_git_cache_from_global(clone_url, destination, working_directory)
      global_cache = global_git_cache_directory(clone_url)
      return false unless global_cache && global_git_cache_ready?(global_cache)

      run_git(working_directory, "clone", "--depth", "1", "--quiet", global_cache, destination)
      true
    end

    def mirror_git_cache_to_global(clone_url, project_cache)
      global_cache = global_git_cache_directory(clone_url)
      return unless global_cache
      return if global_git_cache_ready?(global_cache)

      FileUtils.mkdir_p(File.dirname(global_cache))
      FileUtils.rm_rf(global_cache) if File.exist?(global_cache)
      run_git(File.dirname(project_cache), "clone", "--bare", "--quiet", File.basename(project_cache), global_cache)
    end

    def apply_sparse_checkout(repository, subdir)
      run_git(repository, "sparse-checkout", "init", "--cone")
      run_git(repository, "sparse-checkout", "set", subdir)
    end

    def checkout_git_revision(repository, _clone_url, revision, refresh)
      run_git(repository, "fetch", "--depth", "1", "origin", revision) if refresh
      run_git(repository, "checkout", "--force", revision)
    end

    def refresh_git_worktree(repository, _clone_url)
      run_git(repository, "fetch", "--depth", "1", "origin")
      run_git(repository, "reset", "--hard", "origin/HEAD")
    end

    def refresh_global_from_project(clone_url, project_cache)
      global_cache = global_git_cache_directory(clone_url)
      return unless global_cache

      FileUtils.rm_rf(global_cache) if global_git_cache_ready?(global_cache) || File.exist?(global_cache)
      mirror_git_cache_to_global(clone_url, project_cache)
    end

    def git_head_revision(repository)
      output, status = Open3.capture2e("git", "-C", repository, "rev-parse", "HEAD")
      raise Error.resolution(command_error("git rev-parse HEAD", output)) unless status.success?

      revision = output.strip
      raise Error.resolution("git repository has no HEAD revision") if revision.empty?

      revision
    end

    def run_git(cwd, *arguments)
      output, status = Open3.capture2e("git", "-C", cwd, *arguments)
      return if status.success?

      raise Error.resolution(command_error("git #{arguments.join(" ")}", output))
    end

    def command_error(program, output)
      message = output.strip
      message.empty? ? "#{program} failed" : "#{program} failed: #{message}"
    end
  end
end
