# frozen_string_literal: true

require "spec_helper"
require "fileutils"
require_relative "../support/git_distribution_fixture"

RSpec.describe "git distribution install" do
  let(:workspace) { Dir.mktmpdir("pray-git-distribution-") }

  after do
    FileUtils.rm_rf(workspace)
  end

  it "resolves packages from a git distribution repository" do
    source_repo = File.join(workspace, "source")
    distribution_repo = File.join(workspace, "distribution")
    prayers_root = File.join(distribution_repo, "prayers")
    consumer_repo = File.join(workspace, "consumer")

    FileUtils.mkdir_p(source_repo)
    FileUtils.mkdir_p(distribution_repo)
    FileUtils.mkdir_p(consumer_repo)

    GitDistributionFixture.create_add_fixture(source_repo)

    Dir.chdir(source_repo) do
      GitDistributionFixture.publish_source_to_prayers(source_repo, prayers_root)
    end
    GitDistributionFixture.init_distribution_repo(distribution_repo, prayers_root)
    GitDistributionFixture.write_consumer_prayfile(consumer_repo, distribution_repo)

    Dir.chdir(consumer_repo) do
      Pray::CLI.run(["install"])
    end

    lockfile = File.read(File.join(consumer_repo, "Prayfile.lock"))
    expect(lockfile).to include("sample/base")
    expect(lockfile).to include('revision = "')
    expect(File).to exist(File.join(consumer_repo, "INSTRUCTIONS.md"))
  end

  it "keeps the locked git revision when the distribution moves forward" do
    source_repo = File.join(workspace, "source")
    distribution_repo = File.join(workspace, "distribution")
    prayers_root = File.join(distribution_repo, "prayers")
    consumer_repo = File.join(workspace, "consumer")

    FileUtils.mkdir_p(source_repo)
    FileUtils.mkdir_p(distribution_repo)
    FileUtils.mkdir_p(consumer_repo)

    GitDistributionFixture.create_add_fixture(source_repo)

    Dir.chdir(source_repo) do
      GitDistributionFixture.publish_source_to_prayers(source_repo, prayers_root)
    end
    GitDistributionFixture.init_distribution_repo(distribution_repo, prayers_root)

    initial_revision = GitDistributionFixture.run_git(distribution_repo, "rev-parse", "HEAD").strip
    GitDistributionFixture.write_consumer_prayfile(consumer_repo, distribution_repo)

    Dir.chdir(consumer_repo) do
      Pray::CLI.run(["install"])
    end

    File.write(
      File.join(prayers_root, "v1", "index.json"),
      File.read(File.join(prayers_root, "v1", "index.json")).sub(
        '"packages"',
        '"marker":"moved-forward","packages"'
      )
    )
    GitDistributionFixture.run_git(distribution_repo, "add", "-A")
    GitDistributionFixture.run_git(distribution_repo, "commit", "-m", "advance distribution")

    Dir.chdir(consumer_repo) do
      Pray::CLI.run(["install", "--locked"])
    end

    lockfile = File.read(File.join(consumer_repo, "Prayfile.lock"))
    expect(lockfile).to include(initial_revision)
  end

  it "advances the git revision on update" do
    source_repo = File.join(workspace, "source")
    distribution_repo = File.join(workspace, "distribution")
    prayers_root = File.join(distribution_repo, "prayers")
    consumer_repo = File.join(workspace, "consumer")

    FileUtils.mkdir_p(source_repo)
    FileUtils.mkdir_p(distribution_repo)
    FileUtils.mkdir_p(consumer_repo)

    GitDistributionFixture.create_add_fixture(source_repo)

    Dir.chdir(source_repo) do
      GitDistributionFixture.publish_source_to_prayers(source_repo, prayers_root)
    end
    GitDistributionFixture.init_distribution_repo(distribution_repo, prayers_root)
    GitDistributionFixture.write_consumer_prayfile(consumer_repo, distribution_repo)

    Dir.chdir(consumer_repo) do
      Pray::CLI.run(["install"])
    end

    initial_revision = GitDistributionFixture.run_git(distribution_repo, "rev-parse", "HEAD").strip

    File.write(
      File.join(prayers_root, "v1", "index.json"),
      File.read(File.join(prayers_root, "v1", "index.json")).sub(
        '"packages"',
        '"marker":"update-target","packages"'
      )
    )
    GitDistributionFixture.run_git(distribution_repo, "add", "-A")
    GitDistributionFixture.run_git(distribution_repo, "commit", "-m", "advance distribution")
    advanced_revision = GitDistributionFixture.run_git(distribution_repo, "rev-parse", "HEAD").strip

    expect(advanced_revision).not_to eq(initial_revision)

    Dir.chdir(consumer_repo) do
      Pray::CLI.run(["update"])
    end

    lockfile = File.read(File.join(consumer_repo, "Prayfile.lock"))
    expect(lockfile).to include(advanced_revision)
  end
end
