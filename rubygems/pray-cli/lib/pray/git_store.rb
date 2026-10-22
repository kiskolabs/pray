# frozen_string_literal: true

require "fileutils"
require_relative "error"
require_relative "hashing"
require_relative "git_run"
require_relative "git_clone"

module Pray
  module GitStore
    READY_MARKER = ".pray-ready"

    module_function

    def ensure_global_git_db(clone_url, pinned_revision:, refresh:, offline:, working_directory:)
      db = global_git_cache_directory(clone_url)
      raise Error.resolution("git object cache is not configured") unless db

      with_git_cache_lock(db) do
        unless global_git_cache_ready?(db)
          raise Error.resolution(offline_git_source_uncached(clone_url)) if offline

          install_bare_clone(working_directory, clone_url, db)
        end
        fetch_origin_tip(db, clone_url) if refresh && !offline
        if pinned_revision
          ensure_revision_in_db(db, clone_url, pinned_revision, !offline)
          return [db, pinned_revision]
        end

        [db, git_head_revision(db)]
      end
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
      return false unless File.file?(File.join(global_cache, "HEAD"))
      return false unless File.directory?(File.join(global_cache, "objects"))
      return true if File.file?(File.join(global_cache, READY_MARKER))

      begin
        git_head_revision(global_cache)
        true
      rescue Error
        false
      end
    end

    def mark_git_cache_ready(global_cache)
      File.write(File.join(global_cache, READY_MARKER), "1")
    end

    def offline_git_source_uncached(clone_url)
      "git source #{clone_url} is not cached locally and offline mode is enabled"
    end

    def install_bare_clone(working_directory, clone_url, db)
      FileUtils.mkdir_p(File.dirname(db))
      staging = "#{db}.partial"
      FileUtils.rm_rf(staging) if File.exist?(staging)
      GitClone.clone_bare_git_db(working_directory, clone_url, staging, quiet: false)
      ensure_git_remote_origin(staging, clone_url)
      git_head_revision(staging)
      mark_git_cache_ready(staging)
      if File.exist?(db)
        if global_git_cache_ready?(db)
          FileUtils.rm_rf(staging)
          return
        end
        FileUtils.rm_rf(db)
      end
      FileUtils.mv(staging, db)
    end

    LOCK_ATTEMPTS = 2_400
    LOCK_RETRY_SECONDS = 0.025
    STALE_LOCK_SECONDS = 600

    def with_git_cache_lock(db)
      lock_directory = "#{db}.lock.d"
      FileUtils.mkdir_p(File.dirname(lock_directory))
      acquired = false
      LOCK_ATTEMPTS.times do
        begin
          Dir.mkdir(lock_directory)
          write_lock_owner!(lock_directory)
          acquired = true
          break
        rescue Errno::EEXIST
          clear_stale_lock!(lock_directory)
          sleep LOCK_RETRY_SECONDS
        end
      end
      unless acquired
        raise Error.resolution(
          "timed out waiting for git cache lock #{lock_directory}. " \
          "If no other pray process is using that cache, remove the lock directory and retry."
        )
      end

      begin
        yield
      ensure
        FileUtils.rm_rf(lock_directory)
      end
    end

    def write_lock_owner!(lock_directory)
      File.write(File.join(lock_directory, "owner"), "#{Process.pid}\n#{Time.now.to_i}\n")
    end

    def clear_stale_lock!(lock_directory)
      age = lock_age_seconds(lock_directory)
      return if age < STALE_LOCK_SECONDS

      FileUtils.rm_rf(lock_directory)
    end

    def lock_age_seconds(lock_directory)
      owner = File.join(lock_directory, "owner")
      if File.file?(owner)
        stamp = File.read(owner).lines[1].to_s.strip.to_i
        return Time.now.to_i - stamp if stamp.positive?
      end
      Time.now.to_i - File.mtime(lock_directory).to_i
    rescue Errno::ENOENT
      0
    end
    private_class_method :write_lock_owner!, :clear_stale_lock!, :lock_age_seconds

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
      require_own_git_dir(repository)
      if GitRun.try_run_git(repository, "remote", "get-url", "origin")
        GitRun.run_git(repository, "remote", "set-url", "origin", clone_url)
      else
        GitRun.run_git(repository, "remote", "add", "origin", clone_url)
      end
    end

    # Git searches parent directories when `repository` is not a valid git
    # directory, so a damaged cache inside another checkout would otherwise
    # rewrite that checkout's origin.
    def require_own_git_dir(repository)
      output, status = GitRun.capture_git(repository, "rev-parse", "--absolute-git-dir")
      resolved = status.success? ? output.strip : ""
      own = [repository, File.join(repository, ".git")].any? do |candidate|
        real_path(candidate) && real_path(candidate) == real_path(resolved)
      end
      return if own

      raise Error.resolution(
        "git cache #{repository} is not a usable git repository (git resolved #{resolved.inspect}). " \
        "Remove it and run the command again."
      )
    end

    def real_path(path)
      File.realpath(path)
    rescue SystemCallError
      nil
    end

    def fetch_origin_tip(db, clone_url)
      ensure_git_remote_origin(db, clone_url)
      unless GitRun.try_run_git(db, "fetch", "--depth", "1", "--filter=blob:none", "origin")
        GitRun.run_git(db, "fetch", "--depth", "1", "origin")
      end
      GitRun.run_git(db, "update-ref", "HEAD", "FETCH_HEAD")
      mark_git_cache_ready(db)
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
