# frozen_string_literal: true

require "spec_helper"
require "fileutils"
require_relative "../support/git_distribution_fixture"

RSpec.describe "package upstream refresh" do
  let(:workspace) { Dir.mktmpdir("pray-package-upstream-") }

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

  after { FileUtils.rm_rf(workspace) }

  it "replaces a clean fork from the locked upstream on update" do
    catalog = catalog_after_upstream_bump("~> 1.4")
    Dir.chdir(catalog) { Pray::CLI.run(["update"]) }

    expect(File.read(File.join(catalog, "packages/fork-base/exports/testing-basics.md")))
      .to eq("Testing guidance v2\n")
    spec = File.read(File.join(catalog, "packages/fork-base/fork-base.prayspec"))
    expect(spec).to include("fork/base")
    expect(spec).to include("1.0.0")
    lockfile = File.read(File.join(catalog, "Prayfile.lock"))
    expect(lockfile).to include("1.4.4")
    spec = File.read(File.join(catalog, "packages/fork-base/fork-base.prayspec"))
    expect(spec).to include("fork-base.prayspec")
    Dir.chdir(catalog) { Pray::CLI.run(["package"]) }
    expect(File).to exist(File.join(catalog, ".pray/packages/fork-base-1.0.0.praypkg"))
  end

  it "keeps an exact upstream pin on update" do
    catalog = catalog_after_upstream_bump("= 1.4.3")
    Dir.chdir(catalog) { Pray::CLI.run(["update"]) }

    expect(File.read(File.join(catalog, "packages/fork-base/exports/testing-basics.md")))
      .to eq("Testing guidance\n")
    spec = File.read(File.join(catalog, "packages/fork-base/fork-base.prayspec"))
    expect(spec).to include("= 1.4.3")
    lockfile = File.read(File.join(catalog, "Prayfile.lock"))
    expect(lockfile).to include("1.4.3")
    expect(lockfile).not_to include("1.4.4")
  end

  it "rewrites an exact upstream pin with --latest" do
    catalog = catalog_after_upstream_bump("= 1.4.3")
    Dir.chdir(catalog) { Pray::CLI.run(["update", "--latest"]) }

    expect(File.read(File.join(catalog, "packages/fork-base/exports/testing-basics.md")))
      .to eq("Testing guidance v2\n")
    spec = File.read(File.join(catalog, "packages/fork-base/fork-base.prayspec"))
    expect(spec).to include("= 1.4.4")
    expect(File.read(File.join(catalog, "Prayfile.lock"))).to include("1.4.4")
  end

  it "does not rewrite an upstream pin on --latest --dry-run" do
    catalog = catalog_after_upstream_bump("= 1.4.3")
    expect { Dir.chdir(catalog) { Pray::CLI.run(["update", "--latest", "--dry-run"]) } }
      .to output(/fork\/base upstream/).to_stdout
    spec = File.read(File.join(catalog, "packages/fork-base/fork-base.prayspec"))
    expect(spec).to include("= 1.4.3")
    expect(File.read(File.join(catalog, "packages/fork-base/exports/testing-basics.md")))
      .to eq("Testing guidance\n")
  end

  def catalog_after_upstream_bump(constraint)
    source_repo = File.join(workspace, "source")
    distribution_repo = File.join(workspace, "distribution")
    prayers_root = File.join(distribution_repo, "prayers")
    catalog_repo = File.join(workspace, "catalog")
    FileUtils.mkdir_p(source_repo)
    FileUtils.mkdir_p(distribution_repo)
    FileUtils.mkdir_p(catalog_repo)

    GitDistributionFixture.create_add_fixture(source_repo)
    Dir.chdir(source_repo) do
      GitDistributionFixture.publish_source_to_prayers(source_repo, prayers_root)
    end
    GitDistributionFixture.init_distribution_repo(distribution_repo, prayers_root)
    write_fork_package(catalog_repo, source_repo, constraint)
    write_fork_prayfile(catalog_repo, distribution_repo)
    Dir.chdir(catalog_repo) { Pray::CLI.run(["install"]) }

    bump_upstream_version(source_repo)
    Dir.chdir(source_repo) { Pray::CLI.run(["publish", "--root", prayers_root]) }
    GitDistributionFixture.run_git(distribution_repo, "add", "-A")
    GitDistributionFixture.run_git(distribution_repo, "commit", "-m", "publish 1.4.4")
    catalog_repo
  end

  def write_fork_package(catalog, source, constraint)
    root = File.join(catalog, "packages/fork-base")
    FileUtils.mkdir_p(File.join(root, "exports"))
    FileUtils.cp(File.join(source, "packages/base/README.md"), File.join(root, "README.md"))
    FileUtils.cp(
      File.join(source, "packages/base/exports/testing-basics.md"),
      File.join(root, "exports/testing-basics.md")
    )
    File.write(
      File.join(root, "fork-base.prayspec"),
      <<~PRAYSPEC
        Package::Specification.new do |spec|
          spec.name = "fork/base"
          spec.version = "1.0.0"
          spec.summary = "forked guidance"
          spec.files = ["README.md", "exports/testing-basics.md"]
          spec.exports = {
            "testing-basics" => {
              type: "fragment",
              path: "exports/testing-basics.md",
              summary: "Testing guidance"
            }
          }
          spec.upstream "sample/base", "#{constraint}"
        end
      PRAYSPEC
    )
  end

  def write_fork_prayfile(catalog, distribution)
    File.write(
      File.join(catalog, "Prayfile"),
      <<~PRAYFILE
        prayfile "1"
        source "sample", "git+file://#{distribution}"
        target :tool_a do
          output "INSTRUCTIONS.md"
        end
        agent "fork/base", "~> 1.0", path: "packages/fork-base"
        render mode: :managed, conflict: :fail, churn: :minimal
      PRAYFILE
    )
  end

  def bump_upstream_version(source)
    File.write(
      File.join(source, "packages/base/sample-base.prayspec"),
      <<~PRAYSPEC
        Package::Specification.new do |spec|
          spec.name = "sample/base"
          spec.version = "1.4.4"
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
    File.write(
      File.join(source, "packages/base/exports/testing-basics.md"),
      "Testing guidance v2\n"
    )
  end
end
