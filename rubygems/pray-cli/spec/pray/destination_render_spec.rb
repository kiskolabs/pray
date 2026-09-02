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

  it "preserves unmanaged text around an existing managed span" do
    root = Dir.mktmpdir("pray-compose-preserve-local-")
    begin
      write_package(
        root, "rules", "sample/rules", "rules", "fragment", "exports/rules.md", "Old managed text\n"
      )
      File.write(
        File.join(root, "Prayfile"),
        <<~PRAYFILE
          prayfile "1"
          compose "AGENTS.md" do
            pray "sample/rules", "~> 1.0", path: "packages/rules"
          end
        PRAYFILE
      )
      project = Pray::Resolve.resolve_project(File.join(root, "Prayfile"))
      Pray::Render.write_rendered_targets(project, Pray::Render.render_project(project))
      destination = File.join(root, "AGENTS.md")
      File.write(destination, "#{File.read(destination)}\nLocal text must survive.\n")
      File.write(File.join(root, "packages/rules/exports/rules.md"), "New managed text\n")

      project = Pray::Resolve.resolve_project(File.join(root, "Prayfile"))
      Pray::Render.write_rendered_targets(project, Pray::Render.render_project(project))
      content = File.read(destination)
      expect(content).to include("Local text must survive.")
      expect(content).to include("New managed text")
      expect(content).not_to include("Old managed text")
    ensure
      FileUtils.rm_rf(root)
    end
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

  it "inlines a utf-8 file export into compose as a marked span" do
    root = Dir.mktmpdir("pray-compose-file-")
    begin
      write_package(
        root, "community", "sample/community", "contributing", "file",
        "exports/CONTRIBUTING.md", "Be kind.\n"
      )
      File.write(
        File.join(root, "Prayfile"),
        <<~PRAYFILE
          prayfile "1"
          compose "CONTRIBUTING.md" do
            pray "sample/community", "~> 1.0", path: "packages/community"
          end
        PRAYFILE
      )
      project = Pray::Resolve.resolve_project(File.join(root, "Prayfile"))
      rendered = Pray::Render.render_project(project)
      expect(rendered[0].content).to include("<!-- pray:")
      expect(rendered[0].content).to include("Be kind")
      expect(rendered[0].content).not_to include("# Agent context")
    ensure
      FileUtils.rm_rf(root)
    end
  end

  it "fails closed on compose of JSON" do
    root = Dir.mktmpdir("pray-compose-json-")
    begin
      write_package(
        root, "rules", "sample/rules", "rules", "fragment", "exports/rules.md", "Keep it small.\n"
      )
      File.write(
        File.join(root, "Prayfile"),
        <<~PRAYFILE
          prayfile "1"
          compose "config.json" do
            pray "sample/rules", "~> 1.0", path: "packages/rules"
          end
        PRAYFILE
      )
      project = Pray::Resolve.resolve_project(File.join(root, "Prayfile"))
      expect { Pray::Render.render_project(project) }.to raise_error(
        Pray::Error, /JSON.*file: "config.json"/m
      )
    ensure
      FileUtils.rm_rf(root)
    end
  end

  it "keeps exclusive file destinations unmarked" do
    root = Dir.mktmpdir("pray-exclusive-file-")
    begin
      write_package(
        root, "community", "sample/community", "contributing", "file",
        "exports/CONTRIBUTING.md", "Be kind.\n"
      )
      File.write(
        File.join(root, "Prayfile"),
        <<~PRAYFILE
          prayfile "1"
          pray "sample/community", "~> 1.0", path: "packages/community", file: "CONTRIBUTING.md"
        PRAYFILE
      )
      project = Pray::Resolve.resolve_project(File.join(root, "Prayfile"))
      rendered = Pray::Render.render_project(project)
      expect(rendered).to be_empty
      Pray::Render.write_rendered_targets(project, rendered)
      dest = File.read(File.join(root, "CONTRIBUTING.md"))
      expect(dest).to eq("Be kind.\n")
      expect(dest).not_to include("<!-- pray:")
    ensure
      FileUtils.rm_rf(root)
    end
  end

  it "prefers a fragment when a file export also exists" do
    root = Dir.mktmpdir("pray-compose-prefer-fragment-")
    begin
      package_root = File.join(root, "packages/mixed")
      FileUtils.mkdir_p(File.join(package_root, "exports"))
      File.write(
        File.join(package_root, "mixed.prayspec"),
        <<~SPEC
          Package::Specification.new do |spec|
            spec.name = "sample/mixed"
            spec.version = "1.0.0"
            spec.summary = "fixture"
            spec.files = ["exports/notes.md", "exports/CONTRIBUTING.md"]
            spec.exports = {
              "notes" => { type: "fragment", path: "exports/notes.md" },
              "contributing" => { type: "file", path: "exports/CONTRIBUTING.md" }
            }
          end
        SPEC
      )
      File.write(File.join(package_root, "exports/notes.md"), "Fragment notes\n")
      File.write(File.join(package_root, "exports/CONTRIBUTING.md"), "File contributing\n")
      File.write(
        File.join(root, "Prayfile"),
        <<~PRAYFILE
          prayfile "1"
          compose "AGENTS.md" do
            pray "sample/mixed", "~> 1.0", path: "packages/mixed"
          end
        PRAYFILE
      )
      rendered = Pray::Render.render_project(
        Pray::Resolve.resolve_project(File.join(root, "Prayfile"))
      )
      expect(rendered[0].content).to include("Fragment notes")
      expect(rendered[0].content).not_to include("File contributing")
      expect(rendered[0].content).to include("# Agent context")
      expect(rendered[0].content).to include(".agents/")
    ensure
      FileUtils.rm_rf(root)
    end
  end

  it "fails compose of a binary file export" do
    root = Dir.mktmpdir("pray-compose-binary-")
    begin
      write_package(
        root, "blob", "sample/blob", "icon", "file",
        "exports/icon.md", [0xff, 0xfe, 0x00].pack("C*")
      )
      File.write(
        File.join(root, "Prayfile"),
        <<~PRAYFILE
          prayfile "1"
          compose "ICON.md" do
            pray "sample/blob", "~> 1.0", path: "packages/blob"
          end
        PRAYFILE
      )
      project = Pray::Resolve.resolve_project(File.join(root, "Prayfile"))
      expect { Pray::Render.render_project(project) }.to raise_error(
        Pray::Error, /binary|utf-8/
      )
    ensure
      FileUtils.rm_rf(root)
    end
  end

  it "fails closed on compose of an unknown type" do
    root = Dir.mktmpdir("pray-compose-unknown-")
    begin
      write_package(
        root, "rules", "sample/rules", "rules", "fragment",
        "exports/rules.md", "Keep it small.\n"
      )
      File.write(
        File.join(root, "Prayfile"),
        <<~PRAYFILE
          prayfile "1"
          compose ".zshrc" do
            pray "sample/rules", "~> 1.0", path: "packages/rules"
          end
        PRAYFILE
      )
      project = Pray::Resolve.resolve_project(File.join(root, "Prayfile"))
      expect { Pray::Render.render_project(project) }.to raise_error(
        Pray::Error, /file: "\.zshrc"/
      )
    ensure
      FileUtils.rm_rf(root)
    end
  end

  it "suppresses the Agent banner when compose sets header: false" do
    root = Dir.mktmpdir("pray-compose-header-off-")
    begin
      write_package(
        root, "rules", "sample/rules", "rules", "fragment",
        "exports/rules.md", "Keep it small.\n"
      )
      File.write(
        File.join(root, "Prayfile"),
        <<~PRAYFILE
          prayfile "1"
          compose "AGENTS.md", header: false do
            pray "sample/rules", "~> 1.0", path: "packages/rules"
          end
        PRAYFILE
      )
      rendered = Pray::Render.render_project(
        Pray::Resolve.resolve_project(File.join(root, "Prayfile"))
      )
      expect(rendered[0].content).not_to include("# Agent context")
    ensure
      FileUtils.rm_rf(root)
    end
  end

  it "omits .agents from a forced banner on NOTES.md" do
    root = Dir.mktmpdir("pray-compose-header-on-")
    begin
      write_package(
        root, "rules", "sample/rules", "rules", "fragment",
        "exports/rules.md", "Keep it small.\n"
      )
      File.write(
        File.join(root, "Prayfile"),
        <<~PRAYFILE
          prayfile "1"
          compose "NOTES.md", header: true do
            pray "sample/rules", "~> 1.0", path: "packages/rules"
          end
        PRAYFILE
      )
      rendered = Pray::Render.render_project(
        Pray::Resolve.resolve_project(File.join(root, "Prayfile"))
      )
      expect(rendered[0].content).to include("# Agent context")
      expect(rendered[0].content).not_to include(".agents/")
    ensure
      FileUtils.rm_rf(root)
    end
  end

  it "does not match unused export kinds to a destination role" do
    %w[template command rule asset bundle].each do |kind|
      %w[fragment folder file].each do |role|
        expect(Pray::Destination.export_kind_matches_role?(kind, role)).to be(false)
      end
    end
  end

  it "parses spec.adapters and does not load them" do
    spec = Pray.parse_package_spec(<<~SPEC)
      Package::Specification.new do |spec|
        spec.name = "sample/with-adapters"
        spec.version = "1.0.0"
        spec.files = ["exports/a.md"]
        spec.exports = { "a" => { type: "fragment", path: "exports/a.md" } }
        spec.adapters = { "tool_a" => "adapters/tool_a.toml" }
      end
    SPEC
    expect(spec.adapters["tool_a"]).to eq("adapters/tool_a.toml")
  end
end
