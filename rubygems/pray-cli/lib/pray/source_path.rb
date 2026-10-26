# frozen_string_literal: true

require "pathname"

module Pray
  module SourcePath
    module_function

    def declared_directory(root, relative, label)
      reject_symlinks!(root, relative, label)
      File.expand_path(relative, root)
    end

    def reject_symlinks!(root, relative, label)
      current = Pathname(relative).absolute? ? File::SEPARATOR : root
      status = nil
      Pathname(relative).each_filename do |component|
        current = File.join(current, component)
        status = File.lstat(current)
        raise Error.integrity("#{label} uses a symbolic link: #{relative}") if status.symlink?
      rescue Errno::ENOENT
        return nil
      end
      status
    end

    def read(root, relative, label)
      status = reject_symlinks!(root, relative, label)
      raise Error.integrity("#{label} missing: #{relative}") unless status
      raise Error.integrity("#{label} is a directory: #{relative}") if status.directory?

      File.binread(File.join(root, relative))
    end
  end
end
