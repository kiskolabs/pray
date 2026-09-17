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

  it "gives different cache directories to the same URL with different subdirs" do
    left = described_class.git_source_cache_directory(workspace, "file://repo", "left")
    right = described_class.git_source_cache_directory(workspace, "file://repo", "right")
    shared = described_class.git_source_cache_directory(workspace, "file://repo")
    expect(left).not_to eq(right)
    expect(left).not_to eq(shared)
    expect(right).not_to eq(shared)
  end

  it "finds a leftover subdir checkout when the URL-only cache is missing" do
    clone_url = "file://repo-from-prayfile"
    leftover = described_class.git_source_cache_directory(workspace, clone_url, "left")
    FileUtils.mkdir_p(leftover)
    system("git", "init", "-b", "main", leftover, out: File::NULL, err: File::NULL)
    system("git", "-C", leftover, "remote", "add", "origin", clone_url, out: File::NULL, err: File::NULL)
    expect(described_class.git_source_cached_repository(workspace, clone_url)).to eq(leftover)
  end
end
