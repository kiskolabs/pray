# frozen_string_literal: true

require "fileutils"
require_relative "error"
require_relative "hashing"
require_relative "git_run"
require_relative "git_clone"

module Pray
  module GitStore
    module_function

    def ensure_global_git_db(clone_url, pinned_revision:, refresh:, offline:, working_directory:)
      db = global_git_cache_directory(clone_url)
      raise Error.resolution("git object cache is not configured") unless db

      unless global_git_cache_ready?(db)
        raise Error.resolution(offline_git_source_uncached(clone_url)) if offline

        FileUtils.rm_rf(db) if File.exist?(db)
        FileUtils.mkdir_p(File.dirname(db))
        GitClone.clone_bare_git_db(working_directory, clone_url, db, quiet: false)
        ensure_git_remote_origin(db, clone_url)
      end
      fetch_origin_tip(db, clone_url) if refresh && !offline
      if pinned_revision
        ensure_revision_in_db(db, clone_url, pinned_revision, !offline)
        return [db, pinned_revision]
      end

      [db, git_head_revision(db)]
    end

    def global_cache_root
      return ENV["PRAY_CACHE"] if ENV["PRAY_CACHE"]
      return File.join(ENV["PRAY_HOME"], "cache") if ENV["PRAY_HOME"]

      home = ENV["HOME"]
      home ? File.join(home, ".cache", "pray") : nil
    end

    def global_git_cache_directory(clone_url)
      root = global_cache_root
      root ? File.join(root, "git", Hashing.sha256_prefixed(clone_url)[7, 16]) : nil
    end

    def global_git_cache_ready?(global_cache)
      File.file?(File.join(global_cache, "HEAD")) || File.directory?(File.join(global_cache, ".git"))
    end

    def offline_git_source_uncached(clone_url)
      "git source #{clone_url} is not cached locally and offline mode is enabled"
    end

    def ensure_revision_in_db(db, clone_url, revision, allow_fetch)
      return if GitRun.try_run_git(db, "cat-file", "-e", revision)
      unless allow_fetch
        raise Error.resolution(
          "git source #{db.inspect} is locked to revision #{revision}, but that commit is not available locally and offline mode is enabled"
        )
      end

      ensure_git_remote_origin(db, clone_url)
      fetch_unshallow(db) if File.file?(File.join(db, "shallow"))
      return if GitRun.try_run_git(db, "cat-file", "-e", revision)

      fetch_revision(db, revision)
      return if GitRun.try_run_git(db, "cat-file", "-e", revision)

      raise Error.resolution(
        "git source #{db.inspect} is locked to revision #{revision}, but that commit could not be fetched"
      )
    end

    def ensure_git_remote_origin(repository, clone_url)
      if GitRun.try_run_git(repository, "remote", "get-url", "origin")
        GitRun.run_git(repository, "remote", "set-url", "origin", clone_url)
      else
        GitRun.run_git(repository, "remote", "add", "origin", clone_url)
      end
    end

    def fetch_origin_tip(db, clone_url)
      ensure_git_remote_origin(db, clone_url)
      unless GitRun.try_run_git(db, "fetch", "--depth", "1", "--filter=blob:none", "origin")
        GitRun.run_git(db, "fetch", "--depth", "1", "origin")
      end
      GitRun.run_git(db, "update-ref", "HEAD", "FETCH_HEAD")
    end

    def fetch_unshallow(db)
      return if GitRun.try_run_git(db, "fetch", "--unshallow", "--filter=blob:none", "origin")

      GitRun.run_git(db, "fetch", "--unshallow", "origin")
    end

    def fetch_revision(db, revision)
      return if GitRun.try_run_git(db, "fetch", "--filter=blob:none", "origin", revision)

      GitRun.run_git(db, "fetch", "origin", revision)
    end

    def git_head_revision(repository)
      output, status = GitRun.capture_git(repository, "rev-parse", "HEAD")
      raise Error.resolution(GitRun.command_error("git rev-parse HEAD", output)) unless status.success?

      revision = output.strip
      raise Error.resolution("git repository has no HEAD revision") if revision.empty?

      revision
    end
  end
end
