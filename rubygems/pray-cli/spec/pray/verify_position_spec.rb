# frozen_string_literal: true

require "spec_helper"

RSpec.describe Pray::VerifyPosition do
  def span(id, open_line, close_line, checksum)
    Pray::ManagedSpanRecord.new(
      id: id,
      target: "AGENTS.md",
      open_line: open_line,
      close_line: close_line,
      ideal_checksum: checksum,
      package: "sample/base",
      export: "guidance",
      source_checksum: "sha256:source",
      silenced: false
    )
  end

  it "groups uniform position drift with local cause" do
    spans = [
      span("aaaa1111", 4, 6, "sha256:one"),
      span("bbbb2222", 8, 10, "sha256:two")
    ]
    markers = {
      "aaaa1111" => [6, 8, "sha256:one"],
      "bbbb2222" => [10, 12, "sha256:two"]
    }
    on_disk = [
      "# Title",
      "",
      "Local alpha",
      "Extra unmarked",
      "",
      "Local beta",
      "<!-- pray:aaaa1111 -->",
      "body one",
      "<!-- pray:aaaa1111 -->",
      "",
      "<!-- pray:bbbb2222 -->",
      "body two",
      "<!-- pray:bbbb2222 -->"
    ]
    fresh = [
      "# Title",
      "",
      "Local alpha",
      "Local beta",
      "<!-- pray:aaaa1111 -->",
      "body one",
      "<!-- pray:aaaa1111 -->",
      "",
      "<!-- pray:bbbb2222 -->",
      "body two",
      "<!-- pray:bbbb2222 -->"
    ]
    locals = [
      Pray::ResolvedLocalFile.new(
        path: ".agents/project.md",
        manifest_path: ".agents/project.md",
        content: "Local alpha\nLocal beta\n",
        position: "before",
        optional: false
      )
    ]

    summary = described_class.summarize_position_drift(
      "AGENTS.md",
      spans,
      markers,
      on_disk,
      fresh,
      locals
    )

    expect(summary.marker_count).to eq(2)
    expect(summary.uniform_delta).to eq(2)
    expect(summary.first_id).to eq("aaaa1111")
    message = described_class.format_position_drift_message(summary)
    expect(message).to include("`AGENTS.md` position drift (+2 lines) across 2 markers")
    expect(message).to include("first marker `aaaa1111` lock 4:6, file 6:8")
    expect(message).to include(
      "cause: `AGENTS.md:4` unmarked text differs from `.agents/project.md:2`"
    )
  end

  it "skips checksum mismatched spans" do
    spans = [span("aaaa1111", 2, 4, "sha256:ideal")]
    markers = {"aaaa1111" => [3, 5, "sha256:edited"]}
    on_disk = ["text", "<!-- pray:aaaa1111 -->", "edited", "<!-- pray:aaaa1111 -->"]

    expect(
      described_class.summarize_position_drift(
        "AGENTS.md",
        spans,
        markers,
        on_disk,
        nil,
        []
      )
    ).to be_nil
  end
end
