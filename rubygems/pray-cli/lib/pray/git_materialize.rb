# frozen_string_literal: true

require "fileutils"
require "open3"
require_relative "error"
require_relative "git_clone"
require_relative "git_run"
require_relative "path_safety"

module Pray
  module GitMaterialize
    REVISION_MARKER = ".pray-revision"
    GIT_DIR_MARKER = ".pray-git-dir"

    module_function

    def read_local_artifact_bytes(source_root, artifact)
      if artifact.start_with?("file://")
        path = PathSafety.join_under_root(source_root, artifact.delete_prefix("file://"))
        raise Error.resolution("package artifact path escapes distribution root") unless path

        return File.binread(path)
      end
      if artifact.match?(%r{\A[a-z][a-z0-9+.-]*:}i)
        raise Error.integrity("remote artifact path must be relative: #{artifact}")
      end

      path = PathSafety.join_under_root(source_root, artifact)
      raise Error.resolution("package artifact path escapes distribution root") unless path

      materialize_catalog_file(source_root, artifact) unless File.exist?(path)
      raise Error.resolution("package artifact missing at #{path}") unless File.exist?(path)

      File.binread(path)
    end

    def materialize_catalog_tree(git_dir, dest, revision, subdir, refresh)
      return if !refresh && catalog_matches?(dest, git_dir, revision)

      FileUtils.rm_rf(dest) if File.exist?(dest)
      FileUtils.mkdir_p(dest)
      unpacked = GitClone.catalog_sparse_cones(subdir).any? do |prefix|
        unpack_archive_prefix(git_dir, dest, revision, prefix)
      end
      raise Error.resolution("no pray distribution root in git source at revision #{revision}") unless unpacked

      File.write(File.join(dest, REVISION_MARKER), revision)
      File.write(File.join(dest, GIT_DIR_MARKER), git_dir)
    end

    def materialize_catalog_file(source_root, relative)
      return if File.file?(File.join(source_root, relative))

      markers = find_catalog_markers(source_root)
      return unless markers

      full_path = File.join(source_root, relative)
      repo_relative = full_path.delete_prefix(markers.fetch(:work_tree)).delete_prefix("/")
      GitRun.run_git(
        markers.fetch(:git_dir),
        "--work-tree",
        markers.fetch(:work_tree),
        "checkout",
        markers.fetch(:revision),
        "--",
        repo_relative
      )
    end

    def catalog_matches?(dest, git_dir, revision)
      revision_path = File.join(dest, REVISION_MARKER)
      git_dir_path = File.join(dest, GIT_DIR_MARKER)
      return false unless File.file?(revision_path) && File.file?(git_dir_path)
      return false unless File.read(revision_path).strip == revision
      return false unless File.read(git_dir_path).strip == git_dir

      File.directory?(File.join(dest, "v1", "packages")) ||
        File.directory?(File.join(dest, "prayers", "v1", "packages")) ||
        Dir.children(dest).any? { |name| File.directory?(File.join(dest, name, "v1", "packages")) }
    rescue Errno::ENOENT
      false
    end

    def unpack_archive_prefix(git_dir, dest, revision, prefix)
      env = ENV.to_h.merge("GIT_TERMINAL_PROMPT" => "0")
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

      _ignored, extract_status = Open3.capture2(env, "tar", "-x", "-C", dest, stdin_data: stdout)
      raise Error.resolution("failed to unpack git catalog archive") unless extract_status.success?

      true
    end

    def find_catalog_markers(start)
      current = start
      8.times do
        revision_path = File.join(current, REVISION_MARKER)
        git_dir_path = File.join(current, GIT_DIR_MARKER)
        if File.file?(revision_path) && File.file?(git_dir_path)
          revision = File.read(revision_path).strip
          git_dir = File.read(git_dir_path).strip
          return if revision.empty? || !File.exist?(git_dir)

          return {git_dir: git_dir, revision: revision, work_tree: current}
        end
        parent = File.dirname(current)
        return if parent == current

        current = parent
      end
      nil
    end
  end
end
