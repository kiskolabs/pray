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

  it "records the skills keyword" do
    manifest = Pray.parse_manifest(<<~PRAYFILE)
      prayfile "1"
      target :tool_a do
        skills ".agents/vendor"
      end
      tree ".agents/skills" do
      end
    PRAYFILE

    expect(manifest.deprecated_keywords).to include("skills")
    warnings = manifest.deprecation_warnings
    expect(warnings).to include(a_string_including("`skills`"))
  end

  it "does not mark a tree dest whose path contains skills" do
    manifest = Pray.parse_manifest(<<~PRAYFILE)
      prayfile "1"
      tree ".agents/skills" do
        pray "sample/base", "~> 1.0"
      end
    PRAYFILE

    expect(manifest.deprecated_keywords).to eq([])
  end

  it "warns for spec.skills and skill export type" do
    spec = Pray.parse_package_spec(<<~SPEC)
      Package::Specification.new do |spec|
        spec.name = "sample/legacy"
        spec.version = "1.0.0"
        spec.files = ["folders/review/README.md"]
        spec.exports = {
          "review" => { type: "skill", path: "folders/review" }
        }
        spec.skills = {
          "other" => { path: "folders/other", summary: "other" }
        }
      end
    SPEC

    warnings = spec.deprecation_warnings
    expect(warnings).to include(a_string_including("`spec.skills`"))
    expect(warnings).to include(a_string_including("`skill`"))
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
