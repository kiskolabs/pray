# frozen_string_literal: true

require "digest"

module Pray
  class PackageSpec
    def tree_hash_for_root(root)
      validate_relative_paths!
      file_bytes = {}
      files.each do |file|
        path = File.join(root, file)
        raise Error.integrity("package file missing: #{file}") unless File.exist?(path)
        raise Error.integrity("package file is a directory: #{file}") if File.directory?(path)

        file_bytes[file] = File.binread(path)
      end
      self.class.tree_hash_from_file_bytes(file_bytes)
    end

    def self.tree_hash_from_file_bytes(file_bytes)
      # Stream the same bytes the prior string builder emitted so the digest stays identical.
      digest = Digest::SHA256.new
      file_bytes.keys.sort.each do |path|
        bytes = file_bytes.fetch(path)
        digest << "file\0regular\0"
        digest << path
        digest << "\0"
        digest << "sha256:"
        digest << Digest::SHA256.hexdigest(bytes)
        digest << "\n"
      end
      Hashing.prefixed_hex_digest(digest.digest)
    end
  end
end
