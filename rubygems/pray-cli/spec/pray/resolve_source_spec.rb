# frozen_string_literal: true

require "spec_helper"
require "fileutils"

RSpec.describe Pray::ResolveSource do
  describe ".implied_source_name" do
    let(:sources) do
      {
        "amkisko" => Pray::ManifestSource.new("amkisko", "path", "packages", nil, nil, nil),
        "other" => Pray::ManifestSource.new("other", "path", "packages/other", nil, nil, nil)
      }
    end

    it "uses an explicit source keyword" do
      declaration = Pray::ManifestPackage.new(name: "amkisko/rules", constraint: "1.0.0", source: "other")
      expect(described_class.implied_source_name(declaration, sources)).to eq("other")
    end

    it "matches the package namespace to a source handle" do
      declaration = Pray::ManifestPackage.new(name: "amkisko/rules", constraint: "1.0.0")
      expect(described_class.implied_source_name(declaration, sources)).to eq("amkisko")
    end

    it "uses the sole source when present" do
      sole = {"sample" => sources["amkisko"]}
      declaration = Pray::ManifestPackage.new(name: "sample/rules", constraint: "1.0.0")
      expect(described_class.implied_source_name(declaration, sole)).to eq("sample")
    end

    it "uses the unique path source for an unqualified name" do
      catalog = {
        "amkisko" => Pray::ManifestSource.new("amkisko", "git", "git+https://example.com/prayers.git", nil, nil, nil),
        "local" => Pray::ManifestSource.new("local", "path", "prayers", nil, nil, nil)
      }
      declaration = Pray::ManifestPackage.new(name: "project", constraint: "*")
      expect(described_class.implied_source_name(declaration, catalog)).to eq("local")
    end

    it "still matches a git namespace when a path source exists" do
      catalog = {
        "amkisko" => Pray::ManifestSource.new("amkisko", "git", "git+https://example.com/prayers.git", nil, nil, nil),
        "local" => Pray::ManifestSource.new("local", "path", "prayers", nil, nil, nil)
      }
      declaration = Pray::ManifestPackage.new(name: "amkisko/rules", constraint: "*")
      expect(described_class.implied_source_name(declaration, catalog)).to eq("amkisko")
    end

    it "uses a namespaced name to pick a path source when several exist" do
      catalog = {
        "local" => Pray::ManifestSource.new("local", "path", "prayers", nil, nil, nil),
        "vendor" => Pray::ManifestSource.new("vendor", "path", "vendor", nil, nil, nil)
      }
      declaration = Pray::ManifestPackage.new(name: "local/project", constraint: "*")
      expect(described_class.implied_source_name(declaration, catalog)).to eq("local")
    end

    it "requires source when multiple sources do not match the namespace" do
      declaration = Pray::ManifestPackage.new(name: "third/rules", constraint: "1.0.0")
      expect do
        described_class.implied_source_name(declaration, sources)
      end.to raise_error(Pray::Error, /requires source:/)
    end
  end

  it "resolves a path package from a namespace-matching source without source:" do
    Dir.mktmpdir("pray-implied-source-") do |root|
      package_dir = File.join(root, "packages", "rules")
      FileUtils.mkdir_p(package_dir)
      File.write(
        File.join(package_dir, "rules.prayspec"),
        <<~PRAYSPEC
          Package::Specification.new do |spec|
            spec.name = "amkisko/rules"
            spec.version = "1.0.0"
            spec.files = ["exports/rules.md"]
            spec.exports = {"rules" => {"path" => "exports/rules.md", "kind" => "fragment"}}
          end
        PRAYSPEC
      )
      FileUtils.mkdir_p(File.join(package_dir, "exports"))
      File.write(File.join(package_dir, "exports/rules.md"), "Rules\n")
      FileUtils.mkdir_p(File.join(root, "packages/other"))
      File.write(
        File.join(root, "Prayfile"),
        <<~PRAYFILE
          prayfile "1"
          source "amkisko", path: "packages"
          source "other", path: "packages/other"
          compose "AGENTS.md" do
            pray "amkisko/rules", "~> 1.0"
          end
        PRAYFILE
      )

      project = Pray::Resolve.resolve_project(File.join(root, "Prayfile"))
      expect(project.packages.length).to eq(1)
      expect(project.packages.first.declaration.name).to eq("amkisko/rules")
      expect(project.packages.first.root).to eq(package_dir)
    end
  end
end
