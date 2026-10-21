# frozen_string_literal: true

require "open3"
require_relative "error"

module Pray
  module GitRun
    # Mirrors git's own sanitize_repo_env: `git rev-parse --local-env-vars` minus
    # GIT_CONFIG_PARAMETERS and GIT_CONFIG_COUNT. Git exports these to hooks, so
    # pray started from a hook would otherwise run every cache command against the
    # host repository. The two config variables pass through, as they do when git
    # enters a submodule, because callers such as CI jobs use them to hand git
    # credentials or URL rewrites, and they cannot select a repository.
    REPOSITORY_LOCAL_VARIABLES = %w[
      GIT_ALTERNATE_OBJECT_DIRECTORIES
      GIT_CONFIG
      GIT_OBJECT_DIRECTORY
      GIT_DIR
      GIT_WORK_TREE
      GIT_IMPLICIT_WORK_TREE
      GIT_GRAFT_FILE
      GIT_INDEX_FILE
      GIT_NO_REPLACE_OBJECTS
      GIT_REPLACE_REF_BASE
      GIT_PREFIX
      GIT_SHALLOW_FILE
      GIT_COMMON_DIR
    ].freeze

    module_function

    # Process.spawn merges this hash into the inherited environment, so a nil
    # value is what removes a variable. Credential, SSH, proxy and global config
    # variables still pass through.
    def git_env(extra = {})
      REPOSITORY_LOCAL_VARIABLES.to_h { |name| [name, nil] }.merge(extra)
    end

    def run_git(cwd, *arguments)
      output, status = capture_git(cwd, *arguments)
      return if status.success?

      raise Error.resolution(command_error("git #{arguments.join(" ")}", output))
    end

    def try_run_git(cwd, *arguments)
      _output, status = capture_git(cwd, *arguments)
      status.success?
    end

    def capture_git(cwd, *arguments)
      env = git_env("GIT_TERMINAL_PROMPT" => "0")
      Open3.capture2e(
        env,
        "git",
        "-c",
        "protocol.file.allow=always",
        "-C",
        cwd,
        *arguments,
        stdin_data: ""
      )
    end

    def command_error(program, output)
      message = output.strip
      message.empty? ? "#{program} failed" : "#{program} failed: #{message}"
    end
  end
end
