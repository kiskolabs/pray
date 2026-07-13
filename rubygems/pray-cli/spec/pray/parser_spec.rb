# frozen_string_literal: true

require "spec_helper"

RSpec.describe "Pray parser" do
  it "parses minimal manifest example" do
    manifest = Pray.parse_manifest(<<~PRAYFILE)
      prayfile "1"
      source "default", "https://agents.example.com"
      target :tool_a do
        output "INSTRUCTIONS.md"
        skills ".agents/skills"
      end
      agent "sample/base", "~> 1.4",
        exports: ["testing-basics", "security-basics"]
      local ".agents/project.md"
      render mode: :managed,
        conflict: :fail,
        churn: :minimal
    PRAYFILE

    expect(manifest.prayfile_version).to eq("1")
    expect(manifest.sources.first.name).to eq("default")
    expect(manifest.targets.first.name).to eq("tool_a")
    expect(manifest.targets.first.outputs).to eq(["INSTRUCTIONS.md"])
    expect(manifest.packages.first.name).to eq("sample/base")
    expect(manifest.local.first.path).to eq(".agents/project.md")
    expect(manifest.render.mode).to eq("managed")
  end

  it "parses minimal package spec example" do
    package = Pray.parse_package_spec(<<~SPEC)
      Package::Specification.new do |spec|
        spec.name = "sample/base"
        spec.version = "1.4.3"
        spec.summary = "shared guidance"
        spec.files = ["README.md", "exports/testing-basics.md"]
        spec.exports = {
          "testing-basics" => {
            type: "fragment",
            path: "exports/testing-basics.md",
            summary: "Testing guidance"
          }
        }
        spec.add_dependency "sample/common", "~> 1.0"
      end
    SPEC

    expect(package.name).to eq("sample/base")
    expect(package.version).to eq("1.4.3")
    expect(package.files).to eq(["README.md", "exports/testing-basics.md"])
    expect(package.exports["testing-basics"].path).to eq("exports/testing-basics.md")
    expect(package.dependencies.first.name).to eq("sample/common")
  end

  it "preserves package declaration order" do
    manifest = Pray.parse_manifest(<<~PRAYFILE)
      prayfile "1"
      agent "sample/zebra", "~> 1.0"
      agent "sample/alpha", "~> 1.0"
      agent "sample/middle", "~> 1.0"
    PRAYFILE

    expect(manifest.packages.map(&:name)).to eq(%w[sample/zebra sample/alpha sample/middle])
  end

  it "parses git source keyword form" do
    manifest = Pray.parse_manifest(<<~PRAYFILE)
      prayfile "1"
      source "amkisko", git: "https://github.com/amkisko/prayers"
      agent "amkisko/working-rules", "~> 1.0", source: "amkisko"
    PRAYFILE

    expect(manifest.sources.length).to eq(1)
    expect(manifest.sources.first.name).to eq("amkisko")
    expect(manifest.sources.first.kind).to eq("git")
    expect(manifest.sources.first.url).to eq("git+https://github.com/amkisko/prayers")
  end

  it "parses git source subdir keyword" do
    manifest = Pray.parse_manifest(<<~PRAYFILE)
      prayfile "1"
      source "dist", git: "https://github.com/example/prayers", subdir: "prayers"
    PRAYFILE

    expect(manifest.sources.first.subdir).to eq("prayers")
  end

  it "parses git source distribution alias" do
    manifest = Pray.parse_manifest(<<~PRAYFILE)
      prayfile "1"
      source "amkisko", git: "https://github.com/amkisko/prayers", distribution: "prayers/v1"
    PRAYFILE

    expect(manifest.sources.first.subdir).to eq("prayers/v1")
  end

  it "parses git source rev and tag" do
    manifest = Pray.parse_manifest(<<~PRAYFILE)
      prayfile "1"
      source "pinned", git: "https://github.com/example/prayers", rev: "abc123def456"
      source "tagged", git: "https://github.com/example/prayers", tag: "v1.0.0"
    PRAYFILE

    expect(manifest.sources[0].rev).to eq("abc123def456")
    expect(manifest.sources[0].tag).to be_nil
    expect(manifest.sources[1].tag).to eq("v1.0.0")
    expect(manifest.sources[1].rev).to be_nil
  end

  it "treats bare package version as exact pin" do
    manifest = Pray.parse_manifest(<<~PRAYFILE)
      prayfile "1"
      agent "sample/base", "1.0.0"
    PRAYFILE

    expect(manifest.packages.first.constraint).to eq("=1.0.0")
  end

  it "round-trips package declaration through formatter" do
    manifest = Pray.parse_manifest(<<~PRAYFILE)
      prayfile "1"
      agent "sample/base", "~> 1.0", source: "amkisko", exports: ["testing-basics", "security-basics"]
    PRAYFILE

    formatted = Pray.format_package_declaration(manifest.packages.first)
    expect(formatted).to eq(
      'agent "sample/base", "~> 1.0", source: "amkisko", exports: ["testing-basics", "security-basics"]'
    )
    reparsed = Pray.parse_manifest("prayfile \"1\"\n#{formatted}\n")
    expect(reparsed.packages.first).to eq(manifest.packages.first)
  end

  it "parses pray ssh source url" do
    manifest = Pray.parse_manifest(<<~PRAYFILE)
      prayfile "1"
      source "team", "pray+ssh://pray@prayers.internal:2222/var/lib/pray"
      agent "sample/base", "1.0.0", source: :team
    PRAYFILE

    expect(manifest.sources.length).to eq(1)
    expect(manifest.sources.first.name).to eq("team")
    expect(manifest.sources.first.kind).to eq("pray_ssh")
    expect(manifest.sources.first.url).to eq("pray+ssh://pray@prayers.internal:2222/var/lib/pray")
  end

  it "rejects manifest without prayfile version" do
    expect do
      Pray.parse_manifest(<<~PRAYFILE)
        target :tool_a do
          output "INSTRUCTIONS.md"
        end
      PRAYFILE
    end.to raise_error(Pray::Error) { |error|
      expect(error.category).to eq(:manifest)
      expect(error.message).to include("missing prayfile version")
    }
  end

  it "rejects package spec without end" do
    expect do
      Pray.parse_package_spec(<<~SPEC)
        Package::Specification.new do |spec|
          spec.name = "sample/base"
      SPEC
    end.to raise_error(Pray::Error) { |error|
      expect(error.category).to eq(:parse)
      expect(error.message).to include("missing 'end'")
    }
  end

  it "matches Rust manifest hash for simple-project" do
    prayfile = File.read(File.expand_path("../../../../examples/simple-project/Prayfile", __dir__))
    manifest = Pray.parse_manifest(prayfile)
    expect(manifest.manifest_hash).to eq(
      "sha256:340c8fc15fa0196aadea58a50834d4f726698fb74a4967cdc340e3e653950326"
    )
  end
end
