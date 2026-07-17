# frozen_string_literal: true

require "spec_helper"

RSpec.describe Pray::Render do
  let(:simple_project) { File.expand_path("../../../../examples/simple-project", __dir__) }

  it "renders managed spans for selected exports" do
    project = Pray::Resolve.resolve_project(File.join(simple_project, "Prayfile"))
    rendered = described_class.render_project(project)
    instructions = rendered.find { |target| target.path == "INSTRUCTIONS.md" }

    expect(instructions).not_to be_nil
    expect(instructions.content).to include("<!-- pray:")
    expect(instructions.content).to include("## Shared instructions")
    expect(instructions.managed_spans.map(&:export)).to include("testing-basics", "security-basics")
  end

  it "records checksums for managed spans" do
    project = Pray::Resolve.resolve_project(File.join(simple_project, "Prayfile"))
    rendered = described_class.render_project(project)
    span = rendered.flat_map(&:managed_spans).find { |entry| entry.export == "testing-basics" }

    expect(span.ideal_checksum).to start_with("sha256:")
    expect(span.package).to eq("sample/base")
  end

  it "filters rendered packages by environment groups" do
    manifest = Pray::Manifest.new(
      prayfile_version: "1",
      targets: [
        Pray::ManifestTarget.new(name: "tool_a", outputs: ["INSTRUCTIONS.md"])
      ],
      packages: [
        Pray::ManifestPackage.new(name: "sample/base", groups: []),
        Pray::ManifestPackage.new(name: "sample/dev", groups: ["development"])
      ]
    )
    base_package = Pray::ResolvedPackage.new(
      declaration: manifest.packages[0],
      root: simple_project,
      spec: Pray::PackageSpec.new(name: "sample/base", version: "1.0.0", exports: {}),
      tree_hash: "sha256:base",
      artifact_hash: "sha256:base",
      selected_exports: %w[testing-basics],
      export_bodies: {"testing-basics" => "base export"}
    )
    dev_package = Pray::ResolvedPackage.new(
      declaration: manifest.packages[1],
      root: simple_project,
      spec: Pray::PackageSpec.new(name: "sample/dev", version: "1.0.0", exports: {}),
      tree_hash: "sha256:dev",
      artifact_hash: "sha256:dev",
      selected_exports: %w[dev-only],
      export_bodies: {"dev-only" => "dev export"}
    )
    project = Pray::ResolvedProject.new(
      manifest_path: File.join(simple_project, "Prayfile"),
      project_root: simple_project,
      manifest: manifest,
      manifest_hash: "sha256:test",
      packages: [base_package, dev_package],
      local_files: [],
      source_revisions: {},
      source_host_keys: {},
      environment: "development"
    )

    content = described_class.render_target(project, manifest.targets.first, "INSTRUCTIONS.md").content

    expect(content).to include("base export")
    expect(content).to include("dev export")

    production_project = project.dup
    production_project.environment = "production"
    production_content = described_class.render_target(
      production_project,
      manifest.targets.first,
      "INSTRUCTIONS.md"
    ).content

    expect(production_content).to include("base export")
    expect(production_content).not_to include("dev export")
  end
end
