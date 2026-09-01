# frozen_string_literal: true

require "fileutils"
require "pathname"

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

    def write_rendered_targets(project, rendered)
      rendered.each do |target|
        PathSafety.validate_project_relative_path!(target.path)
        path = File.join(project.project_root, target.path)
        FileUtils.mkdir_p(File.dirname(path))
        File.write(path, target.content)
      end
      materialize_provisioned_exports(project)
    end

    def materialize_provisioned_exports(project)
      symbols = project.manifest.symbols || {}
      planned_provisioned_files(project).each do |file|
        PathSafety.validate_project_relative_path!(file.path)
        destination = File.join(project.project_root, file.path)
        FileUtils.mkdir_p(File.dirname(destination))
        write_provisioned_file(file.source, destination, symbols)
      end
    end

    def write_provisioned_file(source, destination, symbols)
      bytes = File.binread(source)
      text = bytes.force_encoding(Encoding::UTF_8)
      if text.valid_encoding?
        File.write(destination, Substitute.substitute_pray_symbols(text, symbols))
      else
        File.binwrite(destination, bytes)
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

    PlannedProvisionedFile = Struct.new(:path, :source)

    def render_target(project, target, output)
      if target.scoped && target.mode == "compose"
        return render_scoped_compose(project, target, output)
      end
      render_legacy_compose(project, target, output)
    end

    def render_scoped_compose(project, target, output)
      builder = ContentBuilder.new
      append_header(builder, project, output)
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
      append_header(builder, project, output)

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

    def append_header(builder, project, output)
      return unless project.manifest.render.header

      output_name = File.basename(output)
      builder.append_line("<!-- pray:0 ignore-comments -->")
      builder.append_empty_line
      builder.append_line("# Agent context")
      builder.append_empty_line
      builder.append_line("Do not edit managed blocks in `#{output_name}` or provisioned files under `.agents/`.")
      builder.append_line("To change shared guidance, update `Prayfile` and run `pray install`.")
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
      raise Error.render("package #{package.declaration.name} is missing cached export #{export}") unless body

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
      export.nil? || export.kind == "fragment"
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

          planned << PlannedProvisionedFile.new(path: destination, source: source)
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
            planned
          )
        when "file"
          next if package.declaration.file

          source = File.join(package.root, export.path)
          raise Error.render("file export source missing: #{source}") unless File.file?(source)

          destination = File.join(destination_root, export_name, File.basename(source))
          planned << PlannedProvisionedFile.new(
            path: relative_project_path(project, destination),
            source: source
          )
        end
      end
    end

    def folder_destination_name(export_name, export_path)
      File.basename(export_path.delete_suffix("/")).empty? ? export_name : File.basename(export_path.delete_suffix("/"))
    end

    def collect_tree_files(project, source_root, destination_root, relative_files, only, except, planned)
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
          source: source
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

    class ContentBuilder
      def initialize
        @content = +""
      end

      def next_line_number
        @content.count("\n") + 1
      end

      def append_line(line)
        @content << line << "\n"
      end

      def append_empty_line
        @content << "\n"
      end

      def append_body(body)
        trimmed = body.sub(/\n+\z/, "")
        return if trimmed.empty?

        trimmed.each_line(chomp: true) { |line| append_line(line) }
      end

      def finish
        @content.sub(/\n\n+\z/, "\n")
        @content << "\n" unless @content.end_with?("\n")
        @content
      end
    end
  end
end
