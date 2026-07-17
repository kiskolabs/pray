# frozen_string_literal: true

require "fileutils"

module Pray
  module CLI
    def add_command(name:, constraint: nil, path: nil)
      manifest_path_value = manifest_path
      manifest_text = Pray.read_manifest_text(manifest_path_value)
      manifest = Pray.parse_manifest(manifest_text)
      if manifest.packages.any? { |package| package.name == name }
        raise Error.manifest("package #{name} already exists")
      end

      declaration = Pray.format_package_declaration(
        ManifestPackage.new(name: name, constraint: constraint || "*", path: path)
      )
      File.write(manifest_path_value, insert_manifest_statement(manifest_text, declaration))
    end

    def remove_command(name)
      raise Error.manifest("remove requires a package name") unless name

      manifest_path_value = manifest_path
      manifest_text = Pray.read_manifest_text(manifest_path_value)
      manifest = Pray.parse_manifest(manifest_text)
      unless manifest.packages.any? { |package| package.name == name }
        raise Error.manifest("package #{name} not found")
      end

      File.write(manifest_path_value, remove_manifest_statement(manifest_text, name))
      install_command({locked: false, frozen: false, offline: false})
    end

    def list_command
      project = resolve_current_project
      lines = ["Package list"]
      project.packages.each do |package|
        lines << "#{package.declaration.name} #{package.spec.version} source=#{package_source_summary(package)} exports=#{format_list(package.selected_exports)}"
      end
      puts lines.join("\n")
    end

    def tree_command
      project = resolve_current_project
      package_map = project.packages.to_h { |package| [package.declaration.name, package] }
      lines = ["Dependency tree"]
      project.packages.each do |package|
        render_tree_node(package, package_map, 0, Set.new, lines)
      end
      puts lines.join("\n")
    end

    def package_command
      project = resolve_current_project
      project.packages.each do |package|
        output_path = Archive.package_archive_path(package.declaration.name, package.spec.version)
        Archive.write_package_archive(package, output_path)
      end
    end

    def explain_command(name)
      raise Error.resolution("explain requires a package name") unless name

      project = resolve_current_project
      package = project.packages.find { |entry| entry.declaration.name == name }
      raise Error.resolution("package #{name} not found") unless package

      lockfile = File.exist?(lockfile_path) ? Pray.read_lockfile(lockfile_path) : nil
      lockfile_package = lockfile&.package&.find { |entry| entry.name == name }

      lines = ["Package explanation"]
      lines << "name: #{package.declaration.name}"
      lines << "constraint: #{package.declaration.constraint}"
      lines << "resolved version: #{package.spec.version}"
      if package.registry_latest_version
        lines << "registry latest: #{package.registry_latest_version}"
      end
      lines << "source: #{package_source_summary(package)}"
      lines << "exports: #{format_list(package.selected_exports)}"
      lines << "dependencies: #{format_list(package.spec.dependencies.map(&:name))}"
      lines << "tree hash: #{package.tree_hash}"
      lines << "artifact hash: #{package.artifact_hash}"
      if lockfile_package
        lines << "lockfile version: #{lockfile_package.version}"
        lines << "lockfile path: #{lockfile_package.path}"
        lines << "lockfile exports: #{format_list(lockfile_package.exports)}"
      else
        lines << "lockfile record: missing"
      end
      puts lines.join("\n")
    end

    def outdated_command(_arguments)
      previous_lockfile = File.exist?(lockfile_path) ? Pray.read_lockfile(lockfile_path) : nil
      project = resolve_current_project
      rendered = Render.render_project(project)
      latest_lockfile = build_lockfile(project, rendered)
      puts "Outdated packages"
      if previous_lockfile && Pray.lockfiles_equivalent?(latest_lockfile, previous_lockfile)
        puts "All packages up to date"
      else
        Plan.package_summary_lines(previous_lockfile, latest_lockfile, project).each { |line| puts line }
      end
    end
  end
end
