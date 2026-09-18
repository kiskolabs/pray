# frozen_string_literal: true

require "spec_helper"
require "fileutils"
require "open3"

RSpec.describe "pinned git revision fetch" do
  let(:workspace) { Dir.mktmpdir("pray-pinned-git-") }

  around do |example|
    previous_cache = ENV["PRAY_CACHE"]
    ENV["PRAY_CACHE"] = File.join(workspace, "global-cache")
    example.run
  ensure
    if previous_cache.nil?
      ENV.delete("PRAY_CACHE")
    else
      ENV["PRAY_CACHE"] = previous_cache
    end
    FileUtils.rm_rf(workspace)
  end

  it "fetches a locked revision missing from a shallow cache" do
    fixture = pinned_shallow_cache
    _cache, revision = Pray::GitCache.ensure_git_repository(
      fixture.fetch(:root),
      fixture.fetch(:clone_url),
      refresh: false,
      pinned_revision: fixture.fetch(:pinned),
      sparse_subdir: nil
    )
    expect(revision).to eq(fixture.fetch(:pinned))
    parent_status = system(
      "git",
      "-C",
      fixture.fetch(:db),
      "cat-file",
      "-e",
      "#{fixture.fetch(:pinned)}^",
      out: File::NULL,
      err: File::NULL
    )
    expect(parent_status).to be(true)
  end

  it "refuses that fetch when offline" do
    fixture = pinned_shallow_cache
    expect do
      Pray::GitCache.ensure_git_repository(
        fixture.fetch(:root),
        fixture.fetch(:clone_url),
        refresh: false,
        pinned_revision: fixture.fetch(:pinned),
        sparse_subdir: nil,
        offline: true
      )
    end.to raise_error(Pray::Error) { |error|
      expect(error.message).to include(fixture.fetch(:pinned))
      expect(error.message).to include("offline")
      expect(error.message).not_to include("--locked")
    }
  end

  it "refuses to clone when offline and the cache is missing" do
    origin = File.join(workspace, "origin")
    FileUtils.mkdir_p(origin)
    File.write(File.join(origin, "catalog.txt"), "one\n")
    run_git(origin, "init", "--template=", "-b", "main")
    run_git(origin, "config", "user.name", "pray")
    run_git(origin, "config", "user.email", "pray@example.com")
    run_git(origin, "add", "-A")
    run_git(origin, "-c", "commit.gpgsign=false", "-c", "core.hooksPath=/dev/null", "commit", "-m", "one")
    clone_url = "file://#{origin}"
    expect do
      Pray::GitCache.ensure_git_repository(
        workspace,
        clone_url,
        refresh: false,
        pinned_revision: nil,
        sparse_subdir: nil,
        offline: true
      )
    end.to raise_error(Pray::Error) { |error|
      expect(error.message).to include("offline")
      expect(error.message).to include("not cached")
    }
    expect(File.directory?(File.join(Pray::GitCache.git_source_cache_directory(workspace, clone_url), ".git"))).to be(false)
  end

  it "seeds from the global cache when offline and the origin is gone" do
    origin = File.join(workspace, "origin")
    FileUtils.mkdir_p(File.join(origin, "v1/packages"))
    File.write(File.join(origin, "v1/packages/sample.json"), "{}\n")
    run_git(origin, "init", "--template=", "-b", "main")
    run_git(origin, "config", "user.name", "pray")
    run_git(origin, "config", "user.email", "pray@example.com")
    run_git(origin, "add", "-A")
    run_git(origin, "-c", "commit.gpgsign=false", "-c", "core.hooksPath=/dev/null", "commit", "-m", "one")
    clone_url = "file://#{origin}"
    _cache, revision = Pray::GitCache.ensure_git_repository(
      workspace,
      clone_url,
      refresh: false,
      pinned_revision: nil,
      sparse_subdir: nil
    )
    FileUtils.rm_rf(Pray::GitCache.git_source_cache_directory(workspace, clone_url))
    FileUtils.mv(origin, File.join(workspace, "origin-away"))
    _seeded, seeded_revision = Pray::GitCache.ensure_git_repository(
      workspace,
      clone_url,
      refresh: false,
      pinned_revision: nil,
      sparse_subdir: nil,
      offline: true
    )
    expect(seeded_revision).to eq(revision)
  end

  def pinned_shallow_cache
    origin = File.join(workspace, "origin")
    FileUtils.mkdir_p(File.join(origin, "v1/packages"))
    File.write(File.join(origin, "v1/packages/sample.json"), "{}\n")
    run_git(origin, "init", "--template=", "-b", "main")
    run_git(origin, "config", "user.name", "pray")
    run_git(origin, "config", "user.email", "pray@example.com")
    run_git(origin, "config", "uploadpack.allowReachableSHA1InWant", "true")
    run_git(origin, "add", "-A")
    run_git(origin, "-c", "commit.gpgsign=false", "-c", "core.hooksPath=/dev/null", "commit", "-m", "one")
    File.write(File.join(origin, "middle.txt"), "pin\n")
    run_git(origin, "add", "-A")
    run_git(origin, "-c", "commit.gpgsign=false", "-c", "core.hooksPath=/dev/null", "commit", "-m", "middle")
    pinned = run_git(origin, "rev-parse", "HEAD").strip
    File.write(File.join(origin, "later.txt"), "two\n")
    run_git(origin, "add", "-A")
    run_git(origin, "-c", "commit.gpgsign=false", "-c", "core.hooksPath=/dev/null", "commit", "-m", "two")
    clone_url = "file://#{origin}"
    db = Pray::GitStore.global_git_cache_directory(clone_url)
    FileUtils.mkdir_p(File.dirname(db))
    run_git(workspace, "clone", "--bare", "--depth", "1", "--no-local", clone_url, db)
    status = system("git", "-C", db, "cat-file", "-e", pinned, out: File::NULL, err: File::NULL)
    expect(status).to be_falsey
    {root: workspace, clone_url: clone_url, pinned: pinned, db: db}
  end

  def run_git(directory, *arguments)
    env = {
      "GIT_AUTHOR_NAME" => "pray",
      "GIT_AUTHOR_EMAIL" => "pray@example.com",
      "GIT_COMMITTER_NAME" => "pray",
      "GIT_COMMITTER_EMAIL" => "pray@example.com"
    }
    output, status = Open3.capture2e(env, "git", "-C", directory, *arguments)
    raise "git #{arguments.join(" ")} failed: #{output}" unless status.success?

    output
  end
end
