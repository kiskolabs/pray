# frozen_string_literal: true

require "spec_helper"

RSpec.describe "Pray library API" do
  it "loads core library without CLI" do
    library_root = File.expand_path("../../lib", __dir__)
    output = `ruby -I#{library_root} -e 'require "pray"; puts defined?(Pray::CLI) || "none"'`
    expect(output.strip).to eq("none")
  end

  it "parses lockfile text without reading from disk" do
    lockfile = Pray.parse_lockfile(<<~LOCKFILE)
      prayfile_lock = "1"
      spec = "0.1"
      generated_by = "pray test"
      manifest_hash = "sha256:abc"
      source = []
      package = []
      target = []
      managed_span = []
    LOCKFILE

    expect(lockfile.prayfile_lock).to eq("1")
    expect(lockfile.manifest_hash).to eq("sha256:abc")
    expect(Pray.serialize_lockfile(lockfile)).to include('manifest_hash = "sha256:abc"')
    expect(Pray.lockfile_hash(lockfile)).to start_with("sha256:")
  end

  it "round-trips lockfile text through parse and serialize" do
    original = Pray::Lockfile.new(
      manifest_hash: "sha256:roundtrip",
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
      ]
    )
    serialized = Pray.serialize_lockfile(original)
    restored = Pray.parse_lockfile(serialized)

    expect(restored.manifest_hash).to eq(original.manifest_hash)
    expect(restored.package.first.name).to eq("sample/base")
    expect(Pray.lockfiles_equivalent?(original, restored)).to be(true)
  end
end
