# frozen_string_literal: true

require "fileutils"
require "pathname"

module Pray
  module RenderDest
    module_function

    def materialize(project, previous_lockfile = nil)
      planned = Render.planned_provisioned_files(project)
      previous = previous_map(previous_lockfile)
      planned_paths = planned.map(&:path).to_h { |path| [path, true] }
      planned.each { |file| write_leaf(project, file, previous) }
      prune_dropped(project, previous_lockfile, planned_paths) if previous_lockfile
    end

    def provisioned_records(project)
      symbols = project.manifest.symbols || {}
      Render.planned_provisioned_files(project).map do |file|
        expected = Render.expected_provisioned_bytes(file.source, symbols)
        ProvisionedFileRecord.new(
          path: file.path.to_s.tr("\\", "/"),
          content_hash: Hashing.sha256_prefixed(expected.b),
          package: file.package,
          export: file.export
        )
      end
    end

    def fail_if_symlink!(path, display)
      return unless File.symlink?(path)

      raise Error.render("refusing to write `#{display}` because it is a symbolic link")
    end

    def previous_map(lockfile)
      return {} unless lockfile

      Array(lockfile.provisioned).to_h { |record| [record.path, record] }
    end

    def destination_status(project, file, previous_lockfile = nil)
      PathSafety.validate_destination_path!(file.path)
      ensure_safe_destination_ancestors!(project.project_root, file.path, file.path)
      destination = File.join(project.project_root, file.path)
      expected = Render.expected_provisioned_bytes(file.source, project.manifest.symbols || {})
      record = previous_map(previous_lockfile)[file.path.to_s.tr("\\", "/")]
      classify_destination(destination, file.path, expected, record)
    end

    def write_leaf(project, file, previous)
      PathSafety.validate_destination_path!(file.path)
      ensure_safe_destination_ancestors!(project.project_root, file.path, file.path)
      destination = File.join(project.project_root, file.path)
      expected = Render.expected_provisioned_bytes(file.source, project.manifest.symbols || {})
      record = previous[file.path.to_s.tr("\\", "/")]
      status = classify_destination(destination, file.path, expected, record)
      if status == :write
        FileUtils.mkdir_p(File.dirname(destination))
        ensure_safe_destination_ancestors!(project.project_root, file.path, file.path)
        return create_bytes(destination, file.path, expected)
      end
      return if status == :unchanged

      unless record
        raise Error.render("missing lock ownership for `#{file.path}`")
      end

      ensure_safe_destination_ancestors!(project.project_root, file.path, file.path)
      update_bytes(destination, file.path, expected, record.content_hash)
    end

    def classify_destination(destination, display, expected, record)
      kind = destination_kind(destination)
      return :write if kind == :missing

      fail_if_symlink!(destination, display)
      unless kind == :regular
        raise Error.render("refusing to write `#{display}`; destination is not a regular file")
      end
      on_disk = read_regular_bytes(destination, display)
      return :unchanged if on_disk.b == expected.b
      return :update if record && Hashing.sha256_prefixed(on_disk) == record.content_hash

      if record
        raise Error.render("refusing to overwrite `#{display}`; it was provisioned and then edited")
      end

      raise Error.render(
        "refusing to overwrite `#{display}`; it already exists and is not the expected provisioned file"
      )
    end

    def prune_dropped(project, previous, planned_paths)
      Array(previous.provisioned).each do |record|
        PathSafety.validate_destination_path!(record.path)
        next if planned_paths[record.path]

        destination = File.join(project.project_root, record.path)
        ensure_safe_destination_ancestors!(project.project_root, record.path, record.path)
        next unless destination_kind(destination) == :regular

        on_disk = read_regular_bytes(destination, record.path)
        if Hashing.sha256_prefixed(on_disk) == record.content_hash
          ensure_safe_destination_ancestors!(project.project_root, record.path, record.path)
          File.delete(destination)
        end
      end
    end

    def layout_rendered_content(path, display, fresh)
      kind = destination_kind(path)
      return fresh if kind == :missing

      fail_if_symlink!(path, display)
      unless kind == :regular
        raise Error.render("refusing to write `#{display}`; destination is not a regular file")
      end
      RenderPatch.patch_rendered_content(decode_utf8(read_regular_bytes(path, display), display), fresh)
    end

    def write_rendered_content(path, display, fresh)
      return create_bytes(path, display, fresh) if destination_kind(path) == :missing

      open_regular(path, display, File::RDWR) do |file|
        existing = decode_utf8(file.read, display)
        content = RenderPatch.patch_rendered_content(existing, fresh)
        file.rewind
        file.truncate(0)
        file.write(content)
      end
    end

    def ensure_safe_destination_ancestors!(project_root, relative_path, display)
      parent = File.dirname(relative_path.to_s.tr("\\", "/"))
      return if parent == "."

      current = project_root
      parent.split("/").each do |component|
        next if component.empty? || component == "."

        current = File.join(current, component)
        metadata = File.lstat(current)
        if metadata.symlink?
          raise Error.render(
            "refusing to write `#{display}` because a destination parent is a symbolic link"
          )
        end
        unless metadata.directory?
          raise Error.render(
            "refusing to write `#{display}`; a destination parent is not a directory"
          )
        end
      rescue Errno::ENOENT
        next
      end
    end

    def create_bytes(path, display, bytes)
      open_path(path, display, File::WRONLY | File::CREAT | File::EXCL) do |file|
        file.write(bytes)
      end
    end

    def update_bytes(path, display, bytes, authorized_hash)
      open_regular(path, display, File::RDWR) do |file|
        on_disk = file.read
        return if on_disk.b == bytes.b

        unless Hashing.sha256_prefixed(on_disk) == authorized_hash
          raise Error.render("refusing to overwrite `#{display}`; it was provisioned and then edited")
        end
        file.rewind
        file.truncate(0)
        file.write(bytes)
      end
    end

    def read_regular_bytes(path, display)
      open_regular(path, display, File::RDONLY) { |file| file.read }
    end

    def decode_utf8(bytes, display)
      text = bytes.dup.force_encoding(Encoding::UTF_8)
      return text if text.valid_encoding?

      raise Error.render("rendered destination `#{display}` is not valid UTF-8")
    end

    def open_regular(path, display, flags)
      open_path(path, display, flags) do |file|
        unless file.stat.file?
          raise Error.render("refusing to write `#{display}`; destination is not a regular file")
        end
        yield file
      end
    end

    def open_path(path, display, flags)
      no_follow = File.const_defined?(:NOFOLLOW) ? File::NOFOLLOW : 0
      File.open(path, flags | no_follow) { |file| yield file }
    rescue Errno::ELOOP
      raise Error.render("refusing to write `#{display}` because it is a symbolic link")
    end

    def destination_kind(path)
      return :symlink if File.symlink?(path)
      return :missing unless File.exist?(path)
      return :regular if File.file?(path)

      :other
    end
  end
end
