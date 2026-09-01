# frozen_string_literal: true

require "fileutils"

module Pray
  module RegistryInstall
    module_function

    def install_artifact_to_cache(cache_directory, declaration, selected, artifact_bytes, source_url: nil)
      staging_directory = "#{cache_directory}.staging"
      FileUtils.rm_rf(staging_directory)
      FileUtils.mkdir_p(staging_directory)
      unpacked_directory = File.join(staging_directory, "unpacked")
      FileUtils.mkdir_p(unpacked_directory)

      begin
        Registry.validate_and_unpack(
          unpacked_directory, declaration, selected, artifact_bytes, source_url: source_url
        )
        FileUtils.rm_rf(cache_directory) if File.exist?(cache_directory)
        FileUtils.mv(unpacked_directory, cache_directory)
      rescue
        FileUtils.rm_rf(staging_directory)
        raise
      else
        FileUtils.rm_rf(staging_directory)
      end
    end
  end
end
