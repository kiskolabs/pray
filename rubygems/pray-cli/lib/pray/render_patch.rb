# frozen_string_literal: true

module Pray
  module RenderPatch
    module_function

    def patch_rendered_content(existing, fresh)
      existing_segments = split_segments(existing)
      fresh_segments = split_segments(fresh)
      fresh_managed = managed_bodies(fresh_segments)
      return fresh unless overlapping_managed?(existing_segments, fresh_managed)

      used = {}
      output = +""
      remaining = prepend_new_leading_spans(
        existing_segments,
        fresh_segments,
        managed_id_set(existing_segments),
        used,
        output
      )
      splice_existing_managed(remaining, fresh_segments, fresh_managed, used, output)
      append_unused_fresh_managed(fresh_segments, used, output)
      output.end_with?("\n") ? output : "#{output}\n"
    end

    def managed_bodies(segments)
      segments.filter_map { |segment| [segment[:id], segment[:body]] if segment[:kind] == :managed }.to_h
    end

    def overlapping_managed?(existing_segments, fresh_managed)
      existing_segments.any? { |segment| segment[:kind] == :managed && fresh_managed.key?(segment[:id]) }
    end

    def managed_id_set(segments)
      segments.filter_map { |segment| segment[:id] if segment[:kind] == :managed }.to_h { |id| [id, true] }
    end

    def splice_existing_managed(remaining, fresh_segments, fresh_managed, used, output)
      remaining.each_with_index do |segment, index|
        if segment[:kind] == :text
          output << segment[:text]
          next
        end

        used[segment[:id]] = true
        output << managed_segment(segment[:id], fresh_managed.fetch(segment[:id], segment[:body]))
        next_segment = remaining[index + 1]
        next unless next_segment && next_segment[:kind] == :managed

        output << text_between(fresh_segments, segment[:id], next_segment[:id])
      end
    end

    def text_between(segments, left, right)
      copying = false
      text = +""
      segments.each do |segment|
        if segment[:kind] == :managed
          if segment[:id] == left
            copying = true
            text = +""
          elsif copying
            return (segment[:id] == right) ? text : ""
          end
        elsif copying
          text << segment[:text]
        end
      end
      ""
    end

    def append_unused_fresh_managed(fresh_segments, used, output)
      pending = +""
      seen_used = false
      fresh_segments.each do |segment|
        if segment[:kind] == :managed
          if used[segment[:id]]
            seen_used = true
            pending = +""
            next
          end
          output << pending
          pending = +""
          output << managed_segment(segment[:id], segment[:body])
        elsif seen_used
          pending << segment[:text]
        end
      end
    end

    def prepend_new_leading_spans(existing_segments, fresh_segments, existing_ids, used, output)
      missing = fresh_segments.any? { |segment| segment[:kind] == :managed && !existing_ids[segment[:id]] }
      return existing_segments unless missing

      shared_id = fresh_segments.find { |segment| segment[:kind] == :managed && existing_ids[segment[:id]] }&.[](:id)
      return existing_segments unless shared_id

      fresh_segments.each do |segment|
        break if segment[:kind] == :managed && segment[:id] == shared_id

        if segment[:kind] == :text
          output << segment[:text]
        else
          used[segment[:id]] = true
          output << managed_segment(segment[:id], segment[:body])
        end
      end
      skip_until_managed(existing_segments, shared_id)
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

    def skip_until_managed(segments, identifier)
      index = segments.index { |segment| segment[:kind] == :managed && segment[:id] == identifier }
      index ? segments[index..] : segments
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
