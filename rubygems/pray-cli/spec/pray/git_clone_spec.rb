# frozen_string_literal: true

require "spec_helper"
require "fileutils"
require "open3"

RSpec.describe "git blobless catalog clone" do
  let(:workspace) { Dir.mktmpdir("pray-git-blobless-") }
  let(:unused_bytes) { 128 * 1024 }

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

  it "keeps unused artifacts out of the clone and materializes a used file" do
    origin = File.join(workspace, "origin")
    FileUtils.mkdir_p(File.join(origin, "v1/packages/sample"))
    FileUtils.mkdir_p(File.join(origin, "v1/artifacts/sample/base/1.0.0"))
    FileUtils.mkdir_p(File.join(origin, "v1/artifacts/sample/heavy/1.0.0"))
    File.write(File.join(origin, "v1/packages/sample/base.json"), "{}\n")
    used = "v1/artifacts/sample/base/1.0.0/sample-base-1.0.0.praypkg"
    unused = "v1/artifacts/sample/heavy/1.0.0/sample-heavy-1.0.0.praypkg"
    File.write(File.join(origin, used), "used-package\n")
    File.binwrite(File.join(origin, unused), "\x07" * unused_bytes)
    git(origin, "init", "--template=", "-b", "main")
    git(origin, "config", "user.name", "pray")
    git(origin, "config", "user.email", "pray@example.com")
    git(origin, "config", "uploadpack.allowFilter", "true")
    git(origin, "add", "-A")
    git(origin, "-c", "commit.gpgsign=false", "-c", "core.hooksPath=/dev/null", "commit", "-m", "catalog")

    cache, = Pray::GitCache.ensure_git_repository(
      workspace,
      "file://#{origin}",
      refresh: false,
      pinned_revision: nil,
      sparse_subdir: nil
    )
    expect(File).not_to exist(File.join(cache, ".git"))
    expect(File).not_to exist(File.join(cache, unused))
    Pray::GitMaterialize.materialize_catalog_file(cache, used)
    expect(File).to exist(File.join(cache, used))
    expect(directory_bytes(cache)).to be < unused_bytes
  end

  def git(directory, *arguments)
    env = {
      "GIT_AUTHOR_NAME" => "pray",
      "GIT_AUTHOR_EMAIL" => "pray@example.com",
      "GIT_COMMITTER_NAME" => "pray",
      "GIT_COMMITTER_EMAIL" => "pray@example.com"
    }
    output, status = Open3.capture2e(env, "git", "-C", directory, *arguments)
    expect(status).to be_success, "git #{arguments.join(" ")} failed: #{output}"
  end

  def directory_bytes(path)
    total = 0
    Dir.glob(File.join(path, "**", "*"), File::FNM_DOTMATCH).each do |entry|
      next unless File.file?(entry)

      total += File.size(entry)
    end
    total
  end
end
