# frozen_string_literal: true

require "spec_helper"
require "fileutils"
require_relative "../support/git_distribution_fixture"

RSpec.describe "global git cache install" do
  let(:workspace) { Dir.mktmpdir("pray-global-git-cache-") }
  let(:global_cache) { File.join(workspace, "global-cache") }

  around do |example|
    previous_cache = ENV["PRAY_CACHE"]
    ENV["PRAY_CACHE"] = global_cache
    example.run
  ensure
    if previous_cache.nil?
      ENV.delete("PRAY_CACHE")
    else
      ENV["PRAY_CACHE"] = previous_cache
    end
  end

  after do
    FileUtils.rm_rf(workspace)
  end

  it "refreshes the stale global mirror when constraints require newer versions" do
    source_repo = File.join(workspace, "source")
    distribution_repo = File.join(workspace, "distribution")
    prayers_root = File.join(distribution_repo, "prayers")
    first_consumer = File.join(workspace, "first-consumer")
    second_consumer = File.join(workspace, "second-consumer")

    FileUtils.mkdir_p(source_repo)
    FileUtils.mkdir_p(distribution_repo)
    FileUtils.mkdir_p(first_consumer)
    FileUtils.mkdir_p(second_consumer)

    GitDistributionFixture.create_add_fixture(source_repo)
    Dir.chdir(source_repo) do
      GitDistributionFixture.publish_source_to_prayers(source_repo, prayers_root)
    end
    GitDistributionFixture.init_distribution_repo(distribution_repo, prayers_root)

    GitDistributionFixture.write_consumer_prayfile(first_consumer, distribution_repo)
    Dir.chdir(first_consumer) do
      Pray::CLI.run(["install"])
    end
    expect(File.directory?(File.join(global_cache, "git"))).to be(true)

    File.write(
      File.join(source_repo, "packages/base/sample-base.prayspec"),
      <<~PRAYSPEC
        Package::Specification.new do |spec|
          spec.name = "sample/base"
          spec.version = "2.0.0"
          spec.summary = "shared guidance"
          spec.files = ["README.md", "exports/testing-basics.md"]
          spec.exports = {
            "testing-basics" => {
              type: "fragment",
              path: "exports/testing-basics.md",
              summary: "Testing guidance"
            }
          }
        end
      PRAYSPEC
    )
    Dir.chdir(source_repo) do
      Pray::CLI.run(["publish", "--root", prayers_root])
    end
    GitDistributionFixture.run_git(distribution_repo, "add", "-A")
    GitDistributionFixture.run_git(distribution_repo, "commit", "-m", "publish major version")
    updated_revision = GitDistributionFixture.run_git(distribution_repo, "rev-parse", "HEAD").strip

    GitDistributionFixture.write_consumer_prayfile(
      second_consumer,
      distribution_repo,
      constraint: "~> 2.0"
    )
    Dir.chdir(second_consumer) do
      Pray::CLI.run(["install"])
    end

    lockfile = File.read(File.join(second_consumer, "Prayfile.lock"))
    expect(lockfile).to include(updated_revision)
    expect(lockfile).to include("2.0.0")
  end
end
