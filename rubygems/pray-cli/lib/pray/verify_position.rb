# frozen_string_literal: true

module Pray
  module VerifyPosition
    module_function

    PositionDriftSummary = Struct.new(
      :target_path,
      :marker_count,
      :uniform_delta,
      :first_id,
      :lock_open,
      :lock_close,
      :file_open,
      :file_close,
      :cause,
      keyword_init: true
    )

    def summarize_position_drift(target_path, spans, markers, on_disk_lines, fresh_lines, local_files)
      drifted = []
      spans.each do |span|
        marker = markers[span.id]
        next if marker.nil?

        open_line, close_line, checksum = marker
        next if checksum != span.ideal_checksum
        next if open_line == span.open_line && close_line == span.close_line

        drifted << {
          id: span.id,
          lock_open: span.open_line,
          lock_close: span.close_line,
          file_open: open_line,
          file_close: close_line
        }
      end
      return nil if drifted.empty?

      drifted.sort_by! { |marker| marker[:file_open] }
      first = drifted.first
      deltas = drifted.map { |marker| marker[:file_open] - marker[:lock_open] }
      uniform_delta = (deltas.all? { |delta| delta == deltas.first }) ? deltas.first : nil
      cause = if fresh_lines
        unmarked_drift_cause(target_path, on_disk_lines, fresh_lines, local_files)
      end

      PositionDriftSummary.new(
        target_path: target_path,
        marker_count: drifted.length,
        uniform_delta: uniform_delta,
        first_id: first[:id],
        lock_open: first[:lock_open],
        lock_close: first[:lock_close],
        file_open: first[:file_open],
        file_close: first[:file_close],
        cause: cause
      )
    end

    def format_position_drift_message(summary)
      shift = if summary.uniform_delta && !summary.uniform_delta.zero?
        format(" (%+d lines)", summary.uniform_delta)
      else
        ""
      end
      marker_word = (summary.marker_count == 1) ? "marker" : "markers"
      parts = [
        "`#{summary.target_path}` position drift#{shift} across #{summary.marker_count} #{marker_word}",
        "first marker `#{summary.first_id}` lock #{summary.lock_open}:#{summary.lock_close}, file #{summary.file_open}:#{summary.file_close}"
      ]
      parts << "cause: #{summary.cause}" if summary.cause
      parts << "Align unmarked text with compose sources, or run `pray install` to refresh lock positions."
      parts.join("; ")
    end

    def unmarked_drift_cause(target_path, on_disk_lines, fresh_lines, local_files)
      disk_preamble = preamble_lines(on_disk_lines)
      fresh_preamble = preamble_lines(fresh_lines)
      diff = first_line_diff(disk_preamble, fresh_preamble)
      return nil unless diff

      index, disk_line, fresh_line = diff
      target_line = index + 1
      if (local = locate_line_in_locals(local_files, fresh_line))
        return "`#{target_path}:#{target_line}` unmarked text differs from `#{local[0]}:#{local[1]}`"
      end
      if (local = locate_line_in_locals(local_files, disk_line))
        return "`#{target_path}:#{target_line}` unmarked text differs from `#{local[0]}:#{local[1]}`"
      end

      "`#{target_path}:#{target_line}` unmarked text differs from fresh composition"
    end

    def preamble_lines(lines)
      preamble = []
      lines.each do |line|
        break if managed_marker?(line)

        preamble << line
      end
      preamble
    end

    def managed_marker?(line)
      trimmed = line.strip
      return false unless trimmed.start_with?("<!-- pray:") && trimmed.end_with?(" -->")

      id = trimmed.delete_prefix("<!-- pray:").delete_suffix(" -->")
      id != "0 ignore-comments" && id.match?(/\A[a-z0-9]+\z/)
    end

    def first_line_diff(left, right)
      shared = [left.length, right.length].min
      shared.times do |index|
        return [index, left[index], right[index]] if left[index] != right[index]
      end
      return nil if left.length == right.length

      index = shared
      [index, left[index] || "", right[index] || ""]
    end

    def locate_line_in_locals(local_files, line)
      return nil if line.nil? || line.empty?

      local_files.each do |local|
        local.content.each_line(chomp: true).with_index do |candidate, index|
          return [local.manifest_path, index + 1] if candidate == line
        end
      end
      nil
    end
  end
end
