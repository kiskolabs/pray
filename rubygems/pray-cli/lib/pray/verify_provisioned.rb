# frozen_string_literal: true

module Pray
  module VerifyProvisioned
    module_function

    def push_findings(project, report, lockfile)
      push_exclusive_file_export_findings(project, report)
      previous = RenderDest.previous_map(lockfile)
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
        destination_bytes = RenderDest.read_regular_bytes(absolute, path_text)
        expected_bytes = Render.expected_provisioned_bytes(file.source, project.manifest.symbols || {})
        destination_hash = Hashing.sha256_prefixed(destination_bytes)
        next if destination_hash == Hashing.sha256_prefixed(expected_bytes.b)

        owned = previous[path_text]&.content_hash == destination_hash
        recovery = if owned
          "Run `pray install` to restore it."
        else
          "Inspect your changes and move the file aside, then run `pray install` to restore it."
        end
        report.findings << VerificationFinding.new(
          kind: "package_integrity",
          message: "Provisioned file `#{path_text}` no longer matches package `#{file.package}`. #{recovery}"
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
