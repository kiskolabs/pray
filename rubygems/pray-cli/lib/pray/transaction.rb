# frozen_string_literal: true

require_relative "transaction/owner"
require_relative "transaction/journal"
require_relative "transaction/records"

module Pray
  module Transaction
    module_function

    def run(root)
      root = File.expand_path(root)
      current = Thread.current[:pray_write_transaction]
      return yield if current && current.root == root
      raise Error.render("a command cannot write two projects in one transaction") if current

      RenderDest.ensure_safe_destination_ancestors!(root, ".pray/write-state/owner", "project write state")
      directory = File.join(root, ".pray/write-state")
      FileUtils.mkdir_p(directory, mode: 0o700)
      RenderDest.ensure_safe_destination_ancestors!(root, ".pray/write-state/owner", "project write state")
      File.open(directory, File::RDONLY | File::NOFOLLOW) { |file| file.chmod(0o700) }
      [directory, File.join(root, ".pray"), root].each { |path| TransactionOwner.sync_directory(path) }
      release = TransactionOwner.acquire(directory)
      begin
        journal = TransactionJournal.new(root, directory)
        journal.recover
        Thread.current[:pray_write_transaction] = journal
        begin
          result = yield
        rescue => error
          begin
            journal.recover
          rescue => recovery
            raise Error.render("#{error}\nRecovery stopped: #{recovery}")
          end
          raise
        end
        journal.commit
        result
      ensure
        Thread.current[:pray_write_transaction] = nil
        release.call
      end
    end

    def replace(path, before, after)
      journal = Thread.current[:pray_write_transaction]
      return false unless journal

      journal.replace(path, before, after)
      true
    end

    def write_file(path, bytes)
      journal = Thread.current[:pray_write_transaction]
      return File.binwrite(path, bytes) unless journal

      journal.replace(path, TransactionJournal.snapshot(path), bytes.b)
    end
  end
end
