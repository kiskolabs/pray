# frozen_string_literal: true

require "digest"
require "openssl"

module Pray
  module Hashing
    module_function

    def sha256_hex(bytes)
      Digest::SHA256.hexdigest(bytes)
    end

    def sha256_prefixed(bytes)
      prefixed_hex_digest(Digest::SHA256.digest(bytes))
    end

    def prefixed_hex_digest(digest_bytes)
      "sha256:#{digest_bytes.unpack1("H*")}"
    end

    def marker_id(seed)
      sha256_hex(seed)[0, 8]
    end

    def normalize_line_endings(text)
      text.gsub("\r\n", "\n").tr("\r", "\n")
    end

    def checksum_managed_span_content(body)
      normalized = normalize_line_endings(body).sub(/\n+\z/, "")
      sha256_prefixed(normalized)
    end

    def checksum_managed_body_line_refs(body_lines)
      lines = trim_trailing_empty_lines(body_lines)
      digest = OpenSSL::Digest::SHA256.new
      lines.each_with_index do |line, index|
        digest << "\n" if index.positive?
        update_line_endings_normalized(digest, line)
      end
      prefixed_hex_digest(digest.digest)
    end

    def trim_trailing_empty_lines(lines)
      trimmed = lines.dup
      trimmed.pop while trimmed.last == ""
      trimmed
    end

    def update_line_endings_normalized(digest, line)
      digest << if line.include?("\r")
        normalize_line_endings(line)
      else
        line
      end
    end
  end
end
