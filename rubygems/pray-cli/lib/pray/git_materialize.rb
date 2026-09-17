# frozen_string_literal: true

require_relative "git_run"
require_relative "error"
require_relative "path_safety"

module Pray
  module GitMaterialize
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

    def materialize_catalog_file(source_root, relative)
      return if File.file?(File.join(source_root, relative))
      return unless sparse_checkout_enabled?(source_root)

      toplevel = git_toplevel(source_root)
      return unless toplevel

      repo_relative = repo_relative_text(source_root, relative)
      return unless repo_relative

      cone = artifact_sparse_cone(repo_relative)
      GitRun.run_git(toplevel, "sparse-checkout", "add", cone) if cone
      GitRun.run_git(toplevel, "checkout", "HEAD", "--", repo_relative)
    end

    def sparse_checkout_enabled?(source_root)
      output, status = GitRun.capture_git(source_root, "config", "--get", "core.sparseCheckout")
      status.success? && output.strip == "true"
    end

    def git_toplevel(source_root)
      output, status = GitRun.capture_git(source_root, "rev-parse", "--show-toplevel")
      return unless status.success?

      text = output.strip
      text.empty? ? nil : text
    end

    def repo_relative_text(source_root, relative)
      output, status = GitRun.capture_git(source_root, "rev-parse", "--show-prefix")
      return unless status.success?

      prefix = output.strip
      prefix.empty? ? relative : File.join(prefix, relative)
    end

    def artifact_sparse_cone(repo_relative)
      parts = repo_relative.split("/").reject(&:empty?)
      return repo_relative if parts.length < 2

      parts.pop
      parts.pop if parts.last != "artifacts" && parts.length > 3
      parts.join("/")
    end
  end
end
