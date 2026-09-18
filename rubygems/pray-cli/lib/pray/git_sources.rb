# frozen_string_literal: true

require "pathname"
require_relative "git_cache"

module Pray
  GitSourceCheckout = Struct.new(:cache_directory, :revision, :subdir)

  class GitSourceSet
    def initialize(project_root, sources, lockfile, refresh:, offline: false)
      @project_root = project_root
      @lockfile = lockfile
      @refresh = refresh
      @offline = offline
      @sources = sources.select { |source| source.kind == "git" }.to_h { |source| [source.name, source] }
      @checkouts = {}
    end

    def [](name)
      return @checkouts[name] if @checkouts.key?(name)

      source = @sources[name]
      return nil unless source

      checkout = GitSources.prepare_one_git_source(@project_root, source, @lockfile, refresh: @refresh, offline: @offline)
      @checkouts[name] = checkout if checkout
      checkout
    end

    def revisions
      @checkouts.each_with_object({}) do |(name, checkout), collected|
        next if checkout.revision.to_s.empty?

        collected[name] = checkout.revision
      end
    end
  end

  module GitSources
    module_function

    def prepare_git_sources(project_root, sources, lockfile, refresh: false, offline: false)
      GitSourceSet.new(project_root, sources, lockfile, refresh: refresh, offline: offline)
    end

    def prepare_one_git_source(project_root, source, lockfile, refresh:, offline: false)
      clone_url = source.url.delete_prefix("git+")
      if local_filesystem_source?(clone_url) && !local_git_repo_path(project_root, clone_url)
        source_root = local_git_source_root(project_root, clone_url)
        if source_root
          return GitSourceCheckout.new(
            cache_directory: source_root,
            revision: "",
            subdir: source.subdir
          )
        end
      end

      pinned_revision = refresh ? nil : pinned_revision_for_source(lockfile, source)
      cache_directory, revision = GitCache.ensure_git_repository(
        project_root,
        clone_url,
        refresh: refresh,
        pinned_revision: pinned_revision,
        sparse_subdir: source.subdir,
        offline: offline
      )
      GitSourceCheckout.new(
        cache_directory: cache_directory,
        revision: revision,
        subdir: source.subdir
      )
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

    def local_git_source_root(project_root, clone_url)
      path = clone_url_filesystem_path(project_root, clone_url)
      return nil unless File.exist?(path)

      discover_distribution_root(path)
    end

    def git_source_cache_directory(project_root, clone_url, subdir = nil)
      GitCache.git_source_cache_directory(project_root, clone_url, subdir)
    end

    def git_source_cached_repository(project_root, clone_url)
      GitCache.git_source_cached_repository(project_root, clone_url)
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

    def local_git_repo_path(project_root, clone_url)
      path = clone_url_filesystem_path(project_root, clone_url)
      git_directory = File.join(path, ".git")
      File.directory?(git_directory) ? path : nil
    end

    def clone_url_filesystem_path(project_root, clone_url)
      path = clone_url.delete_prefix("file://")
      Pathname.new(path).absolute? ? path : File.expand_path(path, project_root)
    end
  end
end
