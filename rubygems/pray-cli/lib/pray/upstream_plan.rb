# frozen_string_literal: true

module Pray
  module Upstream
    PathUpstreamConflict = Struct.new(
      :package_name,
      :upstream_name,
      :old_version,
      :new_version,
      :paths,
      keyword_init: true
    )

    module_function

    def plan_path_upstream_conflicts(project, previous, selected, options)
      user_config = Config.load_user_config
      git_sources = GitSources.prepare_git_sources(
        project.project_root,
        project.manifest.sources,
        previous,
        refresh: options.refresh || options.refresh_source_revisions,
        offline: options.offline
      )
      sources = Resolve.source_map(project.manifest.sources)
      conflicts = []
      project.packages.each do |package|
        next if selected && package.declaration.name != selected

        conflict = plan_one_path_upstream_conflict(
          project, package, previous, sources, git_sources, user_config, options
        )
        conflicts << conflict if conflict
      end
      conflicts
    end

    def report_path_upstream_would_conflicts(project, previous, selected, options)
      conflicts = plan_path_upstream_conflicts(project, previous, selected, options)
      return if conflicts.empty?

      messages = conflicts.map do |conflict|
        puts "would-conflict #{conflict.package_name}: #{conflict.paths.join(", ")}"
        merge_conflict_message(
          conflict.package_name,
          conflict.upstream_name,
          conflict.old_version,
          conflict.new_version,
          conflict.paths
        )
      end
      raise Error.resolution(messages.join("\n"))
    end

    def plan_one_path_upstream_conflict(project, package, previous, sources, git_sources, user_config, options)
      new_upstream = package.upstream
      return nil unless new_upstream && package.declaration.path

      local_content = local_content_for_refresh(package.root, package.spec)
      return nil if local_content.empty?

      old_upstream = previous&.package&.find { |entry| entry.name == package.declaration.name }&.upstream
      return nil unless old_upstream
      return nil if old_upstream.version == new_upstream.version && old_upstream.tree_hash == new_upstream.tree_hash

      old_package, new_package = resolve_upstream_pair(
        project, previous, sources, git_sources, user_config, options, old_upstream, new_upstream
      )
      old_content = content_file_bytes(old_package.root, old_package.spec)
      new_content = content_file_bytes(new_package.root, new_package.spec)
      _merged, conflict_paths = try_merge_content_files(old_content, new_content, local_content)
      return nil if conflict_paths.empty?

      PathUpstreamConflict.new(
        package_name: package.declaration.name,
        upstream_name: old_upstream.name,
        old_version: old_upstream.version,
        new_version: new_upstream.version,
        paths: conflict_paths
      )
    end
    private_class_method :plan_one_path_upstream_conflict
  end
end
