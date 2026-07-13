# frozen_string_literal: true

require "fileutils"

module Pray
  module CLI
    def manifest_path = MANIFEST_PATH
    def lockfile_path = LOCKFILE_PATH

    def default_output_for_target(target)
      case target
      when "tool_a" then "INSTRUCTIONS"
      when "tool_b" then "TOOL_B"
      else target.upcase
      end
    end

    def build_lockfile(project, rendered)
      LockfileIO.build_lockfile(
        project.manifest_hash,
        project.project_root,
        project.manifest.sources,
        project.manifest.targets,
        rendered,
        project.packages,
        project.source_revisions,
        project.source_host_keys
      )
    end

    def ensure_existing_lockfile(path)
      raise Error.verify("missing Prayfile.lock; run pray install first") unless File.exist?(path)

      Pray.read_lockfile(path)
    end

    def ensure_lockfile_current(project, rendered, existing)
      current = build_lockfile(project, rendered)
      return if Pray.lockfiles_equivalent?(current, existing)

      raise Error.verify("lockfile needs update; rerun pray install to refresh Prayfile.lock")
    end

    def ensure_rendered_outputs_current(project, rendered)
      rendered.each do |target|
        path = File.join(project.project_root, target.path)
        on_disk = File.read(path)
        if on_disk != target.content
          raise Error.render("#{path} is stale; rerun pray install to regenerate it or pray plan to inspect the diff")
        end
      end
    end

    def insert_manifest_statement(text, statement)
      lines = text.lines.map(&:chomp)
      insertion_index = lines.index do |line|
        trimmed = line.lstrip
        trimmed.start_with?("local ") || trimmed.start_with?("render ")
      end || lines.length
      lines.insert(insertion_index, statement)
      output = lines.join("\n")
      output += "\n" unless output.end_with?("\n")
      output
    end

    def remove_manifest_statement(text, name)
      lines = text.lines.map(&:chomp)
      package_prefix = "agent \"#{name}\""
      alternate_prefix = "agent '#{name}'"
      index = lines.index do |line|
        trimmed = line.lstrip
        trimmed.start_with?(package_prefix) || trimmed.start_with?(alternate_prefix)
      end
      if index
        lines.delete_at(index)
        lines.delete_at(index) if index < lines.length && lines[index].strip.empty?
        lines.delete_at(index - 1) if index.positive? && lines[index - 1].strip.empty?
      end
      output = lines.join("\n")
      output += "\n" unless output.end_with?("\n")
      output
    end

    def format_marker_comments(text)
      lines = text.split("\n", -1).map do |line|
        canonical_marker_line(line) || line
      end
      output = lines.join("\n")
      output += "\n" unless output.end_with?("\n")
      output
    end

    def canonical_marker_line(line)
      trimmed = line.strip
      remainder = trimmed.delete_prefix("<!--").strip
      return nil unless remainder.start_with?("pray:")

      content = remainder.delete_prefix("pray:").strip.delete_suffix("-->").strip
      return "<!-- pray:0 ignore-comments -->" if content == "0 ignore-comments"
      return "<!-- pray:#{content} -->" if content.match?(/\A[a-z0-9]+\z/)

      nil
    end

    def package_source_summary(package)
      if package.declaration.path
        "path:#{package.declaration.path}"
      elsif package.declaration.source
        "source:#{package.declaration.source}"
      else
        "root:#{package.root}"
      end
    end

    def format_list(values)
      values.empty? ? "none" : values.join(", ")
    end

    def format_verification_report(report)
      Verify.format_verification_report(report)
    end

    def format_drift_report(report)
      Verify.format_drift_report(report)
    end

    def render_tree_node(package, package_map, depth, ancestry, lines)
      indent = "  " * depth
      lines << "#{indent}#{package.declaration.name} #{package.spec.version}"
      return unless ancestry.add?(package.declaration.name)

      package.spec.dependencies.each do |dependency|
        resolved = package_map[dependency.name]
        if resolved
          if ancestry.include?(resolved.declaration.name)
            lines << "#{indent}  #{resolved.declaration.name} #{resolved.spec.version} (cycle)"
          else
            render_tree_node(resolved, package_map, depth + 1, ancestry, lines)
          end
        else
          lines << "#{indent}  #{dependency.name} (missing)"
        end
      end
    end

    def remove_path_if_exists(path)
      return unless File.exist?(path)

      File.directory?(path) ? FileUtils.rm_rf(path) : File.delete(path)
    end
  end
end
