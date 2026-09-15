# frozen_string_literal: true

module Pray
  module CLI
    def update_latest_command(package, dry_run:, offline:)
      path = manifest_path
      original_text = Pray.read_manifest_text(path)
      manifest_text = original_text
      preview_options = ResolveOptions.new(
        offline: offline,
        refresh: true,
        refresh_source_revisions: true,
        ignore_locked_versions: true
      )
      project = resolve_current_project(preview_options)
      if package && project.manifest.packages.none? { |entry| entry.name == package }
        raise Error.manifest("package #{package} not found")
      end

      manifest_updates = []
      project.packages.each do |resolved|
        next if package && resolved.declaration.name != package
        next unless resolved.registry_latest_version
        next if Constraint.version_satisfies(resolved.registry_latest_version, resolved.declaration.constraint)

        new_constraint = Constraint.latest_constraint_for_package(
          resolved.declaration.constraint,
          resolved.registry_latest_version
        )
        unless Constraint.version_satisfies(resolved.registry_latest_version, new_constraint)
          raise Error.resolution(
            "derived constraint #{new_constraint} does not admit registry latest #{resolved.registry_latest_version} for #{resolved.declaration.name}"
          )
        end

        manifest_updates << {
          name: resolved.declaration.name,
          from_constraint: resolved.declaration.constraint,
          to_constraint: new_constraint,
          registry_latest_version: resolved.registry_latest_version
        }
        updated = resolved.declaration.dup
        updated.constraint = new_constraint
        manifest_text = Pray.replace_package_declaration(manifest_text, updated)
      end

      previous = File.exist?(lockfile_path) ? Pray.read_lockfile(lockfile_path) : nil
      upstream_plans = Upstream.plan_path_upstream_latest_constraints(
        project, previous, package, preview_options
      )
      print_latest_constraint_plans(manifest_updates, upstream_plans)

      return if dry_run

      Upstream.apply_path_upstream_latest_constraints(upstream_plans)
      Transaction.write_file(path, manifest_text) if manifest_text != original_text
      unlocked = package ? Set[package] : Set.new
      options = ResolveOptions.new(
        offline: offline,
        refresh: true,
        refresh_source_revisions: true,
        ignore_locked_versions: package.nil?,
        unlocked_packages: unlocked
      )
      current = resolve_current_project(options)
      Upstream.apply_path_upstream_refreshes(current, previous, package, options)
      install_command(
        {
          locked: false,
          frozen: false,
          offline: offline,
          refresh: true,
          ignore_locked_versions: package.nil?,
          unlocked_packages: unlocked
        }
      )
    end

    def print_latest_constraint_plans(manifest_updates, upstream_plans)
      if manifest_updates.empty? && upstream_plans.empty?
        puts "All package constraints already allow latest versions"
        return
      end

      manifest_updates.each do |update|
        puts "Prayfile: #{update[:name]} constraint #{update[:from_constraint]} -> #{update[:to_constraint]} " \
          "(registry latest #{update[:registry_latest_version]})"
      end
      upstream_plans.each do |plan|
        puts "#{plan.package_name} upstream #{plan.current_constraint} -> #{plan.new_constraint} (latest #{plan.latest_version})"
      end
    end
  end
end
