# frozen_string_literal: true

require "json"
require "socket"
require "securerandom"

module Pray
  module TransactionOwner
    module_function

    def acquire(directory, path = File.join(directory, "owner"), depth = 0)
      raise Error.render("too many interrupted lock recoveries; inspect .pray/write-state") if depth > 16

      owner = {"version" => 1, "pid" => Process.pid, "host" => Socket.gethostname, "token" => SecureRandom.hex(16)}
      linked = publish_owner(directory, path, owner)
      if linked
        sync_directory(directory)
        return -> { File.unlink(path) if read_owner(path)["token"] == owner["token"] }
      end
      previous = read_owner(path)
      if previous["host"] != Socket.gethostname || alive?(previous["pid"])
        raise Error.render("another pray command owns this project; retry after it finishes")
      end
      # The dead owner's unique token serializes stale-lock removal.
      release = acquire(directory, File.join(directory, "reclaim-#{previous["token"]}"), depth + 1)
      begin
        if read_owner(path)["token"] != previous["token"]
          raise Error.render("project ownership changed; retry the command")
        end
        File.unlink(path)
        acquire(directory, path, depth + 1)
      ensure
        release.call
      end
    end

    def publish_owner(directory, path, owner)
      temporary = File.join(directory, "#{owner["token"]}.owner")
      private_file(temporary) { |file|
        file.write(JSON.generate(owner))
        file.fsync
      }
      begin
        File.link(temporary, path)
        true
      rescue Errno::EEXIST
        false
      ensure
        File.unlink(temporary)
      end
    end

    def read_owner(path)
      bytes = RenderDest.read_regular_bytes(path, "project write owner")
      raise Error.render("invalid project write owner") if bytes.bytesize > 4096

      owner = JSON.parse(bytes)
      unless owner.is_a?(Hash) && owner["version"] == 1 && owner["pid"].is_a?(Integer) && owner["pid"] > 0 &&
          owner["host"].is_a?(String) && owner["token"].is_a?(String) && owner["token"].match?(/\A[a-f0-9]{32}\z/)
        raise Error.render("invalid project write owner")
      end
      owner
    rescue JSON::ParserError
      raise Error.render("invalid project write owner")
    end

    def alive?(pid)
      Process.kill(0, pid)
      true
    rescue Errno::ESRCH
      false
    rescue Errno::EPERM, RangeError
      true
    end

    def private_file(path, &block)
      no_follow = File.const_defined?(:NOFOLLOW) ? File::NOFOLLOW : 0
      File.open(path, File::WRONLY | File::CREAT | File::EXCL | no_follow, 0o600, &block)
    end

    def sync_directory(path)
      return if Gem.win_platform?

      File.open(path, File::RDONLY, &:fsync)
    end
  end
end
