# frozen_string_literal: true

require "pathname"

module Pray
  ENV_PROJECT_PATH = "PRAY_PATH"
  ENV_MANIFEST_PATH = "PRAY_FILE_PATH"
  ENV_ENVIRONMENT = "PRAY_ENV"

  ProjectInvocationContext = Struct.new(:project_root, :manifest_path, :environment, keyword_init: true) do
    def lockfile_path
      File.join(project_root, "Prayfile.lock")
    end
  end

  ProjectInvocationOptions = Struct.new(:project_root, :manifest_path, :environment, keyword_init: true)

  module ProjectContext
    module_function

    def from_current_directory
      from_options(ProjectInvocationOptions.new)
    end

    def from_options(options)
      cwd = Dir.pwd
      dotenv = Dotenv.load_dotenv_variables(cwd)
      project_root_hint = options.project_root ||
        env_value(ENV_PROJECT_PATH) ||
        dotenv[ENV_PROJECT_PATH] ||
        cwd
      project_root = canonicalize_path(cwd, project_root_hint)
      manifest_hint = options.manifest_path ||
        env_value(ENV_MANIFEST_PATH) ||
        dotenv[ENV_MANIFEST_PATH] ||
        "Prayfile"
      manifest_path = if Pathname(manifest_hint).absolute?
        canonicalize_path(cwd, manifest_hint)
      else
        canonicalize_path(project_root, manifest_hint)
      end
      environment = (options.environment || env_value(ENV_ENVIRONMENT) || dotenv[ENV_ENVIRONMENT])&.strip
      environment = nil if environment.nil? || environment.empty?

      ProjectInvocationContext.new(
        project_root: project_root,
        manifest_path: manifest_path,
        environment: environment
      )
    end

    def env_value(key)
      value = ENV[key]
      return nil if value.nil? || value.strip.empty?

      value
    end

    def canonicalize_path(base, path)
      resolved = Pathname(path).absolute? ? Pathname(path) : Pathname(base).join(path)
      resolved = resolved.expand_path
      resolved.exist? ? resolved.realpath.to_s : resolved.cleanpath.to_s
    rescue StandardError
      resolved.to_s
    end
  end
end
