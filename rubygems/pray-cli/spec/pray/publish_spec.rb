# frozen_string_literal: true

require "spec_helper"
require "fileutils"
require "json"

RSpec.describe Pray::Publish do
  let(:workspace) { Dir.mktmpdir("pray-publish-") }
  let(:simple_project) { File.expand_path("../../../../examples/simple-project", __dir__) }

  after do
    FileUtils.rm_rf(workspace)
  end

  it "publishes resolved packages to a local distribution root" do
    project_dir = File.join(workspace, "project")
    publish_root = File.join(workspace, "dist")
    FileUtils.cp_r(simple_project, project_dir)

    project = Pray::Resolve.resolve_project(File.join(project_dir, "Prayfile"))
    begin
      described_class.publish_to_root(project, publish_root)
    rescue Pray::Error => error
      skip error.message if error.message.include?("zstd")
      raise
    end

    index = JSON.parse(File.read(File.join(publish_root, "v1", "index.json")))
    expect(index["packages"]).to include("sample/base")

    metadata_path = File.join(publish_root, "v1", "packages", "sample/base.json")
    expect(File).to exist(metadata_path)
    metadata = JSON.parse(File.read(metadata_path))
    expect(metadata["versions"].first["version"]).to eq("1.4.3")
  end

  it "preserves publish metadata until package content changes" do
    project_dir = File.join(workspace, "project")
    publish_root = File.join(workspace, "dist")
    FileUtils.cp_r(simple_project, project_dir)
    project = Pray::Resolve.resolve_project(File.join(project_dir, "Prayfile"))

    described_class.publish_to_root(project, publish_root)
    metadata_path = File.join(publish_root, "v1", "packages", "sample", "base.json")
    metadata = JSON.parse(File.read(metadata_path))
    metadata["versions"].first["published_at"] = "2020-01-01T00:00:00.999Z"
    metadata["versions"].first["yanked"] = true
    File.write(metadata_path, JSON.pretty_generate(metadata))

    described_class.publish_to_root(project, publish_root)
    unchanged_metadata = JSON.parse(File.read(metadata_path))
    unchanged = unchanged_metadata["versions"].first
    expect(unchanged["published_at"]).to eq(1_577_836_800)
    expect(unchanged["yanked"]).to be(true)
    described_class.publish_to_root(project, publish_root)
    expect(JSON.parse(File.read(metadata_path))).to eq(unchanged_metadata)

    described_class.publish_to_root(project, publish_root, signer: "replacement")
    resigned = JSON.parse(File.read(metadata_path))["versions"].first
    expect(resigned["signer"]).to eq("replacement")
    expect(resigned["published_at"]).to eq(1_577_836_800)
    expect(resigned["yanked"]).to be(true)

    prayspec_path = File.join(project_dir, "packages", "base", "sample-base.prayspec")
    File.write(prayspec_path, File.read(prayspec_path).sub("small guidance bundle", "revised guidance bundle"))
    specification_project = Pray::Resolve.resolve_project(File.join(project_dir, "Prayfile"))
    described_class.publish_to_root(specification_project, publish_root)
    specification_changed = JSON.parse(File.read(metadata_path))["versions"].first
    expect(specification_changed["artifact_hash"]).not_to eq(resigned["artifact_hash"])
    expect(specification_changed["published_at"]).not_to eq(1_577_836_800)
    expect(specification_changed["yanked"]).to be(true)

    File.write(File.join(project_dir, "packages", "base", "README.md"), "Changed package\n")
    changed_project = Pray::Resolve.resolve_project(File.join(project_dir, "Prayfile"))
    described_class.publish_to_root(changed_project, publish_root)
    changed = JSON.parse(File.read(metadata_path))["versions"].first
    expect(changed["artifact_hash"]).not_to eq(specification_changed["artifact_hash"])
  end
end
