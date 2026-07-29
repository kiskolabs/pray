# frozen_string_literal: true

require "json"

module Pray
  module ServeFederation
    module_function

    def discovery_response(root)
      peers = read_peers(root)
      body = {
        "spec" => "pray-federation-v1",
        "server" => {
          "name" => "pray",
          "version" => VERSION,
          "capabilities" => ["static_registry", "federation"]
        },
        "sync" => {
          "index_url" => "/v1/sync/index",
          "package_url" => "/v1/sync/package/{name}",
          "artifact_url" => "/v1/artifacts/{package}/{version}/{artifact}",
          "since_param" => "since"
        },
        "peers" => peers
      }
      Serve.ok_response("application/json", JSON.pretty_generate(body))
    end

    def index_response(root)
      index = Publish.load_registry_index(root)
      packages = index.packages.filter_map do |name|
        metadata = Publish.load_registry_package_metadata(
          Publish.registry_metadata_path(root, name), name
        )
        next if metadata.versions.empty?

        updated_at = metadata.versions.map { |version| version.published_at.to_s }.max || "0"
        {
          "name" => name,
          "updated_at" => updated_at,
          "url" => "/v1/sync/package/#{name}"
        }
      end
      body = {
        "spec" => "prayfile-distribution-1",
        "sync_version" => 0,
        "packages" => packages
      }
      Serve.ok_response("application/json", JSON.pretty_generate(body))
    end

    def package_response(root, path)
      name = path.delete_prefix("/v1/sync/package/")
      metadata_path = Publish.registry_metadata_path(root, name)
      return Serve.not_found unless File.file?(metadata_path)

      metadata = Publish.load_registry_package_metadata(metadata_path, name)
      body = {
        "name" => metadata.name,
        "updated_at" => metadata.versions.map { |version| version.published_at.to_s }.max || "0",
        "versions" => metadata.versions.map { |version| transport_version(version) }
      }
      Serve.ok_response("application/json", JSON.pretty_generate(body))
    end

    def append_confession(root, body)
      FileUtils.mkdir_p(File.join(root, "v1"))
      path = File.join(root, "v1", "confessions.jsonl")
      File.open(path, "a") { |file| file.puts(body.to_s.b) }
      Serve.ok_response("application/json", JSON.generate({"status" => "ok"}))
    end

    def transport_version(version)
      hash = {
        "version" => version.version,
        "artifact" => version.artifact,
        "artifact_hash" => version.artifact_hash.to_s,
        "tree_hash" => version.tree_hash.to_s,
        "yanked" => version.yanked,
        "targets" => version.targets,
        "exports" => version.exports,
        "published_at" => version.published_at.to_s
      }
      if version.signer || version.signer_fingerprint
        hash["publisher"] = {
          "id" => version.signer.to_s,
          "key_fingerprint" => version.signer_fingerprint.to_s
        }
      end
      if version.signature
        hash["signature"] = {
          "public_key" => version.signer.to_s,
          "signature" => version.signature,
          "value" => version.signature
        }
      end
      hash
    end

    def read_peers(root)
      path = File.join(root, "v1", "peers.json")
      return [] unless File.file?(path)

      Array(JSON.parse(File.read(path)))
    rescue JSON::ParserError
      []
    end
  end
end
