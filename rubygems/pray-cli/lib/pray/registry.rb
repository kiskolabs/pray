# frozen_string_literal: true

require "json"
require "net/http"
require "uri"
require "fileutils"
require "pathname"
require_relative "path_safety"

module Pray
  RegistryPackageVersion = Struct.new(
    :version, :artifact, :artifact_hash, :tree_hash, :yanked, :targets, :exports,
    :signer, :signer_fingerprint, :published_at, :signature
  ) do
    def initialize(
      artifact_hash: nil, tree_hash: nil, yanked: false, targets: [], exports: [],
      signer: nil, signer_fingerprint: nil, published_at: nil, signature: nil, **kwargs
    )
      super(**kwargs, artifact_hash: artifact_hash, tree_hash: tree_hash, yanked: yanked,
                      targets: targets, exports: exports, signer: signer,
                      signer_fingerprint: signer_fingerprint, published_at: published_at, signature: signature)
    end
  end

  RegistryPackageMetadata = Struct.new(:name, :versions) do
    def initialize(name: nil, versions: [])
      super
    end
  end

  RegistryPackageResolution = Struct.new(
    :root, :signer_fingerprint, :registry_latest_version
  )

  module Registry
    module_function

    def resolve_registry_package_root(project_root, source_url, declaration, preferred_version: nil, offline: false)
      metadata = fetch_package_metadata(source_url, declaration.name)
      registry_latest_version = registry_latest_version_label(metadata)
      selected = select_package_version(metadata, declaration.constraint, preferred_version)
      cache_directory = registry_cache_directory(
        project_root,
        source_url,
        declaration.name,
        selected.version,
        selected.artifact_hash
      )

      if cache_ready?(cache_directory, selected)
        return RegistryPackageResolution.new(
          root: cache_directory,
          signer_fingerprint: selected.signer_fingerprint,
          registry_latest_version: registry_latest_version
        )
      end

      raise Error.resolution(offline_package_error(declaration.name, selected.version)) if offline

      FileUtils.rm_rf(cache_directory) if File.exist?(cache_directory)
      FileUtils.mkdir_p(cache_directory)

      artifact_bytes = read_artifact_bytes(source_url, selected.artifact)
      validate_and_unpack(cache_directory, declaration, selected, artifact_bytes, source_url: source_url)

      RegistryPackageResolution.new(
        root: cache_directory,
        signer_fingerprint: selected.signer_fingerprint,
        registry_latest_version: registry_latest_version
      )
    end

    def resolve_local_registry_package_root(project_root, source_key, source_root, declaration, preferred_version: nil, offline: false)
      metadata_path = registry_metadata_path(source_root, declaration.name)
      unless File.exist?(metadata_path)
        raise Error.resolution(
          "package #{declaration.name} not found in distribution #{source_root.inspect}. " \
          "Missing #{metadata_path}. Check the package name, version constraint `#{declaration.constraint}`, " \
          "and that the source publishes registry metadata."
        )
      end

      metadata = parse_metadata(File.read(metadata_path, encoding: "UTF-8"))
      registry_latest_version = registry_latest_version_label(metadata)
      selected = select_package_version(metadata, declaration.constraint, preferred_version)
      cache_directory = registry_cache_directory(
        project_root,
        source_key,
        declaration.name,
        selected.version,
        selected.artifact_hash
      )

      if cache_ready?(cache_directory, selected)
        return RegistryPackageResolution.new(
          root: cache_directory,
          signer_fingerprint: selected.signer_fingerprint,
          registry_latest_version: registry_latest_version
        )
      end

      raise Error.resolution(offline_package_error(declaration.name, selected.version)) if offline

      FileUtils.rm_rf(cache_directory) if File.exist?(cache_directory)
      FileUtils.mkdir_p(cache_directory)

      artifact_bytes = read_local_artifact_bytes(source_root, selected.artifact)
      validate_and_unpack(cache_directory, declaration, selected, artifact_bytes, source_url: source_root)

      RegistryPackageResolution.new(
        root: cache_directory,
        signer_fingerprint: selected.signer_fingerprint,
        registry_latest_version: registry_latest_version
      )
    end

    def fetch_package_metadata(source_url, package_name)
      if local_source?(source_url)
        metadata_path = registry_metadata_path(source_url, package_name)
        return parse_metadata(File.read(metadata_path, encoding: "UTF-8"))
      end

      PathSafety.reject_unsafe_package_name!(package_name)
      response = http_get(join_url(source_url, "v1/packages/#{package_name}.json"))
      parse_metadata(response)
    end

    def parse_metadata(text)
      data = JSON.parse(text)
      RegistryPackageMetadata.new(
        name: data["name"],
        versions: Array(data["versions"]).map { |entry| version_from_hash(entry) }
      )
    end

    def version_from_hash(entry)
      RegistryPackageVersion.new(
        version: entry["version"],
        artifact: entry["artifact"],
        artifact_hash: entry["artifact_hash"],
        tree_hash: entry["tree_hash"],
        yanked: entry["yanked"] || false,
        targets: entry["targets"] || [],
        exports: entry["exports"] || [],
        signer: entry["signer"],
        signer_fingerprint: entry["signer_fingerprint"],
        published_at: entry["published_at"],
        signature: entry["signature"]
      )
    end

    def select_package_version(metadata, constraint, preferred_version)
      if preferred_version
        version = metadata.versions.find { |entry| entry.version == preferred_version && !entry.yanked }
        return version if version && Constraint.version_satisfies(version.version, constraint)
      end

      selected = nil
      metadata.versions.each do |version|
        next if version.yanked
        next unless Constraint.version_satisfies(version.version, constraint)

        selected = version if selected.nil? || compare_versions(version.version, selected.version).positive?
      end

      raise Error.resolution("no registry version for #{metadata.name} satisfies #{constraint}") unless selected

      selected
    end

    def registry_latest_version_label(metadata)
      highest = metadata.versions.reject(&:yanked).max_by { |entry| Gem::Version.new(entry.version) }
      highest&.version
    end

    def compare_versions(left, right)
      Gem::Version.new(left) <=> Gem::Version.new(right)
    end

    def validate_and_unpack(cache_directory, declaration, selected, artifact_bytes, source_url: nil)
      if selected.artifact_hash
        artifact_hash = Hashing.sha256_prefixed(artifact_bytes)
        if artifact_hash != selected.artifact_hash
          raise Error.integrity(
            "package artifact hash mismatch for #{declaration.name} #{selected.version}"
          )
        end
      end

      verify_registry_signature!(declaration, selected, artifact_bytes)
      Trust.verify_publisher_fingerprint!(source_url, selected) if source_url

      Archive.unpack_praypkg(artifact_bytes, cache_directory)
      spec_path = Resolve.find_prayspec_file(cache_directory)
      spec = Pray.parse_package_spec(File.read(spec_path)).canonicalized
      if spec.name != declaration.name
        raise Error.resolution(
          "package path #{cache_directory.inspect} declares #{spec.name.inspect}, expected #{declaration.name.inspect}"
        )
      end
      if spec.version != selected.version
        raise Error.resolution(
          "package #{declaration.name} version #{spec.version} does not match registry version #{selected.version}"
        )
      end
      if selected.tree_hash
        actual_tree_hash = spec.tree_hash_for_root(cache_directory)
        if actual_tree_hash != selected.tree_hash
          raise Error.integrity(
            "package tree hash mismatch for #{declaration.name} #{selected.version}"
          )
        end
      end
    end

    def read_local_artifact_bytes(source_root, artifact)
      if artifact.start_with?("http://", "https://")
        return http_get(artifact)
      end
      if artifact.start_with?("file://")
        path = PathSafety.join_under_root(source_root, artifact.delete_prefix("file://"))
        raise Error.resolution("package artifact path escapes distribution root") unless path
        return File.binread(path)
      end

      path = PathSafety.join_under_root(source_root, artifact)
      raise Error.resolution("package artifact path escapes distribution root") unless path
      raise Error.resolution("package artifact missing at #{path}") unless File.exist?(path)

      File.binread(path)
    end

    def read_artifact_bytes(source_url, artifact)
      return read_local_artifact_bytes(source_url, artifact) if local_source?(source_url)

      http_get(join_url(source_url, artifact))
    end

    def cache_ready?(cache_directory, selected)
      return false unless File.directory?(cache_directory)

      spec_path = Resolve.find_prayspec_file(cache_directory)
      spec = Pray.parse_package_spec(File.read(spec_path)).canonicalized
      spec.version == selected.version
    rescue Errno::ENOENT, Errno::ENOTDIR, SystemCallError
      false
    end

    def registry_metadata_path(source_root, package_name)
      PathSafety.reject_unsafe_package_name!(package_name)
      packages_root = PathSafety.join_under_root(source_root, "v1", "packages")
      raise Error.resolution("invalid package name: #{package_name.inspect}") unless packages_root

      metadata_path = Pathname.new(packages_root).join("#{package_name}.json").cleanpath.to_s
      unless PathSafety.path_under_root?(packages_root, metadata_path)
        raise Error.resolution("invalid package name: #{package_name.inspect}")
      end

      metadata_path
    end

    def verify_registry_signature!(declaration, selected, artifact_bytes)
      return unless selected.signature

      unless selected.signer && selected.tree_hash
        raise Error.integrity(
          "package signature metadata incomplete for #{declaration.name} #{selected.version}"
        )
      end

      expected = registry_artifact_signature(artifact_bytes, selected.tree_hash, selected.signer)
      return if expected == selected.signature

      raise Error.integrity(
        "package signature mismatch for #{declaration.name} #{selected.version}"
      )
    end

    def registry_cache_directory(project_root, source_key, package_name, version, artifact_hash)
      identifier = [
        source_key,
        package_name,
        version,
        artifact_hash || "no-artifact-hash"
      ].join(":")
      digest = Hashing.sha256_hex(identifier)[0, 16]
      File.join(project_root, ".pray", "cache", "registry", package_name.tr("/", "-"), version, digest)
    end

    def offline_package_error(package_name, version)
      "package #{package_name} #{version} is not cached and offline mode is enabled"
    end

    def local_source?(source_url)
      !source_url.start_with?("http://", "https://", "pray+ssh://", "ssh+pray://")
    end

    def join_url(base, path)
      URI.join(base.end_with?("/") ? base : "#{base}/", path.delete_prefix("/")).to_s
    end

    def http_get(url)
      uri = URI(url)
      response = http_request(uri) { |http| http.get(uri.request_uri) }
      unless response.is_a?(Net::HTTPSuccess)
        raise Error.resolution("HTTP request failed for #{url}: #{response.code}")
      end

      response.body
    end

    def http_put(url, content_type, body)
      uri = URI(url)
      request = Net::HTTP::Put.new(uri)
      request["Content-Type"] = content_type
      request.body = body
      response = http_request(uri) { |http| http.request(request) }
      unless response.is_a?(Net::HTTPSuccess)
        raise Error.resolution("HTTP upload failed for #{url}: #{response.code}")
      end

      response.body
    end

    def http_request(uri)
      Net::HTTP.start(
        uri.hostname,
        uri.port,
        use_ssl: uri.scheme == "https",
        open_timeout: 10,
        read_timeout: 30
      ) do |http|
        yield http
      end
    end

    def registry_artifact_signature(artifact_bytes, tree_hash, signer)
      payload = artifact_bytes + "\0".b + tree_hash.b + "\0".b + signer.b
      Hashing.sha256_prefixed(payload)
    end
  end
end
