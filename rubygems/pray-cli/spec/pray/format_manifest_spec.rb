# frozen_string_literal: true

require "spec_helper"
require "fileutils"
require "tmpdir"

RSpec.describe Pray::FormatManifest do
  def write_package(root, directory, package_name, export_name, export_kind, export_path, body, default_path: nil)
    package_root = File.join(root, "packages", directory)
    FileUtils.mkdir_p(File.join(package_root, File.dirname(export_path)))
    default_path_literal = default_path ? ",\n      default_path: \"#{default_path}\"" : ""
    File.write(
      File.join(package_root, "#{directory}.prayspec"),
      <<~SPEC
        Package::Specification.new do |spec|
          spec.name = "#{package_name}"
          spec.version = "1.0.0"
          spec.summary = "fixture"
          spec.files = ["#{export_path}"]
          spec.exports = {
            "#{export_name}" => {
              type: "#{export_kind}",
              path: "#{export_path}",
              summary: "#{export_name}"#{default_path_literal}
            }
          }
        end
      SPEC
    )
    File.write(File.join(package_root, export_path), body)
  end

  def write_audit_package(root)
    FileUtils.mkdir_p(File.join(root, "packages/audit/skills/audit"))
    File.write(File.join(root, "packages/audit/skills/audit/SKILL.md"), "# Audit\n")
    File.write(
      File.join(root, "packages/audit/audit.prayspec"),
      <<~SPEC
        Package::Specification.new do |spec|
          spec.name = "sample/audit"
          spec.version = "1.0.0"
          spec.summary = "fixture"
          spec.files = ["skills/audit/SKILL.md"]
          spec.exports = {
            "audit" => {
              type: "skill",
              path: "skills/audit",
              summary: "audit"
            }
          }
        end
      SPEC
    )
  end

  it "formats a legacy Prayfile into the recommended destination DSL" do
    Dir.mktmpdir("pray-format-legacy-") do |root|
      FileUtils.mkdir_p(File.join(root, ".agents"))
      write_package(root, "rules", "sample/rules", "rules", "fragment", "exports/rules.md", "Rules\n")
      write_audit_package(root)
      write_package(
        root, "security", "sample/security", "security", "file", "exports/SECURITY.md", "# Security\n",
        default_path: "SECURITY.md"
      )
      File.write(File.join(root, ".agents/project.md"), "Local\n")

      original = <<~PRAYFILE
        prayfile "1"
        target :tool_a do
          output "AGENTS.md"
          skills ".agents/skills"
        end
        agent "sample/rules", "~> 1.0", path: "packages/rules"
        agent "sample/audit", "~> 1.0", path: "packages/audit"
        agent "sample/security", "~> 1.0", path: "packages/security"
        local ".agents/project.md", position: :before
      PRAYFILE
      File.write(File.join(root, "Prayfile"), original)

      project = Pray::Resolve.resolve_project(File.join(root, "Prayfile"))
      hints = described_class.classify_format_hints(project)
      manifest = Pray.parse_manifest(original)
      expect(described_class.uses_destination_dsl?(manifest)).to be(false)

      formatted = described_class.format_recommended(manifest, hints)
      expect(formatted).to include('compose "AGENTS.md" do')
      expect(formatted).to include('pray ".agents/project.md"')
      expect(formatted).to include('pray "sample/rules"')
      expect(formatted).to include('tree ".agents/skills" do')
      expect(formatted).to include('pray "sample/audit"')
      expect(formatted).to include('file: "SECURITY.md"')
      expect(formatted).not_to include("target :tool_a")
      expect(formatted).not_to include('agent "')

      reparsed = Pray.parse_manifest(formatted)
      expect(described_class.uses_destination_dsl?(reparsed)).to be(true)
      expect(reparsed.targets[0].mode).to eq("compose")
      expect(reparsed.targets[1].mode).to eq("tree")
      security = reparsed.packages.find { |entry| entry.name == "sample/security" }
      expect(security.file).to eq("SECURITY.md")
      expect(security.roles).to include("file")

      again = described_class.format_recommended(reparsed, hints)
      expect(again).to eq(formatted)
    end
  end

  it "formats a legacy Prayfile that already has file: bindings" do
    Dir.mktmpdir("pray-format-hybrid-") do |root|
      FileUtils.mkdir_p(File.join(root, ".agents"))
      write_package(root, "rules", "sample/rules", "rules", "fragment", "exports/rules.md", "Rules\n")
      write_audit_package(root)
      write_package(
        root, "security", "sample/security", "security", "file", "exports/SECURITY.md", "# Security\n",
        default_path: "SECURITY.md"
      )
      File.write(File.join(root, ".agents/project.md"), "Local\n")

      original = <<~PRAYFILE
        prayfile "1"
        target :tool_a do
          output "AGENTS.md"
          skills ".agents/skills"
        end
        agent "sample/rules", "~> 1.0", path: "packages/rules"
        agent "sample/audit", "~> 1.0", path: "packages/audit"
        pray "sample/security", "~> 1.0", path: "packages/security", file: "SECURITY.md"
        local ".agents/project.md", position: :before
      PRAYFILE
      File.write(File.join(root, "Prayfile"), original)

      project = Pray::Resolve.resolve_project(File.join(root, "Prayfile"))
      hints = described_class.classify_format_hints(project)
      manifest = Pray.parse_manifest(original)
      expect(described_class.uses_destination_dsl?(manifest)).to be(true)

      formatted = described_class.format_recommended(manifest, hints)
      expect(formatted).to include('compose "AGENTS.md" do')
      expect(formatted).to include('tree ".agents/skills" do')
      expect(formatted).to include('file: "SECURITY.md"')
      expect(formatted).not_to include("target :tool_a")
    end
  end

  it "omits source keyword when namespace matches a source handle" do
    manifest = Pray.parse_manifest(<<~PRAYFILE)
      prayfile "1"
      source "amkisko", path: "packages/amkisko"
      source "other", path: "packages/other"
      compose "AGENTS.md" do
        pray "amkisko/rules", "~> 1.0", source: "amkisko"
        pray "other/notes", "~> 1.0", source: "other"
      end
    PRAYFILE
    formatted = described_class.format_recommended(manifest, {})
    expect(formatted).to include('pray "amkisko/rules", "~> 1.0"')
    expect(formatted).to include('pray "other/notes", "~> 1.0"')
    expect(formatted).not_to include('source: "amkisko"')
    expect(formatted).not_to include('source: "other"')
  end

  it "formats an existing destination DSL manifest idempotently" do
    Dir.mktmpdir("pray-format-dsl-") do |root|
      FileUtils.mkdir_p(File.join(root, ".agents"))
      write_package(root, "rules", "sample/rules", "rules", "fragment", "exports/rules.md", "Rules\n")
      File.write(File.join(root, ".agents/project.md"), "Local\n")

      original = <<~PRAYFILE
        prayfile "1"
        compose "AGENTS.md" do
          pray ".agents/project.md"
          pray "sample/rules", "~> 1.0", path: "packages/rules"
        end
      PRAYFILE
      File.write(File.join(root, "Prayfile"), original)

      project = Pray::Resolve.resolve_project(File.join(root, "Prayfile"))
      hints = described_class.classify_format_hints(project)
      manifest = Pray.parse_manifest(original)
      formatted = described_class.format_recommended(manifest, hints)
      again = described_class.format_recommended(Pray.parse_manifest(formatted), hints)
      expect(formatted).to eq(again)
      expect(formatted).to include('compose "AGENTS.md" do')
    end
  end

  it "classifies package roles from hints when recommending a manifest" do
    manifest = Pray.parse_manifest(<<~PRAYFILE)
      prayfile "1"
      target :tool_a do
        output "AGENTS.md"
        skills ".agents/skills"
      end
      agent "sample/rules", "~> 1.0", path: "packages/rules"
      agent "sample/audit", "~> 1.0", path: "packages/audit"
    PRAYFILE
    hints = {
      "sample/rules" => Pray::PackageFormatHint.new(roles: ["fragment"]),
      "sample/audit" => Pray::PackageFormatHint.new(roles: ["folder"])
    }

    recommended = described_class.recommend_manifest(manifest, hints)
    expect(recommended.targets.length).to eq(2)
    expect(recommended.targets[0].entries.any? { |entry| entry.kind == "package" && entry.name == "sample/rules" }).to be(true)
    expect(recommended.targets[1].entries.any? { |entry| entry.kind == "package" && entry.name == "sample/audit" }).to be(true)
  end
end
