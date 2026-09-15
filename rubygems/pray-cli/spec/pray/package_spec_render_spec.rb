# frozen_string_literal: true

require "spec_helper"

RSpec.describe Pray::PackageSpecRender do
  it "round-trips a fork spec with upstream" do
    spec = Pray.parse_package_spec(<<~PRAYSPEC).canonicalized
      Package::Specification.new do |spec|
        spec.name = "fork/base"
        spec.version = "1.0.0"
        spec.summary = "forked guidance"
        spec.files = ["README.md", "exports/a.md"]
        spec.exports = {
          "a" => {
            type: "fragment",
            path: "exports/a.md",
            summary: "A"
          }
        }
        spec.upstream "sample/base", "~> 1.4"
      end
    PRAYSPEC

    rendered = described_class.render_package_spec(spec)
    parsed = Pray.parse_package_spec(rendered).canonicalized
    expect(parsed.name).to eq("fork/base")
    expect(parsed.upstream.name).to eq("sample/base")
    expect(parsed.upstream.constraint).to eq("~> 1.4")
    expect(parsed.exports.fetch("a").path).to eq("exports/a.md")
  end

  it "round-trips every supported field" do
    spec = Pray.parse_package_spec(<<~PRAYSPEC).canonicalized
      Package::Specification.new do |spec|
        spec.name = "fork/base"
        spec.version = "1.0.0"
        spec.summary = "summary"
        spec.description = "description"
        spec.authors = ["Author"]
        spec.license = "MIT"
        spec.homepage = "https://example.com"
        spec.source_code_uri = "https://example.com/source"
        spec.changelog_uri = "https://example.com/changelog"
        spec.prayfile_version = "1"
        spec.files = ["a.md", "fork.prayspec"]
        spec.exports = {
          "a" => {
            type: "fragment",
            path: "a.md",
            summary: "A",
            only: ["tool_a"],
            except: ["tool_b"],
            default_path: "A.md"
          }
        }
        spec.skills = {
          "review" => { path: "skills/review", summary: "Review" }
        }
        spec.templates = {
          "note" => { path: "templates/note.md", summary: "Note" }
        }
        spec.adapters = { "tool" => "adapter" }
        spec.targets = ["tool_a"]
        spec.add_optional_dependency "sample/common", "~> 1.0"
        spec.metadata = { "labels" => ["stable", :tool-a], "policy" => { "enabled" => true, "fallback" => nil }, "priority" => 1 }
        spec.upstream "sample/base", "~> 1.4"
      end
    PRAYSPEC

    parsed = Pray.parse_package_spec(described_class.render_package_spec(spec)).canonicalized
    expect(parsed).to eq(spec)
  end

  it "keeps local-only content files after a dirty refresh" do
    local = Pray.parse_package_spec(<<~PRAYSPEC)
      Package::Specification.new do |spec|
        spec.name = "fork/base"
        spec.version = "1.0.0"
        spec.files = ["a.md", "extra.md"]
        spec.upstream "sample/base", "~> 1.4"
      end
    PRAYSPEC
    upstream = Pray.parse_package_spec(<<~PRAYSPEC)
      Package::Specification.new do |spec|
        spec.name = "sample/base"
        spec.version = "1.4.4"
        spec.files = ["a.md"]
      end
    PRAYSPEC

    refreshed = described_class.fork_spec_after_refresh(
      local, upstream, "fork.prayspec", false, ["a.md", "extra.md"]
    )
    expect(refreshed.files).to include("fork.prayspec", "a.md", "extra.md")
  end
end
