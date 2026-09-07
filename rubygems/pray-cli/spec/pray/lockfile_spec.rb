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

  it "round-trips provisioned lock records" do
    lockfile = sample_lockfile
    lockfile.provisioned = [
      Pray::ProvisionedFileRecord.new(
        path: ".zshrc",
        content_hash: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        package: "sample/shell",
        export: "zshrc"
      )
    ]
    Pray.write_lockfile(lockfile_path, lockfile)
    restored = Pray.read_lockfile(lockfile_path)

    expect(restored.provisioned.first.path).to eq(".zshrc")
    expect(restored.provisioned.first.package).to eq("sample/shell")
    expect(Pray.serialize_lockfile(restored)).to include("[[provisioned]]")
  end

  it "skips rewriting unchanged lockfiles" do
    lockfile = sample_lockfile
    Pray.write_lockfile(lockfile_path, lockfile)
    bytes = File.binread(lockfile_path)

    Pray.write_lockfile_if_changed(lockfile_path, lockfile)
    expect(File.binread(lockfile_path)).to eq(bytes)
  end

  it "reads quoted keys, Unicode paths, comments and multiline export arrays" do
    lockfile = Pray.parse_lockfile(<<~'TOML')
      "manifest_hash" = 'sha256:original' # retained provenance
      [[package]]
      name = "sample/notes"
      exports = [
        "notes",
        "shell", # a trailing comma is valid
      ]
      [[provisioned]]
      path = "out/\u00e9.txt"
      content_hash = "sha256:content"
      package = "sample/notes"
      export = "notes"
    TOML

    expect(lockfile.manifest_hash).to eq("sha256:original")
    expect(lockfile.package.first.exports).to eq(%w[notes shell])
    expect(lockfile.provisioned.first.path).to eq("out/é.txt")
  end

  it "reports duplicate and malformed fields as lockfile parse errors" do
    ["manifest_hash = 'one'\nmanifest_hash = 'two'", "package = [", 'path = "\\q"'].each do |text|
      expect { Pray.parse_lockfile(text) }.to raise_error(Pray::Error, /lockfile/)
    end
  end
end
