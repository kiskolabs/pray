# frozen_string_literal: true

require "spec_helper"

RSpec.describe Pray::ApplyReport do
  def local_file(checksum)
    Pray::ResolvedLocalFile.new(
      path: "/tmp/.agents/project.md",
      manifest_path: ".agents/project.md",
      content: "note\n",
      source_checksum: checksum,
      position: "after",
      optional: false
    )
  end

  def lockfile_with_local(checksum)
    Pray::Lockfile.new(
      managed_span: [
        Pray::ManagedSpanRecord.new(
          id: "aaaa1111",
          target: "AGENTS.md",
          open_line: 1,
          close_line: 3,
          ideal_checksum: checksum,
          package: Pray::LOCAL_EMBED_PACKAGE,
          export: ".agents/project.md",
          source_checksum: checksum,
          silenced: false
        )
      ]
    )
  end

  def project_with_local(checksum)
    Pray::ResolvedProject.new(
      local_files: [local_file(checksum)]
    )
  end

  it "prints checked when the local checksum is unchanged" do
    lines = described_class.local_summary_lines(
      lockfile_with_local("sha256:old"),
      project_with_local("sha256:old")
    )
    expect(lines.first).to include("checked")
  end

  it "lists local checksum drift for outdated" do
    lines = described_class.outdated_local_lines(
      lockfile_with_local("sha256:old"),
      project_with_local("sha256:new")
    )
    expect(lines.first).to include("sha256:old -> sha256:new")
  end
end
