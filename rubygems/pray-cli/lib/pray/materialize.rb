# frozen_string_literal: true

module Pray
  module Materialize
    module_function

    def materialize_project(
      manifest_path: nil,
      frozen: false,
      locked: false,
      offline: false,
      refresh: false
    )
      context = Invocation.invocation_context
      manifest_path = File.expand_path(manifest_path || context.manifest_path)
      unless File.exist?(manifest_path)
        raise Error.manifest("missing #{manifest_path}")
      end

      project_root = context.project_root
      options = ResolveOptions.new(
        offline: offline,
        refresh: refresh,
        environment: context.environment
      )
      allow_git_refresh_fallback = !locked && !frozen
      project = begin
        Resolve.resolve_project_in_context(manifest_path, project_root, options)
      rescue Error => error
        if allow_git_refresh_fallback &&
            !offline &&
            !refresh &&
            Resolve.resolution_may_benefit_from_git_source_refresh?(error)
          refreshed = options.dup
          refreshed.refresh = true
          Resolve.resolve_project_in_context(manifest_path, project_root, refreshed)
        else
          raise
        end
      end
      rendered = Render.render_project(project)
      lockfile_path = default_lockfile_path(project.project_root)
      next_lockfile = LockfileIO.build_lockfile(
        project.manifest_hash,
        project.environment,
        project.project_root,
        project.manifest.sources,
        project.manifest.targets,
        rendered,
        project.packages,
        project.source_revisions,
        project.source_host_keys
      )

      if locked
        unless File.exist?(lockfile_path)
          raise Error.verify("missing Prayfile.lock; run install first")
        end
        existing = Pray.read_lockfile(lockfile_path)
        unless Pray.lockfiles_equivalent?(existing, next_lockfile)
          raise Error.verify("lockfile needs update; rerun install to refresh Prayfile.lock")
        end
        unless frozen
          Render.write_rendered_targets(project, rendered)
        end
        if frozen
          rendered.each do |target|
            path = File.join(project.project_root, target.path)
            on_disk = File.read(path)
            if on_disk != target.content
              raise Error.render(
                "#{path} is stale; rerun install to regenerate it or plan to inspect the diff"
              )
            end
          end
        end
        return
      end

      if frozen
        existing = File.exist?(lockfile_path) ? Pray.read_lockfile(lockfile_path) : nil
        if existing
          rendered.each do |target|
            output_path = File.join(project.project_root, target.path)
            unless File.exist?(output_path)
              raise Error.verify("Rendered file #{target.path} is missing under frozen mode")
            end
          end
        end
      end

      Pray.write_lockfile_if_changed(lockfile_path, next_lockfile)
      Render.write_rendered_targets(project, rendered)
    end

    def default_manifest_path(working_directory = Dir.pwd)
      File.join(working_directory, "Prayfile")
    end

    def default_lockfile_path(project_root)
      File.join(project_root, "Prayfile.lock")
    end

    def project_root_from_manifest(manifest_path)
      Resolve.project_root_from_manifest(manifest_path)
    end
  end

  module_function

  def materialize_project(...) = Materialize.materialize_project(...)
  def default_manifest_path(...) = Materialize.default_manifest_path(...)
  def default_lockfile_path(...) = Materialize.default_lockfile_path(...)
  def project_root_from_manifest(...) = Materialize.project_root_from_manifest(...)
  def resolve_project(...) = Resolve.resolve_project(...)
  def render_project(...) = Render.render_project(...)
  def inspect_project(...) = Verify.inspect_project(...)
  def verify_project(...) = Verify.verify_project(...)
  def drift_project(...) = Verify.drift_project(...)
end
