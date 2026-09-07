# frozen_string_literal: true

require "base64"
require "pathname"

module Pray
  class TransactionJournal
    MAX_LOG = 96 * 1024 * 1024
    MAX_SAVED = 64 * 1024 * 1024
    MAX_FILE = 32 * 1024 * 1024
    attr_reader :root

    def initialize(root, directory)
      @root = root
      @directory = directory
      @path = File.join(directory, "journal")
      @saved = 0
      @entries = 0
    end

    def append(record)
      bytes = JSON.generate(record) + "\n"
      RenderDest.open_regular(@path, "project recovery journal", File::WRONLY | File::APPEND) do |file|
        raise Error.render("project recovery journal exceeds its 96 MiB limit") if file.size + bytes.bytesize > MAX_LOG

        file.write(bytes)
        file.fsync
      end
    end

    def replace(path, before, after)
      return if before == after

      display = Pathname.new(File.expand_path(path)).relative_path_from(Pathname.new(root)).to_s.tr("\\", "/")
      destination(display)
      validate_budget(before, after)
      raise changed(display) if self.class.snapshot(path) != before

      mode = before.nil? ? 0o644 : File.lstat(path).mode & 0o777
      unless File.exist?(@path)
        TransactionOwner.private_file(@path, &:fsync)
        append({"type" => "start", "version" => 1})
        TransactionOwner.sync_directory(@directory)
      end
      append({"type" => "write", "entry" => {"path" => display,
                                             "before" => before.nil? ? nil : Base64.strict_encode64(before),
                                             "after_hash" => after.nil? ? nil : Hashing.sha256_prefixed(after), "mode" => mode}})
      @saved += (before&.bytesize || 0) + (after&.bytesize || 0)
      @entries += 1
      install(display, before, after, mode)
    end

    def validate_budget(before, after)
      if [before, after].compact.any? { |bytes| bytes.bytesize > MAX_FILE }
        raise Error.render("destination exceeds the 32 MiB limit")
      end
      if @saved + (before&.bytesize || 0) + (after&.bytesize || 0) > MAX_SAVED || @entries == 10_000
        raise Error.render("project write exceeds the 64 MiB or 10000-file recovery limit")
      end
    end

    def destination(display)
      PathSafety.validate_destination_path!(display)
      if display == ".pray/write-state" || display.start_with?(".pray/write-state/")
        raise Error.render("destination overlaps project recovery state")
      end
      RenderDest.ensure_safe_destination_ancestors!(root, display, display)
      File.join(root, display)
    end

    def install(display, expected, bytes, mode)
      path = destination(display)
      FileUtils.mkdir_p(File.dirname(path))
      destination(display)
      temporary = File.join(@directory, "#{SecureRandom.hex(16)}.stage")
      unless bytes.nil?
        TransactionOwner.private_file(temporary) do |file|
          file.write(bytes)
          file.chmod(mode & 0o777)
          file.fsync
        end
      end
      raise changed(display) if self.class.snapshot(path) != expected

      destination(display)
      if !bytes.nil? && expected.nil?
        File.link(temporary, path)
        File.unlink(temporary)
      elsif !bytes.nil?
        File.rename(temporary, path)
      elsif !expected.nil?
        File.unlink(path)
      end
      sync_parents(path)
    end

    def sync_parents(path)
      parent = File.dirname(path)
      loop do
        begin
          TransactionOwner.sync_directory(parent)
        rescue Errno::ENOENT
          nil
        end
        break if parent == root

        parent = File.dirname(parent)
      end
    end

    def recover
      clean_stages
      return unless File.exist?(@path)

      entries, undone, committed = TransactionRecords.read(@path)
      unless committed
        (entries.length - 1).downto(0) do |index|
          next if undone.include?(index)

          restore(entries[index])
          append({"type" => "undone", "index" => index})
        end
      end
      File.unlink(@path)
      TransactionOwner.sync_directory(@directory)
      @saved = @entries = 0
      clean_stages
    rescue ArgumentError
      raise Error.render("invalid recovery bytes")
    end

    def restore(entry)
      before = entry["before"].nil? ? nil : Base64.strict_decode64(entry["before"])
      raise Error.render("recovery file exceeds 32 MiB") if before && before.bytesize > MAX_FILE

      current = self.class.snapshot(destination(entry["path"]))
      if current != before
        current_hash = current.nil? ? nil : Hashing.sha256_prefixed(current)
        raise changed(entry["path"]) if !current.nil? && current_hash != entry["after_hash"]

        install(entry["path"], current, before, entry["mode"])
      end
      # A prior recovery may have stopped before its directory sync completed.
      sync_parents(destination(entry["path"]))
    end

    def commit
      if File.exist?(@path)
        append({"type" => "commit"})
        File.unlink(@path)
        TransactionOwner.sync_directory(@directory)
      end
      clean_stages
    end

    def clean_stages
      Dir.children(@directory).each do |name|
        File.unlink(File.join(@directory, name)) if name.match?(/\A[a-f0-9]{32}\.stage\z/)
      end
    end

    def self.snapshot(path)
      (RenderDest.destination_kind(path) == :missing) ? nil : RenderDest.read_regular_bytes(path, path)
    end

    def changed(display)
      Error.render("`#{display}` changed during the interrupted write. Inspect your changes and move the file aside, then retry; recovery data remains in .pray/write-state")
    end
  end
end
