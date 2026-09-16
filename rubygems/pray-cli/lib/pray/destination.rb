# frozen_string_literal: true

module Pray
  DestinationEntry = Struct.new(:kind, :name, :path) do
    def self.package(name)
      new(kind: "package", name: name, path: nil)
    end

    def self.local(path)
      new(kind: "local", name: nil, path: path)
    end
  end

  module Destination
    module_function

    def local_path_form?(value)
      value.start_with?(".", "/") ||
        value.end_with?(".md", ".txt", ".markdown")
    end

    def destination_target_name(mode, path)
      prefix = case mode
      when "compose" then "compose"
      when "tree" then "tree"
      else "legacy"
      end
      "#{prefix}:#{path}"
    end

    def new_destination_target(mode, path)
      target = ManifestTarget.new(
        name: destination_target_name(mode, path),
        mode: mode,
        scoped: true
      )
      case mode
      when "compose" then target.outputs << path
      when "tree" then target.skills << path
      end
      target
    end

    def upsert_package(manifest, package)
      existing = manifest.packages.find { |candidate| candidate.name == package.name }
      unless existing
        manifest.packages << package
        return
      end

      if existing.constraint != package.constraint &&
          existing.constraint != "*" &&
          package.constraint != "*"
        raise Error.manifest(
          "package #{package.name} declared with conflicting constraints " \
          "(#{existing.constraint} vs #{package.constraint})"
        )
      end
      existing.constraint = package.constraint if existing.constraint == "*" && package.constraint != "*"

      if existing.source.nil?
        existing.source = package.source
      elsif package.source && existing.source != package.source
        raise Error.manifest("package #{package.name} declared with conflicting sources")
      end

      package.exports.each do |export|
        existing.exports << export unless existing.exports.include?(export)
      end
      package.roles.each do |role|
        existing.roles << role unless existing.roles.include?(role)
      end

      if package.file
        if existing.file && existing.file != package.file
          raise Error.manifest(
            "package #{package.name} declared with conflicting file: destinations"
          )
        end
        existing.file = package.file
      end

      existing.bound = existing.bound || package.bound
      existing.optional = existing.optional || package.optional
      existing.path ||= package.path
      existing.git ||= package.git
      existing.tag ||= package.tag
      existing.rev ||= package.rev
      existing.tarball ||= package.tarball
      existing.oci ||= package.oci
      package.groups.each do |group|
        existing.groups << group unless existing.groups.include?(group)
      end
    end

    def upsert_local(manifest, local)
      existing = manifest.local.find { |candidate| candidate.path == local.path }
      unless existing
        manifest.local << local
        return
      end

      existing.bound = existing.bound || local.bound
      existing.optional = existing.optional || local.optional
      if existing.position == "after" && local.position != "after"
        existing.position = local.position
      end
      existing.file ||= local.file
    end

    def bind_package_entry(target, package_name)
      entry = DestinationEntry.package(package_name)
      return if target.entries.any? { |candidate| candidate.kind == "package" && candidate.name == package_name }

      target.entries << entry
    end

    def bind_local_entry(target, path)
      return if target.entries.any? { |candidate| candidate.kind == "local" && candidate.path == path }

      target.entries << DestinationEntry.local(path)
    end

    def role_for_destination(mode)
      case mode
      when "compose" then "fragment"
      when "tree" then "folder"
      end
    end

    def package_bound_to_compose?(package, target)
      if target.scoped && target.mode == "compose"
        return target.entries.any? { |entry| entry.kind == "package" && entry.name == package.name }
      end
      return false if package.bound || package.file

      true
    end

    def package_bound_to_tree?(package, target)
      if target.scoped && target.mode == "tree"
        return target.entries.any? { |entry| entry.kind == "package" && entry.name == package.name }
      end
      return false if package.bound || package.file

      true
    end

    def export_kind_matches_role?(kind, role)
      case role
      when "fragment" then kind == "fragment"
      when "folder" then %w[folder skill].include?(kind)
      when "file" then kind == "file"
      else false
      end
    end

    def local_compose_embed?(manifest, local)
      return false if local.file

      manifest.targets.none? do |target|
        target.mode == "tree" &&
          target.entries.any? { |entry| entry.kind == "local" && entry.path == local.path }
      end
    end

    def try_apply_local_pray(manifest, path, file_dest, other_package_signal, destination_index)
      return false unless local_path_form?(path)

      if file_dest
        return false if other_package_signal
        raise Error.parse("manifest", "a local file cannot use file:")
      end
      return false if other_package_signal
      unless destination_index
        raise Error.parse("manifest", "local pray paths are only valid inside compose")
      end
      mode = manifest.targets[destination_index].mode
      unless mode == "compose"
        raise Error.parse("manifest", "local pray paths are only valid inside compose")
      end
      local = ManifestLocal.new(path: path, bound: true)
      bind_local_entry(manifest.targets[destination_index], local.path)
      upsert_local(manifest, local)
      true
    end
  end
end
