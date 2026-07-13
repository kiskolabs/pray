# frozen_string_literal: true

require "open3"
require "fileutils"
require "pathname"

module Pray
  GitSourceCheckout = Struct.new(:cache_directory, :revision, :subdir, keyword_init: true)

  module GitSources
    module_function

    def prepare_git_sources(project_root, sources, lockfile, refresh: false)
      checkouts = {}
      sources.each do |source|
        next unless source.kind == "git"

        clone_url = source.url.delete_prefix("git+")
        if local_filesystem_source?(clone_url) && !local_git_repo_path(clone_url)
          source_root = local_git_source_root(clone_url)
          if source_root
            checkouts[source.name] = GitSourceCheckout.new(
              cache_directory: source_root,
              revision: "",
              subdir: source.subdir
            )
          end
          next
        end

        pinned_revision = refresh ? nil : pinned_revision_for_source(lockfile, source)
        cache_directory, revision = ensure_git_repository(
          project_root,
          clone_url,
          refresh: refresh,
          pinned_revision: pinned_revision,
          sparse_subdir: source.subdir
        )
        checkouts[source.name] = GitSourceCheckout.new(
          cache_directory: cache_directory,
          revision: revision,
          subdir: source.subdir
        )
      end
      checkouts
    end

    def resolve_distribution_root(repo_root, subdir)
      if subdir
        path = File.join(repo_root, subdir)
        return path if local_distribution_root?(path)

        raise Error.resolution(
          "no pray distribution root at subdir #{path.inspect} in git source #{repo_root.inspect}"
        )
      end

      discover_distribution_root(repo_root) ||
        raise(Error.resolution(
                "no pray distribution root in git source #{repo_root.inspect}. " \
                "Expected v1/packages at the repository root or under prayers/. " \
                "Publish with `pray publish --root ./prayers` or point the source at a distribution repository."
              ))
    end

    def discover_distribution_root(path)
      return path if local_distribution_root?(path)

      prayers_root = File.join(path, "prayers")
      return prayers_root if local_distribution_root?(prayers_root)

      nil
    end

    def local_distribution_root?(path)
      File.directory?(File.join(path, "v1", "packages"))
    end

    def local_git_source_root(clone_url)
      path = if clone_url.start_with?("file://")
               clone_url.delete_prefix("file://")
             else
               clone_url
             end
      return nil unless File.exist?(path)

      discover_distribution_root(path)
    end

    def ensure_git_repository(project_root, clone_url, refresh:, pinned_revision:, sparse_subdir:)
      cache_directory = git_source_cache_directory(project_root, clone_url)
      if File.directory?(File.join(cache_directory, ".git"))
        refresh_global_git_cache(clone_url) if refresh
        if pinned_revision
          checkout_git_revision(cache_directory, clone_url, pinned_revision, refresh)
        elsif refresh
          refresh_git_worktree(cache_directory, clone_url)
        end
        apply_sparse_checkout(cache_directory, sparse_subdir) if sparse_subdir
        revision = git_head_revision(cache_directory)
        return [cache_directory, revision]
      end

      FileUtils.rm_rf(cache_directory) if File.exist?(cache_directory)
      FileUtils.mkdir_p(File.dirname(cache_directory))
      if seed_git_cache_from_global(clone_url, cache_directory, project_root)
        run_git_in_repo(cache_directory, "remote", "set-url", "origin", clone_url)
      else
        run_git(project_root, "clone", "--depth", "1", clone_url, cache_directory)
        mirror_git_cache_to_global(clone_url, cache_directory)
      end
      checkout_git_revision(cache_directory, clone_url, pinned_revision, true) if pinned_revision
      apply_sparse_checkout(cache_directory, sparse_subdir) if sparse_subdir
      [cache_directory, git_head_revision(cache_directory)]
    end

    def git_source_cache_directory(project_root, clone_url)
      File.join(project_root, ".pray", "cache", "git", cache_key(clone_url))
    end

    def cache_key(text)
      Hashing.sha256_prefixed(text)[7, 16]
    end

    def pinned_revision_for_source(lockfile, source)
      if lockfile
        entry = lockfile.source.find { |item| item.name == source.name && item.kind == "git" }
        return entry.revision if entry&.revision
      end
      return source.rev if source.kind == "git" && source.rev

      source.tag if source.kind == "git"
    end

    def local_filesystem_source?(clone_url)
      clone_url.start_with?("file://") || Pathname.new(clone_url).absolute?
    end

    def local_git_repo_path(clone_url)
      path = clone_url.delete_prefix("file://")
      git_directory = File.join(path, ".git")
      File.directory?(git_directory) ? path : nil
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
      run_git_in_repo(repository, "sparse-checkout", "init", "--cone")
      run_git_in_repo(repository, "sparse-checkout", "set", subdir)
    end

    def checkout_git_revision(repository, _clone_url, revision, refresh)
      run_git_in_repo(repository, "fetch", "--depth", "1", "origin", revision) if refresh
      run_git_in_repo(repository, "checkout", "--force", revision)
    end

    def refresh_git_worktree(repository, _clone_url)
      run_git_in_repo(repository, "fetch", "--depth", "1", "origin")
      run_git_in_repo(repository, "reset", "--hard", "origin/HEAD")
    end

    def refresh_global_git_cache(clone_url)
      global_cache = global_git_cache_directory(clone_url)
      return unless global_cache && global_git_cache_ready?(global_cache)

      run_git_in_repo(global_cache, "fetch", "--depth", "1", "origin")
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

      raise Error.resolution(command_error("git #{arguments.join(' ')}", output))
    end

    def run_git_in_repo(repository, *arguments)
      run_git(repository, *arguments)
    end

    def command_error(program, output)
      message = output.strip
      message.empty? ? "#{program} failed" : "#{program} failed: #{message}"
    end
  end
end
