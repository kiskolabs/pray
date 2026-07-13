# frozen_string_literal: true

require "fileutils"

module Pray
  module CLI
    def install_command(flags)
      Pray.materialize_project(
        manifest_path: manifest_path,
        frozen: flags[:frozen],
        locked: flags[:locked],
        offline: flags[:offline],
        refresh: flags[:refresh]
      )
    end

    def render_command(flags)
      project = Resolve.resolve_project(manifest_path)
      rendered = Render.render_project(project)
      if flags[:check]
        ensure_rendered_outputs_current(project, rendered)
      else
        Render.write_rendered_targets(project, rendered)
      end
    end

    def verify_command(flags)
      project = Resolve.resolve_project(manifest_path)
      lockfile = ensure_existing_lockfile(File.join(project.project_root, LOCKFILE_PATH))
      report = Verify.verify_project(project, lockfile, strict: flags[:strict])
      puts format_verification_report(report) unless report.clean?
    end

    def drift_command(flags)
      project = Resolve.resolve_project(manifest_path)
      lockfile = ensure_existing_lockfile(File.join(project.project_root, LOCKFILE_PATH))
      report = if flags[:semantic]
                 Verify.inspect_project(project, lockfile)
               else
                 Verify.drift_project(project, lockfile)
               end
      puts format_drift_report(report) unless report.clean?
    end

    def format_command
      lockfile = Pray.read_lockfile(lockfile_path)
      lockfile.target.each do |target|
        target.outputs.each do |output|
          original = File.read(output)
          formatted = format_marker_comments(Hashing.normalize_line_endings(original))
          File.write(output, formatted) if formatted != original
        end
      end
    end

    def plan_command(_arguments)
      project = Resolve.resolve_project(manifest_path)
      rendered = Render.render_project(project)
      lockfile = build_lockfile(project, rendered)
      previous_lockfile = File.exist?(lockfile_path) ? Pray.read_lockfile(lockfile_path) : nil
      preview = Plan.build_materialization_preview(
        project, rendered, lockfile, lockfile_path, previous_lockfile
      )
      Plan.print_materialization_report(preview, :plan)
    end

    def update_command(arguments)
      package = arguments.reject { |argument| argument.start_with?("--") }.first
      offline = arguments.include?("--offline")
      if package
        manifest = Pray.parse_manifest(Pray.read_manifest_text(manifest_path))
        unless manifest.packages.any? { |entry| entry.name == package }
          raise Error.manifest("package #{package} not found")
        end
      end
      install_command({ locked: false, frozen: false, offline: offline, refresh: true })
    end

    def unlock_command(name)
      raise Error.manifest("unlock requires a package name") unless name

      project = Resolve.resolve_project(manifest_path)
      unless project.manifest.packages.any? { |entry| entry.name == name }
        raise Error.manifest("package #{name} not found")
      end

      previous_lockfile = Pray.read_lockfile(lockfile_path)
      rendered = Render.render_project(project)
      updated_lockfile = build_lockfile(project, rendered)
      merged_lockfile = merge_selected_package_update(previous_lockfile, updated_lockfile, name)
      Pray.write_lockfile(lockfile_path, merged_lockfile)
      Render.write_rendered_targets(project, rendered)
      puts "Unlocked #{name}"
    end

    def merge_selected_package_update(previous, updated, selected_package)
      merged = updated.dup
      merged.package = updated.package.map do |package|
        next package if package.name == selected_package

        previous_package = previous.package.find { |entry| entry.name == package.name }
        previous_package ? package.dup.tap { |copy| copy.version = previous_package.version } : package
      end
      merged
    end
  end
end
