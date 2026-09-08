# frozen_string_literal: true

module Pray
  module Upstream
    module_function

    def ensure_update_supported!(packages, selected = nil)
      unsupported = packages.find do |package|
        package.declaration.path && package.spec.upstream &&
          (selected.nil? || package.declaration.name == selected)
      end
      return unless unsupported

      raise Error.unsupported(
        "this installation cannot refresh upstream package #{unsupported.declaration.name}; " \
        "install pray with Cargo and retry"
      )
    end

    def to_lock_hash(entry)
      hash = {
        "name" => entry.name,
        "version" => entry.version,
        "tree_hash" => entry.tree_hash,
        "artifact_hash" => entry.artifact_hash
      }
      hash["source"] = entry.source unless entry.source.nil?
      hash
    end

    def from_lock_hash(entry)
      return nil if entry.nil?

      LockedUpstream.new(
        name: entry["name"],
        version: entry["version"],
        source: entry["source"],
        tree_hash: entry["tree_hash"],
        artifact_hash: entry["artifact_hash"]
      )
    end

    def lock_path(project_root, sources, git_sources, user_config, declaration, spec, lockfile, options)
      return nil unless spec.upstream
      if spec.name == spec.upstream.name
        raise Error.resolution("package #{spec.name} cannot use itself as upstream")
      end
      return nil unless declaration.path

      refresh = options.ignore_locked_versions || options.unlocked_packages.include?(declaration.name)
      locked = lockfile&.package&.find { |entry| entry.name == declaration.name }&.upstream
      name, constraint, source = resolution_input(declaration, spec, sources, locked, refresh)
      resolved = Resolve.resolve_package(
        project_root,
        sources,
        git_sources,
        user_config,
        ManifestPackage.new(
          name: name,
          constraint: constraint,
          source: source
        ),
        lockfile,
        options: options
      )
      resolved_upstream = LockedUpstream.new(
        name: name,
        version: resolved.spec.version,
        source: source,
        tree_hash: resolved.tree_hash,
        artifact_hash: resolved.artifact_hash
      )
      ensure_locked_match!(locked, resolved_upstream) if !refresh && locked
      resolved_upstream
    end

    def resolution_input(declaration, spec, sources, locked, refresh)
      if locked && !refresh
        if locked.name != spec.upstream.name
          raise Error.integrity(
            "locked upstream name mismatch for #{declaration.name}: " \
            "expected #{locked.name}, found #{spec.upstream.name}"
          )
        end
        return [locked.name, "= #{locked.version}", locked.source]
      end

      name = spec.upstream.name
      source = ResolveSource.implied_source_name(ManifestPackage.new(name: name), sources)
      [name, spec.upstream.constraint, source]
    end

    def ensure_locked_match!(locked, resolved)
      if locked.name != resolved.name || locked.version != resolved.version
        raise Error.integrity("locked upstream identity mismatch")
      end
      raise Error.integrity("locked upstream source mismatch") if locked.source != resolved.source
      raise Error.integrity("locked upstream tree hash mismatch") if locked.tree_hash != resolved.tree_hash
      if locked.artifact_hash != resolved.artifact_hash
        raise Error.integrity("locked upstream artifact hash mismatch")
      end
    end
  end
end
