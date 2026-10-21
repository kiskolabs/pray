# frozen_string_literal: true

require "fileutils"
require_relative "git_run"

module Pray
  module GitLegacy
    module_function

    def quarantine_legacy_project_git_checkouts(project_root, clone_url)
      cache_root = File.join(project_root, ".pray", "cache", "git")
      return unless File.directory?(cache_root)

      Dir.children(cache_root).each do |name|
        next if name.end_with?(".legacy-git")

        path = File.join(cache_root, name)
        next unless File.exist?(File.join(path, ".git"))
        next unless origin_matches?(path, clone_url)

        legacy = File.join(cache_root, "#{name}.legacy-git")
        FileUtils.rm_rf(legacy) if File.exist?(legacy)
        FileUtils.mv(path, legacy)
      end
    end

    def origin_matches?(repository, clone_url)
      output, status = GitRun.capture_git(repository, "remote", "get-url", "origin")
      return false unless status.success?

      origin = output.strip.delete_prefix("git+")
      origin == clone_url
    end
    private_class_method :origin_matches?
  end
end
