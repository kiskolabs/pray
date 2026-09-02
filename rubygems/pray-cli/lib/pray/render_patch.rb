# frozen_string_literal: true

module Pray
  module RenderPatch
    module_function

    def patch_rendered_content(existing, fresh)
      existing_segments = split_segments(existing)
      fresh_segments = split_segments(fresh)
      fresh_managed = fresh_segments.filter_map do |segment|
        [segment[:id], segment[:body]] if segment[:kind] == :managed
      end.to_h
      overlap = existing_segments.any? do |segment|
        segment[:kind] == :managed && fresh_managed.key?(segment[:id])
      end
      return fresh unless overlap

      used = {}
      output = existing_segments.map do |segment|
        next segment[:text] if segment[:kind] == :text

        used[segment[:id]] = true
        managed_segment(segment[:id], fresh_managed.fetch(segment[:id], segment[:body]))
      end.join
      fresh_segments.each do |segment|
        next unless segment[:kind] == :managed
        next if used[segment[:id]]

        output << managed_segment(segment[:id], segment[:body])
      end
      output.end_with?("\n") ? output : "#{output}\n"
    end

    def relocate_managed_spans(content, spans)
      positions = marker_positions(lines_of(content))
      spans.map do |span|
        position = positions[span.id]
        next span unless position

        span.dup.tap do |relocated|
          relocated.open_line = position[0]
          relocated.close_line = position[1]
        end
      end
    end

    def split_segments(content)
      lines = lines_of(content)
      segments = []
      text = +""
      index = 0
      while index < lines.length
        identifier = marker_id(lines[index])
        close = find_closing_marker(lines, index + 1, identifier) if identifier
        if identifier && close
          segments << {kind: :text, text: text} unless text.empty?
          text = +""
          body_lines = lines[(index + 1)...close]
          body = body_lines.empty? ? "" : "#{body_lines.join("\n")}\n"
          segments << {kind: :managed, id: identifier, body: body}
          index = close + 1
        else
          text << "#{lines[index]}\n"
          index += 1
        end
      end
      segments << {kind: :text, text: text} unless text.empty?
      segments
    end

    def lines_of(content)
      content.lines(chomp: true)
    end

    def find_closing_marker(lines, start, identifier)
      (start...lines.length).find { |index| marker_id(lines[index]) == identifier }
    end

    def marker_positions(lines)
      positions = {}
      active = nil
      lines.each_with_index do |line, index|
        identifier = marker_id(line)
        next unless identifier&.match?(/\A[a-z0-9]+\z/)

        if active.nil?
          active = [identifier, index + 1]
        elsif active[0] == identifier
          positions[identifier] = [active[1], index + 1]
          active = nil
        end
      end
      positions
    end

    def marker_id(line)
      match = line.strip.match(/\A<!-- pray:(.+) -->\z/)
      identifier = match && match[1]
      identifier unless identifier == "0 ignore-comments"
    end

    def managed_segment(identifier, body)
      content = body.empty? ? "" : "#{body.sub(/\n+\z/, "")}\n"
      "<!-- pray:#{identifier} -->\n#{content}<!-- pray:#{identifier} -->\n"
    end
  end
end
