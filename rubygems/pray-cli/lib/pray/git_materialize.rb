# frozen_string_literal: true

require "fileutils"
require "open3"
require_relative "error"
require_relative "git_clone"
require_relative "git_run"
require_relative "git_store"
require_relative "path_safety"
require_relative "resource_limits"
require_relative "tar_validation"

module Pray
  module GitMaterialize
    REVISION_MARKER = ".pray-revision"
    GIT_DIR_MARKER = ".pray-git-dir"

    module_function

    def read_local_artifact_bytes(source_root, artifact, offline: false)
      if artifact.include?("://") || artifact.start_with?("file:")
        raise Error.integrity(
          "local and git catalog artifacts must be relative paths under the distribution root, not URLs: #{artifact}"
        )
      end

      path = PathSafety.join_under_root(source_root, artifact)
      raise Error.resolution("package artifact path escapes distribution root") unless path

      materialize_catalog_file(source_root, artifact, offline: offline) unless File.exist?(path)
      raise Error.resolution("package artifact missing at #{path}") unless File.exist?(path)

      File.binread(path)
    end

    def materialize_catalog_tree(git_dir, dest, revision, subdir, refresh, offline: false)
      require_revision_present!(git_dir, revision, offline: offline)
      GitStore.with_git_cache_lock(dest) do
        return if !refresh && catalog_matches?(dest, git_dir, revision)

        staging = "#{dest}.partial"
        FileUtils.rm_rf(staging)
        FileUtils.mkdir_p(staging)
        unpacked = GitClone.catalog_sparse_cones(subdir).any? do |prefix|
          unpack_archive_prefix(git_dir, staging, revision, prefix, offline: offline)
        end
        unless unpacked
          FileUtils.rm_rf(staging)
          if offline
            raise Error.resolution(
              "git catalog at revision #{revision} is not materializable locally and offline mode is enabled"
            )
          end
          raise Error.resolution("no pray distribution root in git source at revision #{revision}")
        end

        File.write(File.join(staging, REVISION_MARKER), revision)
        File.write(File.join(staging, GIT_DIR_MARKER), git_dir)
        publish_catalog_tree!(dest, staging)
      end
    end

    def publish_catalog_tree!(dest, staging)
      FileUtils.mkdir_p(File.dirname(dest))
      if File.exist?(dest)
        backup = "#{dest}.previous"
        FileUtils.rm_rf(backup)
        File.rename(dest, backup)
        begin
          File.rename(staging, dest)
        rescue StandardError
          File.rename(backup, dest) if File.exist?(backup)
          FileUtils.rm_rf(staging)
          raise
        end
        FileUtils.rm_rf(backup)
      else
        File.rename(staging, dest)
      end
    end
    private_class_method :publish_catalog_tree!

    def materialize_catalog_file(source_root, relative, offline: false)
      full_path = File.join(source_root, relative)
      return if File.file?(full_path)

      markers = find_catalog_markers(source_root)
      return unless markers

      GitStore.with_git_cache_lock(markers.fetch(:catalog_root)) do
        return if File.file?(full_path)

        require_revision_present!(markers.fetch(:git_dir), markers.fetch(:revision), offline: offline)
        repo_relative = full_path.delete_prefix(markers.fetch(:catalog_root)).delete_prefix("/")
        object = "#{markers.fetch(:revision)}:#{repo_relative}"
        require_blob_within_limit!(markers.fetch(:git_dir), object, offline: offline)
        env = GitRun.git_env(offline_git_env(offline))
        stdout, status = Open3.capture2(
          env,
          "git",
          "-c",
          "protocol.file.allow=always",
          "-C",
          markers.fetch(:git_dir),
          "show",
          object,
          stdin_data: ""
        )
        unless status.success?
          if offline
            raise Error.resolution(
              "git catalog blob #{repo_relative} at revision #{markers.fetch(:revision)} is not available locally and offline mode is enabled"
            )
          end
          annotate_git_materialize_failure!(
            markers.fetch(:git_dir),
            markers.fetch(:revision),
            "git show failed"
          )
        end
        write_catalog_blob_atomic!(full_path, stdout)
      end
    end

    def require_revision_present!(git_dir, revision, offline: false)
      return if GitRun.try_run_git(git_dir, "cat-file", "-e", revision)

      if offline
        raise Error.resolution(
          "git revision #{revision} is not available in #{git_dir} and offline mode is enabled (git_revision_unfetched)."
        )
      end
      raise Error.resolution(
        "git revision #{revision} is not available in #{git_dir} (git_revision_unfetched). " \
        "Run `pray install --locked`, or clear the project git cache and shared bare then install again."
      )
    end

    def annotate_git_materialize_failure!(git_dir, revision, detail)
      raise Error.resolution(
        "git catalog materialize failed for revision #{revision} in #{git_dir} (git_cache_stale): #{detail}. " \
        "Run `pray install --locked`, or clear `.pray/cache/git` and the shared bare then install again."
      )
    end

    def catalog_matches?(dest, git_dir, revision)
      revision_path = File.join(dest, REVISION_MARKER)
      git_dir_path = File.join(dest, GIT_DIR_MARKER)
      return false unless File.file?(revision_path) && File.file?(git_dir_path)
      return false unless File.read(revision_path).strip == revision
      return false unless File.read(git_dir_path).strip == git_dir
      return false unless GitRun.try_run_git(git_dir, "cat-file", "-e", revision)

      File.directory?(File.join(dest, "v1", "packages")) ||
        File.directory?(File.join(dest, "prayers", "v1", "packages")) ||
        Dir.children(dest).any? { |name| File.directory?(File.join(dest, name, "v1", "packages")) }
    rescue Errno::ENOENT
      false
    end

    def unpack_archive_prefix(git_dir, dest, revision, prefix, offline: false)
      return false unless tree_prefix_within_limits?(git_dir, revision, prefix, offline: offline)

      env = GitRun.git_env(offline_git_env(offline).merge("GIT_TERMINAL_PROMPT" => "0"))
      stdout, _stderr, status = Open3.capture3(
        env,
        "git",
        "-c",
        "protocol.file.allow=always",
        "-C",
        git_dir,
        "archive",
        "--format=tar",
        revision,
        "--",
        prefix,
        stdin_data: ""
      )
      return false unless status.success? && !stdout.empty?
      if stdout.bytesize > ResourceLimits::MAX_ARCHIVE_TOTAL_BYTES
        raise Error.integrity(
          "git catalog archive exceeds #{ResourceLimits::MAX_ARCHIVE_TOTAL_BYTES} bytes"
        )
      end

      TarValidation.extract!(stdout, dest)
      true
    end

    def require_blob_within_limit!(git_dir, object, offline: false)
      env = GitRun.git_env(offline_git_env(offline))
      stdout, status = Open3.capture2(
        env, "git", "-c", "protocol.file.allow=always", "-C", git_dir, "cat-file", "-s", object,
        stdin_data: ""
      )
      return unless status.success?

      size = stdout.strip.to_i
      return if size <= ResourceLimits::MAX_ARCHIVE_ENTRY_BYTES

      raise Error.integrity(
        "git catalog blob exceeds #{ResourceLimits::MAX_ARCHIVE_ENTRY_BYTES} bytes"
      )
    end

    def tree_prefix_within_limits?(git_dir, revision, prefix, offline: false)
      env = GitRun.git_env(offline_git_env(offline))
      stdout, status = Open3.capture2(
        env, "git", "-c", "protocol.file.allow=always", "-C", git_dir,
        "ls-tree", "-r", "-l", revision, "--", prefix, stdin_data: ""
      )
      return false unless status.success? && !stdout.strip.empty?

      entry_count = 0
      total_bytes = 0
      stdout.each_line do |line|
        parts = line.split
        next unless parts[1] == "blob"

        size = parts[3].to_i
        entry_count += 1
        if entry_count > ResourceLimits::MAX_ARCHIVE_ENTRIES
          raise Error.integrity(
            "git catalog archive exceeds #{ResourceLimits::MAX_ARCHIVE_ENTRIES} entries"
          )
        end
        if size > ResourceLimits::MAX_ARCHIVE_ENTRY_BYTES
          raise Error.integrity(
            "git catalog archive entry exceeds #{ResourceLimits::MAX_ARCHIVE_ENTRY_BYTES} bytes"
          )
        end
        total_bytes += size
        if total_bytes > ResourceLimits::MAX_ARCHIVE_TOTAL_BYTES
          raise Error.integrity(
            "git catalog archive exceeds #{ResourceLimits::MAX_ARCHIVE_TOTAL_BYTES} bytes"
          )
        end
      end
      entry_count.positive?
    end

    def write_catalog_blob_atomic!(full_path, bytes)
      FileUtils.mkdir_p(File.dirname(full_path))
      partial = "#{full_path}.partial"
      File.binwrite(partial, bytes)
      File.rename(partial, full_path)
    rescue StandardError
      FileUtils.rm_f(partial)
      raise
    end

    def find_catalog_markers(start)
      current = start
      8.times do
        revision_path = File.join(current, REVISION_MARKER)
        git_dir_path = File.join(current, GIT_DIR_MARKER)
        if File.file?(revision_path) && File.file?(git_dir_path)
          revision = File.read(revision_path).strip
          git_dir = File.read(git_dir_path).strip
          return unless acceptable_revision?(revision) && acceptable_shared_bare?(git_dir)

          return {git_dir: git_dir, revision: revision, catalog_root: current}
        end
        parent = File.dirname(current)
        return if parent == current

        current = parent
      end
      nil
    end

    def acceptable_revision?(revision)
      return false if revision.empty? || revision.start_with?("-")
      return false unless [40, 64].include?(revision.length)

      revision.match?(/\A[0-9a-fA-F]+\z/)
    end

    def acceptable_shared_bare?(git_dir)
      return false unless File.file?(File.join(git_dir, "HEAD"))
      return false unless File.directory?(File.join(git_dir, "objects"))

      root = GitStore.global_cache_root
      return false unless root

      expected = File.join(root, "git")
      PathSafety.path_under_root?(expected, git_dir)
    end

    def offline_git_env(offline)
      offline ? {"GIT_NO_LAZY_FETCH" => "1"} : {}
    end
  end
end
