# frozen_string_literal: true

require "fileutils"
require "open3"

module GitDistributionFixture
  module_function

  def create_add_fixture(repo)
    FileUtils.mkdir_p(File.join(repo, "packages/base/exports"))
    File.write(
      File.join(repo, "Prayfile"),
      <<~PRAYFILE
        prayfile "1"
        target :tool_a do
          output "INSTRUCTIONS.md"
        end
        render mode: :managed, conflict: :fail, churn: :minimal
      PRAYFILE
    )
    File.write(
      File.join(repo, "packages/base/sample-base.prayspec"),
      <<~PRAYSPEC
        Package::Specification.new do |spec|
          spec.name = "sample/base"
          spec.version = "1.4.3"
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
    File.write(File.join(repo, "packages/base/README.md"), "package readme\n")
    File.write(
      File.join(repo, "packages/base/exports/testing-basics.md"),
      "Testing guidance\n"
    )
  end

  def run_git(directory, *arguments)
    output, status = Open3.capture2e("git", "-C", directory, *arguments)
    raise "git #{arguments.join(" ")} failed: #{output}" unless status.success?

    output
  end

  def init_distribution_repo(distribution_repo, prayers_root)
    FileUtils.mkdir_p(prayers_root)
    run_git(distribution_repo, "init", "-b", "main")
    run_git(distribution_repo, "config", "user.name", "Pray Test")
    run_git(distribution_repo, "config", "user.email", "pray@example.com")
    run_git(distribution_repo, "add", "-A")
    run_git(distribution_repo, "commit", "-m", "initial distribution")
  end

  def publish_source_to_prayers(source_repo, prayers_root)
    Dir.chdir(source_repo) do
      Pray::CLI.run(["add", "sample/base", "--path", "packages/base"])
      Pray::CLI.run(["publish", "--root", prayers_root])
    end
  end

  def write_consumer_prayfile(consumer_repo, distribution_repo, constraint: "~> 1.4")
    File.write(
      File.join(consumer_repo, "Prayfile"),
      <<~PRAYFILE
        prayfile "1"
        source "dist", "git+file://#{distribution_repo}"
        agent "sample/base", "#{constraint}", source: "dist"
        target :tool_a do
          output "INSTRUCTIONS.md"
        end
        render mode: :managed, conflict: :fail, churn: :minimal
      PRAYFILE
    )
  end
end
