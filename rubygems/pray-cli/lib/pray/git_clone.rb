# frozen_string_literal: true

require_relative "git_run"

module Pray
  module GitClone
    module_function

    def clone_git_cache(working_directory, source, destination, quiet:)
      filtered = ["clone", "--depth", "1", "--filter=blob:none", "--sparse", source, destination]
      filtered.insert(1, "--quiet") if quiet
      return if GitRun.try_run_git(working_directory, *filtered)

      full = ["clone", "--depth", "1", source, destination]
      full.insert(1, "--quiet") if quiet
      GitRun.run_git(working_directory, *full)
    end

    def apply_sparse_checkout(repository, subdir = nil)
      GitRun.run_git(repository, "sparse-checkout", "init", "--cone")
      GitRun.run_git(repository, "sparse-checkout", "set", *catalog_sparse_cones(subdir))
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
