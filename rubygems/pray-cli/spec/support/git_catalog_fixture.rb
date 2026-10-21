# frozen_string_literal: true

require "fileutils"
require "open3"

module GitCatalogFixture
  module_function

  def pinned_shallow_cache(workspace)
    origin = File.join(workspace, "origin")
    FileUtils.mkdir_p(File.join(origin, "v1/packages"))
    File.write(File.join(origin, "v1/packages/sample.json"), "{}\n")
    run_git(origin, "init", "--template=", "-b", "main")
    run_git(origin, "config", "user.name", "pray")
    run_git(origin, "config", "user.email", "pray@example.com")
    run_git(origin, "config", "uploadpack.allowReachableSHA1InWant", "true")
    commit_all(origin, "one")
    File.write(File.join(origin, "middle.txt"), "pin\n")
    commit_all(origin, "middle")
    pinned = run_git(origin, "rev-parse", "HEAD").strip
    File.write(File.join(origin, "later.txt"), "two\n")
    commit_all(origin, "two")
    clone_url = "file://#{origin}"
    db = Pray::GitStore.global_git_cache_directory(clone_url)
    FileUtils.mkdir_p(File.dirname(db))
    run_git(workspace, "clone", "--bare", "--depth", "1", "--no-local", clone_url, db)
    if system("git", "-C", db, "cat-file", "-e", pinned, out: File::NULL, err: File::NULL)
      raise "fixture cache must lack the pin #{pinned}"
    end

    {root: workspace, origin: origin, clone_url: clone_url, pinned: pinned, db: db}
  end

  def commit_all(directory, message)
    run_git(directory, "add", "-A")
    run_git(directory, "-c", "commit.gpgsign=false", "-c", "core.hooksPath=/dev/null", "commit", "-m", message)
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
