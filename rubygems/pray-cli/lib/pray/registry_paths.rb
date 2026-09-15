# frozen_string_literal: true

require "pathname"

module Pray
  module RegistryPaths
    module_function

    def local_registry_root(project_root, source_url)
      path = source_url.delete_prefix("file://")
      Pathname.new(path).absolute? ? path : File.expand_path(path, project_root)
    end
  end
end
