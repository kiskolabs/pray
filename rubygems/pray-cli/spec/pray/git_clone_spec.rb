# frozen_string_literal: true

require "spec_helper"
require "fileutils"
require "open3"

RSpec.describe "git blobless catalog clone" do
  let(:workspace) { Dir.mktmpdir("pray-git-blobless-") }
  let(:unused_bytes) { 128 * 1024 }

  after do
    FileUtils.rm_rf(workspace)
  end

  it "keeps unused artifacts out of the clone and materializes a used file" do
    origin = File.join(workspace, "origin")
    clone = File.join(workspace, "clone")
    FileUtils.mkdir_p(File.join(origin, "v1/packages/sample"))
    FileUtils.mkdir_p(File.join(origin, "v1/artifacts/sample/base/1.0.0"))
    FileUtils.mkdir_p(File.join(origin, "v1/artifacts/sample/heavy/1.0.0"))
    File.write(File.join(origin, "v1/packages/sample/base.json"), "{}\n")
    used = "v1/artifacts/sample/base/1.0.0/sample-base-1.0.0.praypkg"
    unused = "v1/artifacts/sample/heavy/1.0.0/sample-heavy-1.0.0.praypkg"
    File.write(File.join(origin, used), "used-package\n")
    File.binwrite(File.join(origin, unused), "\x07" * unused_bytes)
    git(origin, "init", "-b", "main")
    git(origin, "config", "user.name", "pray")
    git(origin, "config", "user.email", "pray@example.com")
    git(origin, "config", "uploadpack.allowFilter", "true")
    git(origin, "add", "-A")
    git(origin, "-c", "commit.gpgsign=false", "-c", "core.hooksPath=/dev/null", "commit", "-m", "catalog")

    Pray::GitClone.clone_git_cache(workspace, "file://#{origin}", clone, quiet: true)
    Pray::GitClone.apply_sparse_checkout(clone)
    expect(File).not_to exist(File.join(clone, unused))
    Pray::GitMaterialize.materialize_catalog_file(clone, used)
    expect(File).to exist(File.join(clone, used))
    expect(directory_bytes(clone)).to be < unused_bytes
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
