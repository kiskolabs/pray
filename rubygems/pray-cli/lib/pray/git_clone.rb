# frozen_string_literal: true

require_relative "git_run"

module Pray
  module GitClone
    module_function

    def clone_bare_git_db(working_directory, source, destination, quiet:)
      local = source.start_with?("file://", "/") ? ["--no-local"] : []
      filtered = ["clone", "--bare", "--depth", "1", "--filter=blob:none", *local, source, destination]
      filtered.insert(1, "--quiet") if quiet
      return if GitRun.try_run_git(working_directory, *filtered)

      full = ["clone", "--bare", "--depth", "1", *local, source, destination]
      full.insert(1, "--quiet") if quiet
      GitRun.run_git(working_directory, *full)
    end

    def catalog_sparse_cones(subdir)
      if subdir && !subdir.empty?
        ["#{subdir}/v1/packages"]
      else
        ["v1/packages", "prayers/v1/packages"]
      end
    end
  end
end
