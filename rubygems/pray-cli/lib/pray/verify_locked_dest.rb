# frozen_string_literal: true

module Pray
  module Verify
    module_function

    def inspect_locked_destinations(project_root, lockfile)
      report = VerificationReport.new
      lockfile.managed_span.group_by(&:target).each do |target_path, spans|
        absolute_path = File.join(project_root, target_path)
        unless File.exist?(absolute_path)
          report.findings << VerificationFinding.new(
            kind: "verify_error",
            message: "Rendered file `#{target_path}` is missing. Run `pray install` to generate it."
          )
          next
        end

        text = RenderDest.decode_utf8(
          RenderDest.read_regular_bytes(absolute_path, target_path),
          target_path
        )
        markers = marker_positions(text.lines(chomp: true))
        spans.each do |span|
          marker = markers[span.id]
          unless marker
            report.findings << VerificationFinding.new(
              kind: "removed_prayer",
              message: "`#{target_path}` is missing managed marker `#{span.id}` for `#{span.package}::#{span.export}`. Run `pray install` to restore the managed span."
            )
            next
          end
          next if marker[2] == span.ideal_checksum

          report.findings << VerificationFinding.new(
            kind: "custom_implementation",
            message: "`#{target_path}` marker `#{span.id}` (`#{span.package}::#{span.export}`) was edited. Restore the managed block or run `pray install` to regenerate it."
          )
        end
        find_orphan_marker_findings(spans, markers, target_path).each do |finding|
          report.findings << finding
        end
      end
      report
    end
  end
end
