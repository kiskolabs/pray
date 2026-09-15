# frozen_string_literal: true

require "spec_helper"
require "fileutils"
require_relative "../support/git_distribution_fixture"

RSpec.describe "pray update --latest" do
  let(:workspace) { Dir.mktmpdir("pray-update-latest-") }

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
  end

  after do
    FileUtils.rm_rf(workspace)
  end

  it "rewrites a Prayfile constraint that does not admit the registry latest" do
    consumer = consumer_after_major_publish

    expect { Dir.chdir(consumer) { Pray::CLI.run(["update", "--latest"]) } }
      .to output(/Prayfile: sample\/base constraint ~> 1.4 -> ~> 2.0/).to_stdout

    expect(File.read(File.join(consumer, "Prayfile"))).to include("~> 2.0")
    expect(File.read(File.join(consumer, "Prayfile.lock"))).to include("2.0.0")
  end

  it "does not write Prayfile on --latest --dry-run" do
    consumer = consumer_after_major_publish
    original = File.read(File.join(consumer, "Prayfile"))
    lockfile = File.read(File.join(consumer, "Prayfile.lock"))

    expect { Dir.chdir(consumer) { Pray::CLI.run(["update", "--latest", "--dry-run"]) } }
      .to output(/Prayfile: sample\/base constraint ~> 1.4 -> ~> 2.0/).to_stdout

    expect(File.read(File.join(consumer, "Prayfile"))).to eq(original)
    expect(File.read(File.join(consumer, "Prayfile.lock"))).to eq(lockfile)
  end

  def consumer_after_major_publish
    source_repo = File.join(workspace, "source")
    distribution_repo = File.join(workspace, "distribution")
    prayers_root = File.join(distribution_repo, "prayers")
    consumer_repo = File.join(workspace, "consumer")
    FileUtils.mkdir_p(source_repo)
    FileUtils.mkdir_p(distribution_repo)
    FileUtils.mkdir_p(consumer_repo)

    GitDistributionFixture.create_add_fixture(source_repo)
    GitDistributionFixture.publish_source_to_prayers(source_repo, prayers_root)
    GitDistributionFixture.init_distribution_repo(distribution_repo, prayers_root)
    GitDistributionFixture.write_consumer_prayfile(consumer_repo, distribution_repo)
    Dir.chdir(consumer_repo) { Pray::CLI.run(["install"]) }
    expect(File.read(File.join(consumer_repo, "Prayfile.lock"))).to include("1.4.3")

    rewrite_base_version(source_repo, "2.0.0")
    Dir.chdir(source_repo) { Pray::CLI.run(["publish", "--root", prayers_root]) }
    GitDistributionFixture.run_git(distribution_repo, "add", "-A")
    GitDistributionFixture.run_git(distribution_repo, "commit", "-m", "publish major version")
    consumer_repo
  end

  def rewrite_base_version(source, version)
    File.write(
      File.join(source, "packages/base/sample-base.prayspec"),
      <<~PRAYSPEC
        Package::Specification.new do |spec|
          spec.name = "sample/base"
          spec.version = "#{version}"
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
  end
end
