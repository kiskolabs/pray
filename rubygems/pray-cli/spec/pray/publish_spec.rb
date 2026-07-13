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
end
