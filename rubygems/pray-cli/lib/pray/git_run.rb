# frozen_string_literal: true

require "open3"
require_relative "error"

module Pray
  module GitRun
    module_function

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
      env = ENV.to_h.merge("GIT_TERMINAL_PROMPT" => "0")
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
