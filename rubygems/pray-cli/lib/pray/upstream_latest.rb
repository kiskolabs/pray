# frozen_string_literal: true

module Pray
  module Upstream
    module_function

    PathUpstreamLatestConstraint = Struct.new(
      :package_name, :upstream_name, :current_constraint, :latest_version, :new_constraint, :package_root,
      keyword_init: true
    )

    def plan_path_upstream_latest_constraints(project, previous, selected, options)
      user_config = Config.load_user_config
      git_sources = GitSources.prepare_git_sources(
        project.project_root,
        project.manifest.sources,
        previous,
        refresh: options.refresh || options.refresh_source_revisions,
        offline: options.offline
      )
      sources = Resolve.source_map(project.manifest.sources)
      plans = []
      project.packages.each do |package|
        next if selected && package.declaration.name != selected

        plan = plan_one_path_upstream(project, package, sources, git_sources, user_config, previous, options)
        plans << plan if plan
      end
      plans
    end

    def apply_path_upstream_latest_constraints(plans)
      plans.each do |plan|
        spec_path = Resolve.find_prayspec_file(plan.package_root)
        spec = Pray.parse_package_spec(File.read(spec_path))
        unless spec.upstream
          raise Error.resolution("package #{plan.package_name} has no upstream pin")
        end

        spec.upstream = PackageUpstream.new(
          name: spec.upstream.name,
          constraint: plan.new_constraint
        )
        Transaction.write_file(spec_path, PackageSpecRender.render_package_spec(spec))
      end
    end

    def latest_spec_upstream_constraint(current, latest_version)
      derived = Constraint.latest_constraint_for_package(current, latest_version)
      return "= #{latest_version}" if derived.start_with?("=")

      derived
    end

    def plan_one_path_upstream(project, package, sources, git_sources, user_config, lockfile, options)
      upstream = package.spec.upstream
      return unless upstream && package.declaration.path

      source = ResolveSource.implied_source_name(ManifestPackage.new(name: upstream.name), sources)
      resolved = Resolve.resolve_package(
        project.project_root,
        sources,
        git_sources,
        user_config,
        ManifestPackage.new(name: upstream.name, constraint: "*", source: source),
        lockfile,
        options: options
      )
      return if Constraint.version_satisfies(resolved.spec.version, upstream.constraint)

      new_constraint = latest_spec_upstream_constraint(upstream.constraint, resolved.spec.version)
      unless Constraint.version_satisfies(resolved.spec.version, new_constraint)
        raise Error.resolution(
          "derived constraint #{new_constraint} does not admit latest #{resolved.spec.version} for #{package.declaration.name}"
        )
      end

      PathUpstreamLatestConstraint.new(
        package_name: package.declaration.name,
        upstream_name: upstream.name,
        current_constraint: upstream.constraint,
        latest_version: resolved.spec.version,
        new_constraint: new_constraint,
        package_root: package.root
      )
    end
    private_class_method :plan_one_path_upstream
  end
end
