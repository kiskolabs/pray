# frozen_string_literal: true

module Pray
  module TransactionRecords
    module_function

    def read(path)
      bytes = RenderDest.open_regular(path, "project recovery journal", File::RDWR) do |file|
        raise Error.render("project recovery journal exceeds its 96 MiB limit") if file.size > TransactionJournal::MAX_LOG

        contents = file.read(TransactionJournal::MAX_LOG + 1) || ""
        raise Error.render("project recovery journal exceeds its 96 MiB limit") if contents.bytesize > TransactionJournal::MAX_LOG

        complete = (contents.rindex("\n") || -1) + 1
        file.truncate(complete)
        file.fsync
        contents.byteslice(0, complete)
      end
      parse(bytes)
    end

    def parse(bytes)
      entries = []
      undone = {}
      committed = false
      bytes.lines.each_with_index do |line, index|
        record = JSON.parse(line)
        case record["type"]
        when "start"
          invalid! unless record["version"] == 1 && index == 0
        when "write"
          invalid! unless index > 0 && !committed && undone.empty?
          validate_entry(record["entry"], entries.length)
          entries << record["entry"]
        when "undone"
          invalid! unless !committed && record["index"].is_a?(Integer) && (0...entries.length).cover?(record["index"])
          undone[record["index"]] = true
        when "commit"
          invalid! unless index > 0 && undone.empty? && !committed
          committed = true
        else
          invalid!
        end
      end
      [entries, undone, committed]
    rescue JSON::ParserError
      invalid!
    end

    def validate_entry(entry, count)
      invalid! if count >= 10_000
      invalid! unless entry.is_a?(Hash) && entry["path"].is_a?(String)
      invalid! unless entry["mode"].is_a?(Integer) && (0..0o777).cover?(entry["mode"])
      invalid! unless entry["before"].nil? || entry["before"].is_a?(String)
      if entry["after_hash"]
        invalid! unless entry["after_hash"].is_a?(String) && entry["after_hash"].match?(/\Asha256:[a-f0-9]{64}\z/)
      end
      PathSafety.validate_destination_path!(entry["path"])
    end

    def invalid!
      raise Error.render("invalid project recovery journal state")
    end
  end
end
