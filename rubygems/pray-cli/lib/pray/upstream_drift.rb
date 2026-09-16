# frozen_string_literal: true

module Pray
  module Upstream
    module_function

    def path_fork_drift_lines(project, previous, options)
      user_config = Config.load_user_config
      git_sources = GitSources.prepare_git_sources(
        project.project_root,
        project.manifest.sources,
        previous,
        refresh: options.refresh || options.refresh_source_revisions
      )
      sources = Resolve.source_map(project.manifest.sources)
      lines = []
      project.packages.each do |package|
        next unless package.declaration.path
        next unless package.upstream

        upstream = package.upstream
        resolved = resolve_named(
          project.project_root, sources, git_sources, user_config, previous, options,
          upstream.name, "= #{upstream.version}", upstream.source
        )
        upstream_content = content_file_bytes(resolved.root, resolved.spec)
        local_content = local_content_for_refresh(package.root, package.spec)
        if local_content.empty?
          lines << "#{package.declaration.name} has no content files; run pray install to copy #{upstream.name} #{upstream.version}"
          next
        end

        overlay_file_changes(local_content, upstream_content).each do |path, change|
          lines << overlay_drift_line(
            package.declaration.name, upstream.name, upstream.version, path, change
          )
        end
      end
      lines
    end
  end
end
