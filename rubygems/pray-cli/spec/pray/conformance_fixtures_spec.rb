# frozen_string_literal: true

require "spec_helper"
require "json"

RSpec.describe "RFC 0100 conformance fixtures" do
  FIXTURES = File.expand_path("../../../../fixtures", __dir__)

  it "parses the minimal Prayfile fixture" do
    dir = File.join(FIXTURES, "parser", "minimal-prayfile")
    expected = JSON.parse(File.read(File.join(dir, "expected.json")))
    manifest = Pray.parse_manifest(File.read(File.join(dir, "Prayfile")))

    expect(manifest.prayfile_version).to eq(expected["prayfile_version"])
    expect(manifest.packages.map(&:name)).to eq(expected["package_names"])
    expect(manifest.targets.map(&:name)).to eq(expected["target_names"])
  end

  it "parses the minimal prayspec fixture" do
    dir = File.join(FIXTURES, "prayspec", "minimal-package")
    expected = JSON.parse(File.read(File.join(dir, "expected.json")))
    spec = Pray.parse_package_spec(File.read(File.join(dir, "sample-base.prayspec")))

    expect(spec.name).to eq(expected["name"])
    expect(spec.version).to eq(expected["version"])
    expect(spec.files).to eq(expected["files"])
  end

  it "parses the minimal lockfile fixture" do
    dir = File.join(FIXTURES, "lockfile", "minimal")
    expected = JSON.parse(File.read(File.join(dir, "expected.json")))
    lockfile = Pray.parse_lockfile(File.read(File.join(dir, "Prayfile.lock")))

    expect(lockfile.prayfile_lock).to eq(expected["prayfile_lock"])
    expect(lockfile.spec).to eq(expected["spec"])
    expect(lockfile.package.map(&:name)).to eq(expected["package_names"])
    expect(lockfile.managed_span.map(&:id)).to eq(expected["managed_span_ids"])
  end

  it "rejects the invalid lockfile fixture" do
    text = File.read(File.join(FIXTURES, "lockfile", "invalid-toml", "Prayfile.lock"))
    expect { Pray.parse_lockfile(text) }.to raise_error(Pray::Error)
  end

  it "renders the compose fragment fixture to the expected dest" do
    dir = File.join(FIXTURES, "render", "compose-fragment")
    expected = JSON.parse(File.read(File.join(dir, "expected.json")))
    project = Pray.resolve_project(File.join(dir, "Prayfile"), offline: true)
    rendered = Pray.render_project(project)

    expect(project.packages.map { |package| package.declaration.name }).to eq(expected["package_names"])
    expect(rendered.length).to eq(1)
    expect(rendered.first.path).to eq(expected["dest_path"])
    dest = File.read(File.join(dir, "expected", expected["dest_path"]))
    expect(rendered.first.content).to eq(dest)
  end

  %w[span-matching span-edited span-missing-dest span-removed span-orphan].each do |pack|
    it "inspects locked destinations for #{pack}" do
      dir = File.join(FIXTURES, "lockfile", pack)
      expected = JSON.parse(File.read(File.join(dir, "expected.json")))
      lockfile = Pray.parse_lockfile(File.read(File.join(dir, "Prayfile.lock")))
      report = Pray.inspect_locked_destinations(dir, lockfile)
      kinds = report.findings.map(&:kind).sort
      expect(kinds).to eq(expected["finding_kinds"].sort)
    end
  end
end
