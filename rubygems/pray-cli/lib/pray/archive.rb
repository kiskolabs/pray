# frozen_string_literal: true

require "json"
require "open3"
require "fileutils"
require_relative "archive_unpack"

module Pray
  module Archive
    module_function

    def build_package_archive_bytes(package)
      prayspec_path = Resolve.find_prayspec_file(package.root)
      prayspec_name = File.basename(prayspec_path)
      metadata = package_metadata_json(package)

      Dir.mktmpdir("pray-package-") do |staging|
        File.write(File.join(staging, "metadata.json"), metadata)
        File.write(File.join(staging, prayspec_name), File.binread(prayspec_path))
        package.spec.files.each do |file|
          destination = File.join(staging, file)
          FileUtils.mkdir_p(File.dirname(destination))
          File.binwrite(destination, File.binread(File.join(package.root, file)))
        end

        with_binary_process_encoding do
          tar_bytes, status = Open3.capture2(
            {"COPYFILE_DISABLE" => "1"},
            "tar", "-cf", "-", "-C", staging, "."
          )
          raise Error.integrity("failed to build package tar archive") unless status.success?

          zstd_bytes, status = Open3.capture2("zstd", "-q", "-c", stdin_data: tar_bytes)
          unless status.success?
            raise Error.unsupported("zstd is required to build package archives")
          end

          zstd_bytes
        end
      end
    end

    def write_package_archive(package, output_path)
      FileUtils.mkdir_p(File.dirname(output_path))
      File.binwrite(output_path, build_package_archive_bytes(package))
    end

    def unpack_praypkg(artifact_bytes, output_directory)
      ArchiveUnpack.unpack_praypkg(artifact_bytes, output_directory)
    end

    def package_archive_path(package_name, version)
      slug = package_name.tr("/", "-")
      File.join(".pray", "packages", "#{slug}-#{version}.praypkg")
    end

    def package_metadata_json(package)
      JSON.generate(
        "name" => package.spec.name,
        "version" => package.spec.version,
        "tree_hash" => package.tree_hash,
        "exports" => package.selected_exports
      )
    end

    def with_binary_process_encoding(&block)
      ArchiveUnpack.with_binary_process_encoding(&block)
    end
  end
end
