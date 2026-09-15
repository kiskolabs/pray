# frozen_string_literal: true

require "fileutils"
require "pathname"
require_relative "archive_unpack"
require_relative "hashing"
require_relative "resource_limits"

module Pray
  module ResolveTarball
    module_function

    def package_root(project_root, tarball, offline:)
      artifact_bytes = read_tarball_bytes(project_root, tarball, offline: offline)
      if artifact_bytes.bytesize > ResourceLimits::MAX_ARCHIVE_TOTAL_BYTES
        raise Error.integrity(
          "package archive exceeds #{ResourceLimits::MAX_ARCHIVE_TOTAL_BYTES} bytes"
        )
      end

      cache_key = Hashing.sha256_prefixed(artifact_bytes).delete_prefix("sha256:")[0, 16]
      cache_directory = File.join(project_root, ".pray", "cache", "tarball", cache_key)
      if File.directory?(cache_directory)
        begin
          Resolve.find_prayspec_file(cache_directory)
          return cache_directory
        rescue Error
          nil
        end
      end

      staging_directory = "#{cache_directory}.staging"
      FileUtils.rm_rf(staging_directory)
      FileUtils.mkdir_p(staging_directory)
      ArchiveUnpack.unpack_praypkg(artifact_bytes, staging_directory)
      Resolve.find_prayspec_file(staging_directory)
      FileUtils.mkdir_p(File.dirname(cache_directory))
      FileUtils.rm_rf(cache_directory)
      FileUtils.mv(staging_directory, cache_directory)
      cache_directory
    end

    def read_tarball_bytes(project_root, tarball, offline:)
      if tarball.start_with?("http://", "https://")
        if offline
          raise Error.resolution(
            "tarball #{tarball} is not cached locally and offline mode is enabled"
          )
        end
        return Registry.http_get(tarball)
      end

      path = local_tarball_path(project_root, tarball)
      unless File.file?(path)
        raise Error.resolution("tarball missing at #{path}")
      end

      File.binread(path)
    end

    def local_tarball_path(project_root, tarball)
      path = tarball.delete_prefix("file://")
      Pathname.new(path).absolute? ? path : File.expand_path(path, project_root)
    end
  end
end
