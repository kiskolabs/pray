# frozen_string_literal: true

module Pray
  module CLI
    def version_command
      puts "pray #{Pray::VERSION}"
    end

    def clean_command(unused: false)
      project_root = Invocation.project_root
      if unused
        CacheClean.clean_unused_registry_cache(project_root)
        return
      end
      remove_path_if_exists(File.join(project_root, ".pray/cache"))
      remove_path_if_exists(File.join(project_root, ".pray/vendor"))
      remove_path_if_exists(File.join(project_root, ".pray/state.json"))
    end
  end
end
