# frozen_string_literal: true

require_relative "resource_limits"

module Pray
  module HttpBody
    module_function

    def reject_oversized_content_length!(content_length, max_bytes = ResourceLimits::MAX_HTTP_RESPONSE_BYTES)
      return if content_length.nil?
      return if content_length <= max_bytes

      raise Error.resolution("HTTP response exceeds #{max_bytes} bytes")
    end

    def append_chunk!(body, chunk, max_bytes = ResourceLimits::MAX_HTTP_RESPONSE_BYTES)
      body << chunk
      return body if body.bytesize <= max_bytes

      raise Error.resolution("HTTP response exceeds #{max_bytes} bytes")
    end

    def read_response!(response, max_bytes = ResourceLimits::MAX_HTTP_RESPONSE_BYTES)
      length = response["content-length"]
      parsed = length && Integer(length, exception: false)
      reject_oversized_content_length!(parsed, max_bytes)
      body = +"".b
      response.read_body do |chunk|
        append_chunk!(body, chunk, max_bytes)
      end
      body
    end
  end
end
