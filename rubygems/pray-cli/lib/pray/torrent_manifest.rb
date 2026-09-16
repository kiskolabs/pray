# frozen_string_literal: true

require "json"
require "fileutils"

module Pray
  TORRENT_MANIFEST_SPEC = "pray-torrent-v1"
  DEFAULT_TORRENT_PIECE_SIZE = 16 * 1024

  module TorrentManifest
    module_function

    def path_for(artifact_path)
      "#{artifact_path}.praytorrent.json"
    end

    def payload(name:, version:, artifact_path:, archive_bytes:, trackers: [])
      {
        "spec" => TORRENT_MANIFEST_SPEC,
        "name" => name,
        "version" => version,
        "artifact_url" => artifact_path,
        "artifact_hash" => Hashing.sha256_prefixed(archive_bytes),
        "piece_size" => DEFAULT_TORRENT_PIECE_SIZE,
        "length" => archive_bytes.bytesize,
        "pieces" => piece_hashes(archive_bytes),
        "sources" => [artifact_path],
        "trackers" => Array(trackers)
      }
    end

    def bytes(name:, version:, artifact_path:, archive_bytes:, trackers: [])
      JSON.pretty_generate(
        payload(
          name: name,
          version: version,
          artifact_path: artifact_path,
          archive_bytes: archive_bytes,
          trackers: trackers
        )
      )
    end

    def write(root, name:, version:, artifact_path:, archive_bytes:, settings:)
      return unless settings.allows_torrent?

      path = File.join(root, path_for(artifact_path))
      FileUtils.mkdir_p(File.dirname(path))
      File.write(
        path,
        bytes(
          name: name,
          version: version,
          artifact_path: artifact_path,
          archive_bytes: archive_bytes,
          trackers: settings.bootstrap_trackers
        )
      )
    end

    def piece_hashes(archive_bytes)
      length = archive_bytes.bytesize
      return [] if length.zero?

      hashes = []
      start = 0
      while start < length
        hashes << Hashing.sha256_prefixed(archive_bytes.byteslice(start, DEFAULT_TORRENT_PIECE_SIZE))
        start += DEFAULT_TORRENT_PIECE_SIZE
      end
      hashes
    end

    def descriptor_present?(root, artifact_path, settings)
      !settings.allows_torrent? || File.file?(File.join(root, path_for(artifact_path)))
    end
  end
end
