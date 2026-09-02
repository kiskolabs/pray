# frozen_string_literal: true

module Pray
  module VerifyProvisioned
    module_function

    def push_findings(project, report)
      push_exclusive_file_export_findings(project, report)
      Render.planned_provisioned_files(project).each do |file|
        path_text = file.path.to_s.tr("\\", "/")
        absolute = File.join(project.project_root, file.path)
        if File.symlink?(absolute)
          report.findings << VerificationFinding.new(
            kind: "verify_error",
            message: "Provisioned file `#{path_text}` is a symbolic link. Remove the link or choose another destination."
          )
          next
        end
        unless File.file?(absolute)
          report.findings << VerificationFinding.new(
            kind: "verify_error",
            message: "Provisioned file `#{path_text}` from `#{file.package}` is missing. Run `pray install` to materialize it."
          )
          next
        end
        destination_bytes = File.binread(absolute)
        expected_bytes = Render.expected_provisioned_bytes(file.source, project.manifest.symbols || {})
        next if Hashing.sha256_prefixed(destination_bytes) == Hashing.sha256_prefixed(expected_bytes.b)

        report.findings << VerificationFinding.new(
          kind: "package_integrity",
          message: "Provisioned file `#{path_text}` no longer matches package `#{file.package}`. Run `pray install` to restore it."
        )
      end
    end

    def push_exclusive_file_export_findings(project, report)
      project.packages.each do |package|
        destination = package.declaration.file
        next unless destination

        has_file_export = package.selected_exports.any? do |name|
          export = package.spec.exports[name]
          export && export.kind == "file"
        end
        next if has_file_export

        report.findings << VerificationFinding.new(
          kind: "verify_error",
          message: "Package `#{package.declaration.name}` declares file: \"#{destination}\" but has no selected file export."
        )
      end
    end
  end
end
