# frozen_string_literal: true

require "spec_helper"
require "fileutils"

RSpec.describe Pray::GitSources do
  let(:workspace) { Dir.mktmpdir("pray-git-") }

  after do
    FileUtils.rm_rf(workspace)
  end

  it "discovers distribution root at repository root" do
    distribution = File.join(workspace, "dist")
    FileUtils.mkdir_p(File.join(distribution, "v1", "packages"))
    expect(described_class.discover_distribution_root(distribution)).to eq(distribution)
  end

  it "discovers distribution root under prayers/" do
    repo = File.join(workspace, "repo")
    prayers = File.join(repo, "prayers")
    FileUtils.mkdir_p(File.join(prayers, "v1", "packages"))
    expect(described_class.discover_distribution_root(repo)).to eq(prayers)
  end

  it "uses pinned lockfile revision for git sources" do
    lockfile = Pray::Lockfile.new(
      source: [
        Pray::LockSource.new(
          name: "dist",
          kind: "git",
          url: "git+https://example.com/prayers.git",
          revision: "abc123"
        )
      ]
    )
    source = Pray::ManifestSource.new(
      name: "dist",
      kind: "git",
      url: "git+https://example.com/prayers.git"
    )
    expect(described_class.pinned_revision_for_source(lockfile, source)).to eq("abc123")
  end
end
