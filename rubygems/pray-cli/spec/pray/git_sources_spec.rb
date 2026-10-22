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

  it "ignores a leftover full clone when the shared bare is missing" do
    clone_url = "file://repo-from-prayfile"
    leftover = described_class.git_source_cache_directory(workspace, clone_url, "left")
    FileUtils.mkdir_p(leftover)
    system("git", "init", "-b", "main", leftover, out: File::NULL, err: File::NULL)
    system("git", "-C", leftover, "remote", "add", "origin", clone_url, out: File::NULL, err: File::NULL)
    expect(described_class.git_source_cached_repository(workspace, clone_url)).to be_nil
  end

  it "prefers a stub .pray-git-dir over a leftover full clone" do
    clone_url = "file://repo-from-prayfile"
    previous_cache = ENV["PRAY_CACHE"]
    ENV["PRAY_CACHE"] = File.join(workspace, "global-cache")
    begin
      shared = Pray::GitStore.global_git_cache_directory(clone_url)
      FileUtils.mkdir_p(shared)
      system("git", "init", "--bare", "-b", "main", shared, out: File::NULL, err: File::NULL)
      leftover = described_class.git_source_cache_directory(workspace, clone_url, "left")
      FileUtils.mkdir_p(File.join(leftover, ".git"))
      stub = described_class.git_source_cache_directory(workspace, clone_url)
      FileUtils.mkdir_p(stub)
      File.write(File.join(stub, ".pray-git-dir"), shared)
      expect(described_class.git_source_cached_repository(workspace, clone_url)).to eq(shared)
    ensure
      if previous_cache
        ENV["PRAY_CACHE"] = previous_cache
      else
        ENV.delete("PRAY_CACHE")
      end
    end
  end

  it "rejects a stub .pray-git-dir outside the expected global bare" do
    clone_url = "file://repo-from-prayfile"
    previous_cache = ENV["PRAY_CACHE"]
    ENV["PRAY_CACHE"] = File.join(workspace, "global-cache")
    begin
      foreign = File.join(workspace, "foreign-bare")
      FileUtils.mkdir_p(foreign)
      system("git", "init", "--bare", "-b", "main", foreign, out: File::NULL, err: File::NULL)
      stub = described_class.git_source_cache_directory(workspace, clone_url)
      FileUtils.mkdir_p(stub)
      File.write(File.join(stub, ".pray-git-dir"), foreign)
      expect(described_class.git_source_cached_repository(workspace, clone_url)).to be_nil
    ensure
      if previous_cache
        ENV["PRAY_CACHE"] = previous_cache
      else
        ENV.delete("PRAY_CACHE")
      end
    end
  end
end
