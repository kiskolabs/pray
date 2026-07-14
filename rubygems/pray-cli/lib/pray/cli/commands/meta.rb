# frozen_string_literal: true

module Pray
  module CLI
    def version_command
      puts "pray #{Pray::VERSION}"
    end

    def clean_command
      remove_path_if_exists(".pray/cache")
      remove_path_if_exists(".pray/vendor")
      remove_path_if_exists(".pray/state.json")
    end
  end
end
