# frozen_string_literal: true

module Pray
  module ServeRange
    module_function

    def apply(status, body, range_header)
      return [status, body, nil] if range_header.nil? || range_header.empty?
      return [status, body, nil] unless status == 200 && !body.empty?

      start_offset, end_offset = parse_byte_range(range_header, body.bytesize)
      unless start_offset
        return [416, "".b, "bytes */#{body.bytesize}"]
      end

      [
        206,
        body.b[start_offset..end_offset],
        "bytes #{start_offset}-#{end_offset}/#{body.bytesize}"
      ]
    end

    def parse_byte_range(header, total)
      range = header.delete_prefix("bytes=")
      start_text, end_text = range.split("-", 2)
      return unless start_text && end_text

      start_offset = Integer(start_text, exception: false)
      end_offset = Integer(end_text, exception: false)
      return if start_offset.nil? || end_offset.nil?
      return unless start_offset <= end_offset && end_offset < total

      [start_offset, end_offset]
    end
  end
end
