# frozen_string_literal: true

require "spec_helper"
require "fileutils"

RSpec.describe "git commands started from a hook" do
  let(:workspace) { Dir.mktmpdir("pray-git-host-") }
  let(:fixture) { GitCatalogFixture.pinned_shallow_cache(workspace) }

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

  it "leaves the host repository alone when GIT_DIR is set" do
    expect_host_untouched_with { |host| {"GIT_DIR" => host.fetch(:git_dir)} }
  end

  it "leaves the host index alone when GIT_INDEX_FILE is set" do
    expect_host_untouched_with { |host| {"GIT_INDEX_FILE" => File.join(host.fetch(:git_dir), "index")} }
  end

  it "leaves the host repository alone when GIT_WORK_TREE is set" do
    expect_host_untouched_with do |host|
      {
        "GIT_DIR" => host.fetch(:git_dir),
        "GIT_WORK_TREE" => host.fetch(:path),
        "GIT_INDEX_FILE" => File.join(host.fetch(:git_dir), "index")
      }
    end
  end

  it "leaves the host HEAD and branches alone on refresh" do
    host = host_repository
    before = host_state(host)
    _cache, revision = with_caller_env("GIT_DIR" => host.fetch(:git_dir)) do
      ensure_repository(fixture.fetch(:clone_url), refresh: true, pinned_revision: nil)
    end
    expect(host_changes(host, before)).to eq([])
    expect(revision).to eq(GitCatalogFixture.run_git(fixture.fetch(:origin), "rev-parse", "HEAD").strip)
  end

  it "applies git config the caller passes in the environment" do
    alias_url = "file:///pray-caller-config-alias"
    caller_config = {
      "GIT_CONFIG_COUNT" => "1",
      "GIT_CONFIG_KEY_0" => "url.#{fixture.fetch(:clone_url)}.insteadOf",
      "GIT_CONFIG_VALUE_0" => alias_url
    }
    _cache, revision = with_caller_env(caller_config) do
      ensure_repository(alias_url, refresh: false, pinned_revision: nil)
    end
    expect(revision).to eq(GitCatalogFixture.run_git(fixture.fetch(:origin), "rev-parse", "HEAD").strip)
  end

  it "repairs a broken cache without touching the host repository" do
    host_remote = "https://example.com/host.git"
    File.write(File.join(workspace, "README.md"), "host\n")
    GitCatalogFixture.run_git(workspace, "init", "--template=", "-b", "main")
    GitCatalogFixture.run_git(workspace, "remote", "add", "origin", host_remote)
    FileUtils.rm_rf(File.join(fixture.fetch(:db), "objects"))
    _cache, revision = ensure_repository(
      fixture.fetch(:clone_url),
      refresh: false,
      pinned_revision: fixture.fetch(:pinned)
    )
    expect(File).to exist(File.join(fixture.fetch(:db), "HEAD"))
    expect(File).to be_directory(File.join(fixture.fetch(:db), "objects"))
    expect(revision).to eq(fixture.fetch(:pinned))
    expect(GitCatalogFixture.run_git(workspace, "remote", "get-url", "origin").strip).to eq(host_remote)
  end

  def expect_host_untouched_with
    host = host_repository
    before = host_state(host)
    _cache, revision = with_caller_env(yield(host)) do
      ensure_repository(fixture.fetch(:clone_url), refresh: false, pinned_revision: fixture.fetch(:pinned))
    end
    expect(host_changes(host, before)).to eq([])
    expect(revision).to eq(fixture.fetch(:pinned))
  end

  def ensure_repository(clone_url, refresh:, pinned_revision:)
    Pray::GitCache.ensure_git_repository(
      workspace,
      clone_url,
      refresh: refresh,
      pinned_revision: pinned_revision,
      sparse_subdir: nil
    )
  end

  def with_caller_env(variables)
    previous = variables.keys.to_h { |name| [name, ENV[name]] }
    variables.each { |name, value| ENV[name] = value }
    yield
  ensure
    previous.each { |name, value| ENV[name] = value }
  end

  def host_repository
    remote = File.join(fixture.fetch(:root), "host-remote.git")
    GitCatalogFixture.run_git(fixture.fetch(:root), "init", "--bare", "--template=", remote)
    path = File.join(fixture.fetch(:root), "host")
    FileUtils.mkdir_p(path)
    File.write(File.join(path, "README.md"), "host\n")
    GitCatalogFixture.run_git(path, "init", "--template=", "-b", "main")
    GitCatalogFixture.commit_all(path, "host")
    GitCatalogFixture.run_git(path, "remote", "add", "origin", remote)
    GitCatalogFixture.run_git(path, "push", "origin", "main")
    GitCatalogFixture.run_git(path, "fetch", "origin")
    {path: path, git_dir: File.join(path, ".git")}
  end

  def host_state(host)
    {
      "origin URL" => GitCatalogFixture.run_git(host.fetch(:path), "remote", "get-url", "origin").strip,
      "HEAD or refs" => GitCatalogFixture.run_git(host.fetch(:path), "show-ref", "--head").strip,
      "index" => File.binread(File.join(host.fetch(:git_dir), "index"))
    }
  end

  def host_changes(host, before)
    after = host_state(host)
    before.keys.reject { |label| before[label] == after[label] }
  end
end
