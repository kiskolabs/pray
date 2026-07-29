# frozen_string_literal: true

require "spec_helper"

RSpec.describe "legacy Prayfile keyword deprecation" do
  it "records target, output, and agent warnings" do
    manifest = Pray.parse_manifest(<<~PRAYFILE)
      prayfile "1"
      target :tool_a do
        output "INSTRUCTIONS.md"
      end
      agent "sample/base", "~> 1.0"
    PRAYFILE

    expect(manifest.deprecated_keywords).to eq(%w[target output agent])
    warnings = manifest.deprecation_warnings
    expect(warnings.length).to eq(3)
    expect(warnings).to all(include("version 2"))
  end

  it "does not mark recommended compose/pray forms" do
    manifest = Pray.parse_manifest(<<~PRAYFILE)
      prayfile "1"
      compose "AGENTS.md" do
        pray "sample/base", "~> 1.0"
      end
    PRAYFILE

    expect(manifest.deprecated_keywords).to eq([])
    expect(manifest.deprecation_warnings).to eq([])
  end
end
