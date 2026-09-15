# frozen_string_literal: true

module Pray
  module Upstream
    module_function

    def apply_path_upstream_refreshes(project, previous, selected, options)
      user_config = Config.load_user_config
      git_sources = GitSources.prepare_git_sources(
        project.project_root,
        project.manifest.sources,
        previous,
        refresh: options.refresh || options.refresh_source_revisions
      )
      sources = Resolve.source_map(project.manifest.sources)
      changed = false
      project.packages.each do |package|
        next if selected && package.declaration.name != selected
        next unless apply_one_path_upstream(project, package, previous, sources, git_sources, user_config, options)

        changed = true
      end
      changed
    end

    def apply_one_path_upstream(project, package, previous, sources, git_sources, user_config, options)
      new_upstream = package.upstream
      return false unless new_upstream && package.declaration.path

      old_upstream = previous&.package&.find { |entry| entry.name == package.declaration.name }&.upstream
      return false unless old_upstream
      return false if old_upstream.version == new_upstream.version && old_upstream.tree_hash == new_upstream.tree_hash

      old_package, new_package = resolve_upstream_pair(
        project, previous, sources, git_sources, user_config, options, old_upstream, new_upstream
      )
      old_content, local_content, merged = merge_fork_content(package, old_upstream, new_upstream, old_package, new_package)
      write_content_files(package.root, old_content, merged)
      write_refreshed_spec(package, new_package, old_content, local_content, merged)
      true
    end

    def resolve_upstream_pair(project, previous, sources, git_sources, user_config, options, old_upstream, new_upstream)
      old_package = resolve_named(
        project.project_root, sources, git_sources, user_config, previous, options,
        old_upstream.name, "= #{old_upstream.version}", old_upstream.source
      )
      resolved_old = LockedUpstream.new(
        name: old_upstream.name,
        version: old_package.spec.version,
        source: old_upstream.source,
        tree_hash: old_package.tree_hash,
        artifact_hash: old_package.artifact_hash
      )
      ensure_locked_match!(old_upstream, resolved_old)
      new_package = resolve_named(
        project.project_root, sources, git_sources, user_config, previous, options,
        new_upstream.name, "= #{new_upstream.version}", new_upstream.source
      )
      [old_package, new_package]
    end

    def merge_fork_content(package, old_upstream, new_upstream, old_package, new_package)
      old_content = content_file_bytes(old_package.root, old_package.spec)
      new_content = content_file_bytes(new_package.root, new_package.spec)
      local_content = content_file_bytes(package.root, package.spec)
      merged, conflicts = try_merge_content_files(old_content, new_content, local_content)
      unless conflicts.empty?
        raise Error.resolution(
          merge_conflict_message(
            package.declaration.name, old_upstream.name, old_upstream.version, new_upstream.version, conflicts
          )
        )
      end
      [old_content, local_content, merged]
    end

    def write_refreshed_spec(package, new_package, old_content, local_content, merged)
      spec_path = Resolve.find_prayspec_file(package.root)
      updated = PackageSpecRender.fork_spec_after_refresh(
        package.spec,
        new_package.spec,
        File.basename(spec_path),
        clean_replica?(old_content, local_content),
        merged.keys
      )
      Transaction.write_file(spec_path, PackageSpecRender.render_package_spec(updated))
    end

    def resolve_named(project_root, sources, git_sources, user_config, lockfile, options, name, constraint, source)
      Resolve.resolve_package(
        project_root,
        sources,
        git_sources,
        user_config,
        ManifestPackage.new(name: name, constraint: constraint, source: source),
        lockfile,
        options: options
      )
    end

    def content_file_bytes(root, spec)
      paths = content_paths(spec.files)
      if paths.length > ResourceLimits::MAX_ARCHIVE_ENTRIES
        raise Error.integrity("package content exceeds #{ResourceLimits::MAX_ARCHIVE_ENTRIES} files")
      end

      files = {}
      total_bytes = 0
      paths.each do |relative|
        PathSafety.validate_archive_member_path!(relative)
        path = File.join(root, relative)
        raise Error.integrity("package file missing: #{relative}") unless File.file?(path)

        size = File.size(path)
        if size > ResourceLimits::MAX_ARCHIVE_ENTRY_BYTES
          raise Error.integrity("package file exceeds #{ResourceLimits::MAX_ARCHIVE_ENTRY_BYTES} bytes: #{relative}")
        end

        total_bytes += size
        if total_bytes > ResourceLimits::MAX_ARCHIVE_TOTAL_BYTES
          raise Error.integrity("package content exceeds #{ResourceLimits::MAX_ARCHIVE_TOTAL_BYTES} bytes")
        end

        files[relative] = File.binread(path)
      end
      files
    end

    def write_content_files(root, old_content, merged)
      (old_content.keys + merged.keys).each { |path| PathSafety.validate_archive_member_path!(path) }
      old_content.each_key do |path|
        next if merged.key?(path)

        Transaction.remove_file(File.join(root, path))
      end
      merged.each do |relative, bytes|
        Transaction.write_file(File.join(root, relative), bytes)
      end
    end
    private_class_method :apply_one_path_upstream, :resolve_upstream_pair, :merge_fork_content,
      :write_refreshed_spec, :resolve_named
  end
end
