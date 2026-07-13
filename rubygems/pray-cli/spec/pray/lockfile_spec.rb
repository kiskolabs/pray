# frozen_string_literal: true

require "spec_helper"
require "fileutils"

RSpec.describe Pray::LockfileIO do
  let(:workspace) { Dir.mktmpdir("pray-lockfile-") }
  let(:lockfile_path) { File.join(workspace, "Prayfile.lock") }

  after do
    FileUtils.rm_rf(workspace)
  end

  def sample_lockfile
    Pray::Lockfile.new(
      manifest_hash: "sha256:abc",
      package: [
        Pray::LockedPackage.new(
          name: "sample/base",
          version: "1.4.3",
          path: "packages/base",
          tree_hash: "sha256:tree",
          artifact_hash: "sha256:artifact",
          artifact: "sample-base-1.4.3.praypkg",
          exports: ["testing-basics"]
        )
      ],
      target: [Pray::LockedTarget.new(name: "tool_a", outputs: ["INSTRUCTIONS.md"])]
    )
  end

  it "serializes fixture lockfile in canonical pretty format" do
    fixture_path = File.expand_path("../../../../examples/simple-project/Prayfile.lock", __dir__)
    expected = File.read(fixture_path)
    lockfile = Pray.parse_lockfile(expected)

    expect(Pray.serialize_lockfile(lockfile)).to eq(expected)
  end

  it "round-trips lockfile serialization" do
    original = sample_lockfile
    Pray.write_lockfile(lockfile_path, original)
    restored = Pray.read_lockfile(lockfile_path)

    expect(restored.manifest_hash).to eq(original.manifest_hash)
    expect(restored.package.first.name).to eq("sample/base")
    expect(restored.package.first.exports).to eq(["testing-basics"])
    expect(restored.target.first.outputs).to eq(["INSTRUCTIONS.md"])
  end

  it "skips rewriting unchanged lockfiles" do
    lockfile = sample_lockfile
    Pray.write_lockfile(lockfile_path, lockfile)
    bytes = File.binread(lockfile_path)

    Pray.write_lockfile_if_changed(lockfile_path, lockfile)
    expect(File.binread(lockfile_path)).to eq(bytes)
  end
end
