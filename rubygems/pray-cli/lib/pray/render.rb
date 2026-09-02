# frozen_string_literal: true

require "fileutils"
require "pathname"
require_relative "render_content"

module Pray
  RenderedTarget = Struct.new(:path, :content, :managed_spans) do
    def initialize(path:, content:, managed_spans: [])
      super
    end
  end

  module Render
    module_function

    def render_project(project)
      rendered = []
      project.manifest.targets.each do |target|
        output = target.outputs.first
        next unless output
        rendered << render_target(project, target, output)
      end
      rendered
    end

    def write_rendered_targets(project, rendered, previous_lockfile = nil)
      rendered.each do |target|
        PathSafety.validate_destination_path!(target.path)
        RenderDest.ensure_safe_destination_ancestors!(project.project_root, target.path, target.path)
        path = File.join(project.project_root, target.path)
        FileUtils.mkdir_p(File.dirname(path))
        RenderDest.ensure_safe_destination_ancestors!(project.project_root, target.path, target.path)
        RenderDest.write_rendered_content(path, target.path, target.content)
      end
      materialize_provisioned_exports(project, previous_lockfile)
    end

    def materialize_provisioned_exports(project, previous_lockfile = nil)
      RenderDest.materialize(project, previous_lockfile)
    end

    def expected_provisioned_bytes(source, symbols)
      bytes = File.binread(source)
      text = bytes.dup.force_encoding(Encoding::UTF_8)
      if text.valid_encoding?
        Substitute.substitute_pray_symbols(text, symbols)
      else
        bytes
      end
    end

    def planned_provisioned_files(project)
      planned = []
      collect_exact_file_bindings(project, planned)
      project.manifest.targets.each do |target|
        target.skills.each do |folder_root|
          destination_root = File.join(project.project_root, folder_root)
          project.packages.each do |package|
            next unless Environment.package_matches_environment?(package.declaration.groups, project.environment)
            next unless Destination.package_bound_to_tree?(package.declaration, target)

            collect_legacy_skill_files(project, package, destination_root, planned)
            collect_selected_export_files(project, package, destination_root, planned)
          end
        end
      end
      planned.sort_by(&:path).uniq(&:path)
    end

    PlannedProvisionedFile = Struct.new(:path, :source, :package, :export, keyword_init: true)

    def render_target(project, target, output)
      ComposeDest.ensure_html_comment_dest!(output)
      if target.scoped && target.mode == "compose"
        return render_scoped_compose(project, target, output)
      end
      render_legacy_compose(project, target, output)
    end

    def render_scoped_compose(project, target, output)
      builder = ContentBuilder.new
      append_header(builder, project, target, output)
      symbols = project.manifest.symbols || {}
      managed_spans = []

      (target.entries || []).each do |entry|
        if entry.kind == "local"
          append_scoped_local(builder, project, entry.path, symbols)
        else
          append_scoped_package(builder, managed_spans, project, target, output, entry.name, symbols)
        end
      end

      RenderedTarget.new(path: output, content: builder.finish, managed_spans: managed_spans)
    end

    def render_legacy_compose(project, target, output)
      builder = ContentBuilder.new
      append_header(builder, project, target, output)

      unbound_locals = project.local_files.select do |local|
        declaration = project.manifest.local.find { |entry| entry.path == local.manifest_path }
        declaration.nil? || !declaration.bound
      end

      unless unbound_locals.empty?
        builder.append_line("## Additional instructions")
        builder.append_empty_line
      end
      symbols = project.manifest.symbols || {}
      unbound_locals.each do |local|
        next if local.content.empty? && local.optional

        builder.append_line("### #{local.manifest_path}")
        builder.append_body(Substitute.substitute_pray_symbols(local.content, symbols))
        builder.append_empty_line
      end

      builder.append_line("## Shared instructions")
      builder.append_empty_line

      managed_spans = []
      project.packages.each do |package|
        next unless Environment.package_matches_environment?(package.declaration.groups, project.environment)
        next unless Destination.package_bound_to_compose?(package.declaration, target)

        package.selected_exports.each do |export|
          next unless should_inline_export?(package, export)

          append_managed_export(builder, managed_spans, package, export, target, output, symbols)
        end
      end

      RenderedTarget.new(path: output, content: builder.finish, managed_spans: managed_spans)
    end

    def append_header(builder, project, target, output)
      text = ComposeDest.header_text(target, output, project.manifest.render.header)
      return unless text

      builder.append_body(text)
      builder.append_empty_line
    end

    def append_scoped_local(builder, project, path, symbols)
      local = project.local_files.find { |candidate| candidate.manifest_path == path }
      return unless local
      return if local.content.empty? && local.optional

      builder.append_body(Substitute.substitute_pray_symbols(local.content, symbols))
      builder.append_empty_line
    end

    def append_scoped_package(builder, managed_spans, project, target, output, package_name, symbols)
      package = project.packages.find { |candidate| candidate.declaration.name == package_name }
      return unless package
      return unless Environment.package_matches_environment?(package.declaration.groups, project.environment)

      package.selected_exports.each do |export|
        next unless should_inline_export?(package, export)

        append_managed_export(builder, managed_spans, package, export, target, output, symbols)
      end
    end

    def append_managed_export(builder, managed_spans, package, export, target, output, symbols)
      body = package.export_bodies[export]
      unless body
        raise Error.integrity("compose cannot write binary export #{export}; use file: for unmarked bytes") if package.spec.exports[export]&.kind == "file"
        raise Error.render("package #{package.declaration.name} is missing cached export #{export}")
      end

      body = Substitute.substitute_pray_symbols(body, symbols)
      identifier = Hashing.marker_id("#{package.declaration.name}:#{export}:#{target.name}")
      open_line = builder.next_line_number
      builder.append_line("<!-- pray:#{identifier} -->")
      builder.append_body(body)
      close_line = builder.next_line_number
      builder.append_line("<!-- pray:#{identifier} -->")
      managed_spans << ManagedSpanRecord.new(
        id: identifier,
        target: output,
        open_line: open_line,
        close_line: close_line,
        ideal_checksum: Hashing.checksum_managed_span_content(body),
        package: package.declaration.name,
        export: export,
        source_checksum: package.source_checksum,
        silenced: false
      )
      builder.append_empty_line
    end

    def should_inline_export?(package, export_name)
      export = package.spec.exports[export_name]
      export.nil? || %w[fragment file].include?(export.kind)
    end

    def collect_exact_file_bindings(project, planned)
      project.packages.each do |package|
        destination = package.declaration.file
        next unless destination
        next unless Environment.package_matches_environment?(package.declaration.groups, project.environment)

        matched = false
        package.selected_exports.each do |export_name|
          export = package.spec.exports[export_name]
          next unless export&.kind == "file"

          source = File.join(package.root, export.path)
          raise Error.render("file export source missing: #{source}") unless File.file?(source)

          planned << PlannedProvisionedFile.new(
            path: destination,
            source: source,
            package: package.declaration.name,
            export: export_name
          )
          matched = true
          break
        end
        unless matched
          raise Error.render(
            "package #{package.declaration.name} has file: \"#{destination}\" but no selected file export"
          )
        end
      end
    end

    def collect_legacy_skill_files(project, package, destination_root, planned)
      package.spec.skills.each do |skill_name, skill|
        next if legacy_skill_covered_by_export?(package, skill)

        skill_files = package.skill_files[skill_name]
        raise Error.render("package #{package.declaration.name} has no indexed files for legacy skill #{skill_name}") unless skill_files

        collect_tree_files(
          project,
          File.join(package.root, skill.path),
          File.join(destination_root, skill_name),
          skill_files,
          [],
          [],
          package.declaration.name,
          skill_name,
          planned
        )
      end
    end

    def legacy_skill_covered_by_export?(package, skill)
      package.spec.exports.any? do |export_name, export|
        package.selected_exports.include?(export_name) &&
          %w[folder skill].include?(export.kind) &&
          export.path.delete_suffix("/") == skill.path.delete_suffix("/")
      end
    end

    def collect_selected_export_files(project, package, destination_root, planned)
      package.selected_exports.each do |export_name|
        export = package.spec.exports[export_name]
        next unless export

        case export.kind
        when "folder", "skill"
          indexed_files = package.skill_files[export_name]
          unless indexed_files
            raise Error.render("package #{package.declaration.name} has no indexed files for folder export #{export_name}")
          end

          destination_name = folder_destination_name(export_name, export.path)
          collect_tree_files(
            project,
            File.join(package.root, export.path),
            File.join(destination_root, destination_name),
            indexed_files,
            export.only || [],
            export.except || [],
            package.declaration.name,
            export_name,
            planned
          )
        when "file"
          next if package.declaration.file

          source = File.join(package.root, export.path)
          raise Error.render("file export source missing: #{source}") unless File.file?(source)

          destination = File.join(destination_root, export_name, File.basename(source))
          planned << PlannedProvisionedFile.new(
            path: relative_project_path(project, destination),
            source: source,
            package: package.declaration.name,
            export: export_name
          )
        end
      end
    end

    def folder_destination_name(export_name, export_path)
      File.basename(export_path.delete_suffix("/")).empty? ? export_name : File.basename(export_path.delete_suffix("/"))
    end

    def collect_tree_files(project, source_root, destination_root, relative_files, only, except, package_name, export_name, planned)
      raise Error.render("folder source directory missing: #{source_root}") unless File.directory?(source_root)
      raise Error.render("no files listed in package manifest for #{source_root}") if relative_files.empty?

      matched = false
      relative_files.each do |relative|
        next if !only.empty? && !only.include?(relative)
        next if except.include?(relative)

        source = File.join(source_root, relative)
        raise Error.render("provisioned file missing: #{source}") unless File.file?(source)

        destination = File.join(destination_root, relative)
        planned << PlannedProvisionedFile.new(
          path: relative_project_path(project, destination),
          source: source,
          package: package_name,
          export: export_name
        )
        matched = true
      end

      if !matched && only.empty? && except.empty?
        raise Error.render("no files listed in package manifest for #{source_root}")
      end
    end

    def relative_project_path(project, absolute)
      Pathname(absolute).relative_path_from(Pathname(project.project_root)).to_s
    rescue ArgumentError
      absolute
    end
  end
end
