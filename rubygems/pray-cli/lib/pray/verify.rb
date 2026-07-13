# frozen_string_literal: true

module Pray
  VerificationFinding = Struct.new(:kind, :message, keyword_init: true) do
    def warning? = kind == "orphan_marker"
    def error? = !warning?
  end

  VerificationReport = Struct.new(:findings, keyword_init: true) do
    def initialize(findings: [])
      super
    end

    def clean? = findings.empty?
    def warnings? = findings.any?(&:warning?)
    def errors? = findings.any?(&:error?)
  end

  module Verify
    module_function

    def inspect_project(project, lockfile)
      collect_verification_report(project, lockfile).first
    end

    def verify_project(project, lockfile, strict: false)
      report = inspect_project(project, lockfile)
      return report if report.clean?

      if strict || report.errors?
        raise Error.verify(format_verification_report(report))
      end

      report
    end

    def drift_project(project, lockfile)
      report, rendered_targets = collect_verification_report(project, lockfile)
      Render.render_project(project).each do |target|
        normalized_fresh = Hashing.normalize_line_endings(target.content)
        on_disk = rendered_targets[target.path]
        on_disk = Hashing.normalize_line_endings(on_disk) if on_disk
        unless on_disk == normalized_fresh
          report.findings << VerificationFinding.new(
            kind: "renderer_drift",
            message: "#{target.path} differs from fresh render"
          )
        end
        unless lockfile_targets(lockfile).include?(target.path)
          report.findings << VerificationFinding.new(
            kind: "renderer_drift",
            message: "#{target.path} is not tracked in lockfile"
          )
        end
      end

      if report.clean?
        report
      else
        raise Error.verify(format_drift_report(report))
      end
    end

    def collect_verification_report(project, lockfile)
      report = VerificationReport.new
      rendered_targets = {}

      if project.manifest_hash != lockfile.manifest_hash
        report.findings << VerificationFinding.new(
          kind: "verify_error",
          message: "Prayfile changed since `Prayfile.lock` was generated. Run `pray install` to refresh the lockfile."
        )
      end

      locked_packages = lockfile.package.to_h { |entry| [entry.name, entry] }
      project.packages.each do |package|
        locked = locked_packages.delete(package.declaration.name)
        if locked
          if locked.tree_hash != package.tree_hash
            report.findings << VerificationFinding.new(
              kind: "package_integrity",
              message: "Package `#{package.declaration.name}` no longer matches the locked tree hash. Run `pray install` to re-resolve packages."
            )
          end
          if locked.version != package.spec.version
            report.findings << VerificationFinding.new(
              kind: "verify_error",
              message: "Package `#{package.declaration.name}` resolved to version #{package.spec.version} but `Prayfile.lock` has #{locked.version}. Run `pray install` to refresh the lockfile."
            )
          end
        else
          report.findings << VerificationFinding.new(
            kind: "verify_error",
            message: "Package `#{package.declaration.name}` is declared in Prayfile but missing from `Prayfile.lock`. Run `pray install` to update the lockfile."
          )
        end
      end
      locked_packages.each_value do |locked|
        report.findings << VerificationFinding.new(
          kind: "verify_error",
          message: "Package `#{locked.name}` is in `Prayfile.lock` but not declared in Prayfile. Remove it from the lockfile with `pray install` or add it back to Prayfile."
        )
      end

      target_spans = lockfile.managed_span.group_by(&:target)
      target_spans.each do |target_path, spans|
        absolute_path = File.join(project.project_root, target_path)
        unless File.exist?(absolute_path)
          report.findings << VerificationFinding.new(
            kind: "verify_error",
            message: "Rendered file `#{target_path}` is missing. Run `pray install` to generate it."
          )
          next
        end

        text = File.read(absolute_path)
        rendered_targets[target_path] = text
        lines = text.lines(chomp: true)
        markers = marker_positions(lines)
        spans.each do |span|
          marker = markers[span.id]
          if marker.nil?
            report.findings << VerificationFinding.new(
              kind: "removed_prayer",
              message: "`#{target_path}` is missing managed marker `#{span.id}` for `#{span.package}::#{span.export}`. Run `pray install` to restore the managed span."
            )
          else
            open_line, close_line, checksum = marker
            if checksum != span.ideal_checksum
              report.findings << VerificationFinding.new(
                kind: "custom_implementation",
                message: "`#{target_path}` marker `#{span.id}` (`#{span.package}::#{span.export}`) was edited. Restore the managed block or run `pray install` to regenerate it."
              )
            end
            if open_line != span.open_line || close_line != span.close_line
              report.findings << VerificationFinding.new(
                kind: "position_drift",
                message: "`#{target_path}` marker `#{span.id}` (`#{span.package}::#{span.export}`) moved to different lines. Run `pray install` to restore expected positions."
              )
            end
          end
        end
        find_orphan_marker_findings(spans, markers, target_path).each do |finding|
          report.findings << finding
        end
      end

      project.local_files.each do |local|
        next if local.optional
        next if File.exist?(local.path)

        report.findings << VerificationFinding.new(
          kind: "verify_error",
          message: Resolve.missing_local_embed_guidance(local.manifest_path)
        )
      end

      [report, rendered_targets]
    end

    def find_orphan_marker_findings(spans, markers, target_path)
      tracked_ids = spans.to_h { |span| [span.id, true] }
      markers.keys.filter_map do |marker_id|
        next if marker_id == "0" || tracked_ids[marker_id]

        VerificationFinding.new(
          kind: "orphan_marker",
          message: "`#{target_path}` contains marker `#{marker_id}` that is not tracked in `Prayfile.lock`. Remove the marker or run `pray install` to reconcile."
        )
      end
    end

    def marker_positions(lines)
      markers = {}
      active = nil
      lines.each_with_index do |line, index|
        parsed = parse_marker(line)
        if parsed.nil?
          active[2] << line if active
          next
        end
        case parsed
        when :ignore
          next
        else
          identifier = parsed
          if active.nil?
            active = [identifier, index + 1, []]
          elsif active[0] == identifier
            checksum = Hashing.checksum_managed_body_line_refs(active[2])
            markers[active[0]] = [active[1], index + 1, checksum]
            active = nil
          end
        end
      end
      markers
    end

    def parse_marker(line)
      trimmed = line.strip
      remainder = trimmed.delete_prefix("<!--").strip
      return nil unless remainder.start_with?("pray:")

      content = remainder.delete_prefix("pray:").strip.delete_suffix("-->").strip
      return :ignore if content == "0 ignore-comments"
      return content if content.match?(/\A[a-z0-9]+\z/)

      nil
    end

    def lockfile_targets(lockfile)
      lockfile.target.flat_map(&:outputs)
    end

    def format_verification_report(report)
      report.findings.map { |finding| "#{finding.kind}: #{finding.message}" }.join("\n")
    end

    def format_drift_report(report)
      sections = {}
      report.findings.each do |finding|
        section = drift_section_for_kind(finding.kind)
        sections[section] ||= []
        sections[section] << finding
      end

      ordered = [
        "Lockfile changes",
        "Package changes",
        "Managed span changes",
        "Rendered file changes",
        "Warnings"
      ]
      lines = []
      ordered.each do |section|
        findings = sections[section]
        next unless findings

        lines << section
        findings.each do |finding|
          lines << "  #{finding.kind}: #{finding.message}"
        end
      end
      lines.join("\n")
    end

    def drift_section_for_kind(kind)
      case kind
      when "verify_error" then "Lockfile changes"
      when "package_integrity" then "Package changes"
      when "custom_implementation", "removed_prayer", "position_drift", "orphan_marker" then "Managed span changes"
      when "renderer_drift" then "Rendered file changes"
      else "Warnings"
      end
    end
  end
end
