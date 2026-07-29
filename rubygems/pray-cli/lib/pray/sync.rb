# frozen_string_literal: true

require "json"
require "fileutils"
require "uri"

module Pray
  module Sync
    module_function

    def synchronize_registry(root, peer_sources)
      root = File.expand_path(root)
      if peer_sources.empty?
        raise Error.unsupported("no federation peers configured")
      end

      peer_sources.each do |peer|
        if peer.start_with?("pray+ssh://", "ssh+pray://")
          raise Error.unsupported("pray_ssh sync peers are not implemented yet in pray-cli Ruby")
        end
      end

      known_peers = load_known_peers(root)
      pending = peer_sources.dup
      discovered = peer_sources.to_set
      peer_sources.each { |url| upsert_known_peer(known_peers, {"name" => url, "url" => url, "public" => false}) }

      package_versions = {}
      peer_count = 0

      while (peer_source = pending.shift)
        next unless discovered.delete?(peer_source)

        peer_count += 1
        discovery = fetch_json("#{trim_slash(peer_source)}/.well-known/pray-federation.json")
        unless discovery["spec"] == "pray-federation-v1"
          raise Error.resolution("peer #{peer_source} does not speak the pray federation protocol")
        end

        Array(discovery["peers"]).each do |peer|
          peer = normalize_peer(peer)
          next if peer["url"] == peer_source

          upsert_known_peer(known_peers, peer)
          next if pending.include?(peer["url"]) || discovered.include?(peer["url"])

          pending << peer["url"]
          discovered << peer["url"]
        end

        index = fetch_json("#{trim_slash(peer_source)}/v1/sync/index")
        unless index["spec"] == "prayfile-distribution-1"
          raise Error.resolution(
            "peer #{peer_source} returned unsupported registry index spec: #{index["spec"]}"
          )
        end

        Array(index["packages"]).each do |summary|
          name = summary["name"]
          metadata = fetch_json("#{trim_slash(peer_source)}/v1/sync/package/#{name}")
          unless metadata["name"] == name
            raise Error.resolution(
              "peer #{peer_source} returned mismatched package metadata for #{name}"
            )
          end
          sync_package(root, peer_source, metadata, package_versions)
        end
      end

      write_known_peers(root, known_peers)
      write_local_index(root, package_versions)
      {peers: peer_count, packages: package_versions.length, known_peers: known_peers.length}
    end

    def load_sync_peers(root)
      path = File.join(root, "v1", "peers.json")
      unless File.file?(path)
        raise Error.unsupported("no federation peers configured")
      end

      Array(JSON.parse(File.read(path))).map { |peer| normalize_peer(peer) }
    rescue JSON::ParserError => error
      raise Error.parse("peer list", error.message)
    end

    def sync_package(root, peer_source, metadata, package_versions)
      name = metadata["name"]
      package_versions[name] ||= load_local_versions(root, name)

      Array(metadata["versions"]).each do |version_data|
        version = registry_version_from_transport(version_data)
        existing = package_versions[name][version.version]
        if existing
          if same_identity?(existing, version)
            package_versions[name][version.version] = existing
            next
          end
          raise Error.integrity(
            "conflicting metadata for package #{name} version #{version.version}"
          )
        end

        artifact_hash = version.artifact_hash
        unless artifact_hash
          raise Error.integrity("federation package #{name} #{version.version} is missing an artifact hash")
        end

        bytes = fetch_bytes(peer_source, version.artifact)
        computed = Hashing.sha256_prefixed(bytes)
        if computed != artifact_hash
          raise Error.integrity("artifact hash mismatch for #{name} #{version.version}")
        end

        write_artifact(root, version.artifact, bytes)
        package_versions[name][version.version] = version
      end
    end

    def registry_version_from_transport(data)
      signer = data.dig("publisher", "id") || data["signer"]
      fingerprint = data.dig("publisher", "key_fingerprint") || data["signer_fingerprint"]
      signature = data.dig("signature", "value") || data["signature"]
      RegistryPackageVersion.new(
        version: data["version"].to_s,
        artifact: data["artifact"].to_s,
        artifact_hash: data["artifact_hash"].to_s,
        tree_hash: data["tree_hash"],
        yanked: data.fetch("yanked", false),
        targets: Array(data["targets"]),
        exports: Array(data["exports"]),
        signer: signer,
        signer_fingerprint: fingerprint,
        published_at: data["published_at"],
        signature: signature.is_a?(Hash) ? signature["value"] : signature
      )
    end

    def same_identity?(left, right)
      left.artifact_hash == right.artifact_hash && left.tree_hash == right.tree_hash
    end

    def load_local_versions(root, package_name)
      path = Publish.registry_metadata_path(root, package_name)
      metadata = Publish.load_registry_package_metadata(path, package_name)
      metadata.versions.to_h { |version| [version.version, version] }
    end

    def write_local_index(root, package_versions)
      package_versions.each do |name, versions|
        metadata = RegistryPackageMetadata.new(name: name, versions: versions.values)
        Publish.write_registry_package_metadata(
          Publish.registry_metadata_path(root, name), metadata
        )
      end
      index = Publish.load_registry_index(root)
      names = index.packages.to_set
      package_versions.each_key { |name| names << name }
      index.packages = names.sort
      Publish.write_registry_index(root, index)
    end

    def write_artifact(root, artifact_path, bytes)
      relative = PathSafety.sanitize_relative_path(artifact_path)
      path = File.join(root, relative)
      FileUtils.mkdir_p(File.dirname(path))
      File.binwrite(path, bytes)
    end

    def load_known_peers(root)
      path = File.join(root, "v1", "peers.json")
      return [] unless File.file?(path)

      Array(JSON.parse(File.read(path))).map { |peer| normalize_peer(peer) }
    rescue JSON::ParserError
      []
    end

    def write_known_peers(root, peers)
      path = File.join(root, "v1", "peers.json")
      FileUtils.mkdir_p(File.dirname(path))
      File.write(path, JSON.pretty_generate(peers))
    end

    def upsert_known_peer(peers, peer)
      existing = peers.find { |entry| entry["url"] == peer["url"] }
      if existing
        existing.merge!(peer)
      else
        peers << peer
      end
    end

    def normalize_peer(peer)
      url = peer["url"].to_s
      raise Error.parse("peer list", "peer url is required") if url.empty?

      {"name" => peer["name"].to_s.empty? ? url : peer["name"], "url" => url,
       "public" => !!peer["public"]}
    end

    def fetch_json(url)
      JSON.parse(Registry.http_get(url))
    rescue JSON::ParserError => error
      raise Error.parse("federation response", error.message)
    end

    def fetch_bytes(peer_source, artifact)
      if artifact.start_with?("http://", "https://")
        return Registry.http_get(artifact).b
      end

      Registry.http_get("#{trim_slash(peer_source)}/#{artifact.delete_prefix("/")}").b
    end

    def trim_slash(value)
      value.to_s.sub(%r{/+\z}, "")
    end
  end
end
