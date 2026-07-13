# frozen_string_literal: true

require "fileutils"

module Pray
  module Plan
    MaterializationPreview = Struct.new(
      :package_lines, :lockfile, :targets, :provisioned, :warnings,
      keyword_init: true
    )

    module_function

    def build_materialization_preview(project, rendered, lockfile, lockfile_path, previous_lockfile)
      MaterializationPreview.new(
        package_lines: package_summary_lines(previous_lockfile, lockfile, project),
        lockfile: lockfile_change_status(lockfile_path, lockfile),
        targets: rendered.map { |target| target_change(project, target) },
        provisioned: Render.planned_provisioned_files(project).map { |file| provisioned_change(project, file) },
        warnings: []
      )
    end

    def print_materialization_report(preview, mode)
      puts mode == :plan ? "Plan" : "Install"
      preview.package_lines.each { |line| puts line }
      puts "Lockfile: #{preview.lockfile}"
      preview.targets.each do |path, change|
        puts "Target #{path}: #{change}"
      end
      preview.provisioned.each do |path, change|
        puts "Provisioned #{path}: #{change}"
      end
      preview.warnings.each { |warning| puts "Warning: #{warning}" }
    end

    def package_summary_lines(previous_lockfile, lockfile, _project)
      if previous_lockfile.nil?
        lockfile.package.map do |entry|
          "Package #{entry.name} #{entry.version} (new)"
        end
      else
        lockfile.package.filter_map do |entry|
          previous = previous_lockfile.package.find { |package| package.name == entry.name }
          next "Package #{entry.name} #{entry.version} (new)" unless previous
          next if previous.version == entry.version && previous.tree_hash == entry.tree_hash

          "Package #{entry.name} #{previous.version} -> #{entry.version}"
        end
      end
    end

    def lockfile_change_status(lockfile_path, lockfile)
      return "create" unless File.exist?(lockfile_path)

      existing = Pray.read_lockfile(lockfile_path)
      Pray.lockfiles_equivalent?(lockfile, existing) ? "unchanged" : "update"
    end

    def target_change(project, target)
      path = File.join(project.project_root, target.path)
      change = if !File.exist?(path)
                 "write"
               elsif File.read(path) == target.content
                 "unchanged"
               else
                 "update"
               end
      [target.path, change]
    end

    def provisioned_change(project, file)
      destination = File.join(project.project_root, file.path)
      change = if !File.exist?(destination)
                 "write"
               elsif FileUtils.compare_file(destination, file.source)
                 "unchanged"
               else
                 "update"
               end
      [file.path, change]
    end
  end
end
