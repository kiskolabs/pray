# frozen_string_literal: true

require "json"
require "fileutils"
require "time"

module Pray
  RegistryIndex = Struct.new(:spec, :packages) do
    def initialize(spec: "prayfile-distribution-1", packages: [])
      super
    end
  end

  module Publish
    module_function

    def publish_to_root(project, root, signer: "local", signer_fingerprint: nil)
      root = File.expand_path(root)
      index = load_registry_index(root)
      package_names = index.packages.to_set

      project.packages.each do |package|
        artifact_path = registry_artifact_path(package.declaration.name, package.spec.version)
        metadata_path = registry_metadata_path(root, package.declaration.name)
        metadata = load_registry_package_metadata(metadata_path, package.declaration.name)
        existing = metadata.versions.find { |entry| entry.version == package.spec.version }
        stored_artifact = stored_package_artifact(root, artifact_path, package, existing)
        if stored_artifact && stored_publish_matches?(
          stored_artifact, package, signer, signer_fingerprint, existing
        )
          package_names << package.declaration.name
          write_registry_package_metadata(metadata_path, metadata)
          next
        end

        archive_bytes = Archive.build_package_archive_bytes(package)
        write_output_bytes(File.join(root, artifact_path), archive_bytes)
        version_entry = published_registry_package_version(
          package,
          signer,
          signer_fingerprint,
          archive_bytes,
          artifact_path
        )
        preserve_existing_publish_metadata(metadata, version_entry, stored_artifact)
        metadata.versions.reject! { |entry| entry.version == version_entry.version }
        metadata.versions << version_entry
        write_registry_package_metadata(metadata_path, metadata)
        package_names << package.declaration.name
      end

      index.packages = package_names.sort
      write_registry_index(root, index)
    end

    def publish_to_server(project, server_url, signer: "local", signer_fingerprint: nil)
      project.packages.each do |package|
        archive_bytes = Archive.build_package_archive_bytes(package)
        artifact_path = registry_artifact_path(package.declaration.name, package.spec.version)
        Registry.http_put(join_url(server_url, artifact_path), "application/octet-stream", archive_bytes)

        metadata = RegistryPackageMetadata.new(
          name: package.declaration.name,
          versions: [
            published_registry_package_version(
              package,
              signer,
              signer_fingerprint,
              archive_bytes,
              artifact_path
            )
          ]
        )
        Registry.http_put(
          join_url(server_url, "v1/packages/#{package.declaration.name}.json"),
          "application/json",
          JSON.pretty_generate(metadata_to_hash(metadata))
        )
      end
    end

    def published_registry_package_version(package, signer, signer_fingerprint, archive_bytes, artifact_path)
      RegistryPackageVersion.new(
        version: package.spec.version,
        artifact: artifact_path,
        artifact_hash: Hashing.sha256_prefixed(archive_bytes),
        tree_hash: package.tree_hash,
        yanked: false,
        targets: package.spec.targets,
        exports: package.spec.exports.keys,
        signer: signer,
        signer_fingerprint: signer_fingerprint,
        published_at: Time.now.to_i,
        signature: Registry.registry_artifact_signature(archive_bytes, package.tree_hash, signer)
      )
    end

    def stored_package_artifact(root, artifact_path, package, existing)
      return unless existing&.artifact == artifact_path && existing.tree_hash == package.tree_hash

      stored_path = File.join(root, artifact_path)
      return unless File.file?(stored_path)

      artifact_bytes = File.binread(stored_path)
      return unless existing.artifact_hash == Hashing.sha256_prefixed(artifact_bytes)
      return unless stored_prayspec_matches?(artifact_bytes, package.root)

      artifact_bytes
    end

    def stored_publish_matches?(artifact_bytes, package, signer, signer_fingerprint, existing)
      existing.signer == signer &&
        existing.signer_fingerprint == signer_fingerprint &&
        existing.signature == Registry.registry_artifact_signature(artifact_bytes, package.tree_hash, signer)
    end

    def stored_prayspec_matches?(artifact_bytes, package_root)
      Dir.mktmpdir("pray-publish-check-") do |directory|
        Archive.unpack_praypkg(artifact_bytes, directory)
        stored_path = Resolve.find_prayspec_file(directory)
        current_path = Resolve.find_prayspec_file(package_root)
        File.basename(stored_path) == File.basename(current_path) &&
          File.binread(stored_path) == File.binread(current_path)
      end
    rescue Error, SystemCallError
      false
    end

    def preserve_existing_publish_metadata(metadata, version_entry, stored_artifact)
      existing = metadata.versions.find { |entry| entry.version == version_entry.version }
      return unless existing

      version_entry.yanked = existing.yanked
      version_entry.published_at = existing.published_at if stored_artifact
    end

    def load_registry_index(root)
      path = File.join(root, "v1", "index.json")
      return RegistryIndex.new unless File.exist?(path)

      data = JSON.parse(File.read(path))
      RegistryIndex.new(spec: data["spec"], packages: data["packages"] || [])
    end

    def write_registry_index(root, index)
      path = File.join(root, "v1", "index.json")
      FileUtils.mkdir_p(File.dirname(path))
      File.write(path, JSON.pretty_generate({"spec" => index.spec, "packages" => index.packages}))
    end

    def load_registry_package_metadata(path, package_name)
      if File.exist?(path)
        data = JSON.parse(File.read(path))
        RegistryPackageMetadata.new(
          name: data["name"],
          versions: Array(data["versions"]).map { |entry| Registry.version_from_hash(entry) }
        )
      else
        RegistryPackageMetadata.new(name: package_name, versions: [])
      end
    end

    def write_registry_package_metadata(path, metadata)
      FileUtils.mkdir_p(File.dirname(path))
      File.write(path, JSON.pretty_generate(metadata_to_hash(metadata)))
    end

    def metadata_to_hash(metadata)
      {
        "name" => metadata.name,
        "versions" => metadata.versions.map { |entry| version_to_hash(entry) }
      }
    end

    def version_to_hash(entry)
      hash = {
        "version" => entry.version,
        "artifact" => entry.artifact,
        "yanked" => entry.yanked,
        "targets" => entry.targets,
        "exports" => entry.exports
      }
      hash["artifact_hash"] = entry.artifact_hash if entry.artifact_hash
      hash["tree_hash"] = entry.tree_hash if entry.tree_hash
      hash["signer"] = entry.signer if entry.signer
      hash["signer_fingerprint"] = entry.signer_fingerprint if entry.signer_fingerprint
      hash["published_at"] = entry.published_at if entry.published_at
      hash["signature"] = entry.signature if entry.signature
      hash
    end

    def registry_metadata_path(root, package_name)
      File.join(root, "v1", "packages", "#{package_name}.json")
    end

    def registry_artifact_path(package_name, version)
      artifact_name = "#{package_name.tr("/", "-")}-#{version}.praypkg"
      "v1/artifacts/#{package_name}/#{version}/#{artifact_name}"
    end

    def write_output_bytes(path, bytes)
      FileUtils.mkdir_p(File.dirname(path))
      File.binwrite(path, bytes)
    end

    def join_url(base, path)
      Registry.join_url(base, path)
    end
  end
end
