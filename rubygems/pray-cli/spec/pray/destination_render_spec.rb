# frozen_string_literal: true

require "spec_helper"
require "fileutils"

RSpec.describe "destination render" do
  def write_package(root, directory, package_name, export_name, export_kind, export_path, body)
    package_root = File.join(root, "packages", directory)
    FileUtils.mkdir_p(File.join(package_root, File.dirname(export_path)))
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
              summary: "#{export_name}"
            }
          }
        end
      SPEC
    )
    File.write(File.join(package_root, export_path), body)
  end

  it "still fans out fragments and skills for the legacy shape" do
    root = Dir.mktmpdir("pray-legacy-fanout-")
    begin
      FileUtils.mkdir_p(File.join(root, ".agents"))
      write_package(
        root, "rules", "sample/rules", "rules", "fragment", "exports/rules.md", "Legacy rules\n"
      )
      FileUtils.mkdir_p(File.join(root, "packages/audit/skills/audit"))
      File.write(File.join(root, "packages/audit/skills/audit/SKILL.md"), "# Audit skill\n")
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
      File.write(File.join(root, ".agents/project.md"), "Local note\n")
      File.write(
        File.join(root, "Prayfile"),
        <<~PRAYFILE
          prayfile "1"
          target :tool_a do
            output "INSTRUCTIONS.md"
            skills ".agents/skills"
          end
          agent "sample/rules", "~> 1.0", path: "packages/rules"
          agent "sample/audit", "~> 1.0", path: "packages/audit"
          local ".agents/project.md"
        PRAYFILE
      )

      project = Pray::Resolve.resolve_project(File.join(root, "Prayfile"))
      rendered = Pray::Render.render_project(project)
      expect(rendered.length).to eq(1)
      expect(rendered[0].content).to include("Legacy rules")
      expect(rendered[0].content).to include("Local note")
      expect(rendered[0].content).to include("## Shared instructions")

      planned = Pray::Render.planned_provisioned_files(project)
      expect(planned.map(&:path)).to include(a_string_ending_with(".agents/skills/audit/SKILL.md"))
    ensure
      FileUtils.rm_rf(root)
    end
  end

  it "isolates compose, tree, and file bindings" do
    root = Dir.mktmpdir("pray-destination-dsl-")
    begin
      FileUtils.mkdir_p(File.join(root, ".agents"))
      write_package(
        root, "rules", "sample/rules", "rules", "fragment", "exports/rules.md", "Compose rules\n"
      )
      write_package(
        root, "unbound", "sample/unbound", "unbound", "fragment", "exports/unbound.md",
        "Should not appear\n"
      )
      FileUtils.mkdir_p(File.join(root, "packages/audit/skills/audit"))
      File.write(File.join(root, "packages/audit/skills/audit/SKILL.md"), "# Audit skill\n")
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
      FileUtils.mkdir_p(File.join(root, "packages/security/exports"))
      File.write(
        File.join(root, "packages/security/exports/SECURITY.md"),
        "# Security Policy\n\nEmail: ((pray:security_email))\n"
      )
      File.write(
        File.join(root, "packages/security/security.prayspec"),
        <<~SPEC
          Package::Specification.new do |spec|
            spec.name = "sample/security"
            spec.version = "1.0.0"
            spec.summary = "fixture"
            spec.files = ["exports/SECURITY.md"]
            spec.exports = {
              "security" => {
                type: "file",
                path: "exports/SECURITY.md",
                default_path: "SECURITY.md"
              }
            }
          end
        SPEC
      )
      File.write(File.join(root, ".agents/project.md"), "Local first\n")
      File.write(
        File.join(root, "Prayfile"),
        <<~PRAYFILE
          prayfile "1"
          pray do
            security_email "security@example.com"
          end
          compose "AGENTS.md" do
            pray ".agents/project.md"
            pray "sample/rules", "~> 1.0", path: "packages/rules"
          end
          tree ".agents/skills" do
            pray "sample/audit", "~> 1.0", path: "packages/audit"
          end
          pray "sample/security", "~> 1.0", path: "packages/security", file: "SECURITY.md"
          agent "sample/unbound", "~> 1.0", path: "packages/unbound"
        PRAYFILE
      )

      project = Pray::Resolve.resolve_project(File.join(root, "Prayfile"))
      rendered = Pray::Render.render_project(project)
      expect(rendered.length).to eq(1)
      content = rendered[0].content
      expect(content).to include("Local first")
      expect(content).to include("Compose rules")
      expect(content).not_to include("Should not appear")
      expect(content).not_to include("## Shared instructions")

      planned = Pray::Render.planned_provisioned_files(project)
      expect(planned.map(&:path)).to include("SECURITY.md")
      expect(planned.map(&:path)).to include(a_string_ending_with(".agents/skills/audit/SKILL.md"))
      expect(planned.map(&:path).grep(/security\/SECURITY\.md/)).to be_empty

      Pray::Render.write_rendered_targets(project, rendered)
      security = File.read(File.join(root, "SECURITY.md"))
      expect(security).to eq("# Security Policy\n\nEmail: security@example.com\n")
    ensure
      FileUtils.rm_rf(root)
    end
  end

  it "limits the provisioned tree with a folder export only filter" do
    root = Dir.mktmpdir("pray-folder-only-")
    begin
      FileUtils.mkdir_p(File.join(root, "packages/templates/templates"))
      File.write(File.join(root, "packages/templates/templates/issue.md"), "issue\n")
      File.write(File.join(root, "packages/templates/templates/pr.md"), "pr\n")
      File.write(File.join(root, "packages/templates/templates/draft.md"), "draft\n")
      File.write(
        File.join(root, "packages/templates/templates.prayspec"),
        <<~SPEC
          Package::Specification.new do |spec|
            spec.name = "sample/templates"
            spec.version = "1.0.0"
            spec.summary = "fixture"
            spec.files = ["templates/issue.md", "templates/pr.md", "templates/draft.md"]
            spec.exports = {
              "templates" => {
                type: "folder",
                path: "templates",
                only: ["issue.md", "pr.md"]
              }
            }
          end
        SPEC
      )
      File.write(
        File.join(root, "Prayfile"),
        <<~PRAYFILE
          prayfile "1"
          tree ".agents/templates" do
            pray "sample/templates", "~> 1.0", path: "packages/templates"
          end
        PRAYFILE
      )

      project = Pray::Resolve.resolve_project(File.join(root, "Prayfile"))
      planned = Pray::Render.planned_provisioned_files(project)
      paths = planned.map(&:path)
      expect(paths).to include(a_string_ending_with("issue.md"))
      expect(paths).to include(a_string_ending_with("pr.md"))
      expect(paths.grep(/draft\.md\z/)).to be_empty
    ensure
      FileUtils.rm_rf(root)
    end
  end

  it "parses singular export and alias keywords for resolution" do
    root = Dir.mktmpdir("pray-export-alias-")
    begin
      write_package(
        root, "rules", "sample/rules", "rules", "fragment", "exports/rules.md", "Alias rules\n"
      )
      File.write(
        File.join(root, "Prayfile"),
        <<~PRAYFILE
          prayfile "1"
          compose "AGENTS.md" do
            include "sample/rules", "~> 1.0", path: "packages/rules", export: "rules"
          end
        PRAYFILE
      )

      project = Pray::Resolve.resolve_project(File.join(root, "Prayfile"))
      expect(project.packages[0].selected_exports).to eq(["rules"])
      rendered = Pray::Render.render_project(project)
      expect(rendered[0].content).to include("Alias rules")
    ensure
      FileUtils.rm_rf(root)
    end
  end
end
