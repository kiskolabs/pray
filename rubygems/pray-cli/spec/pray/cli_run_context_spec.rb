# frozen_string_literal: true

require "spec_helper"
require "fileutils"

RSpec.describe "CLI.run invocation context" do
  after { Pray::Invocation.context = nil }

  def write_path_package(root, body)
    package_root = File.join(root, "packages/shell")
    FileUtils.mkdir_p(File.join(package_root, "exports"))
    File.write(
      File.join(package_root, "shell.prayspec"),
      <<~SPEC
        Package::Specification.new do |spec|
          spec.name = "sample/shell"
          spec.version = "1.0.0"
          spec.summary = "fixture"
          spec.files = ["exports/zshrc"]
          spec.exports = {
            "zshrc" => { type: "file", path: "exports/zshrc" }
          }
        end
      SPEC
    )
    File.write(File.join(package_root, "exports/zshrc"), body)
    File.write(
      File.join(root, "Prayfile"),
      <<~PRAYFILE
        prayfile "1"
        pray "sample/shell", "~> 1.0", path: "packages/shell", file: ".zshrc"
      PRAYFILE
    )
  end

  it "installs the current project after CLI.run in a removed directory" do
    first = Dir.mktmpdir("pray-cli-context-first-")
    second = Dir.mktmpdir("pray-cli-context-second-")
    begin
      write_path_package(first, "first aliases\n")
      write_path_package(second, "second aliases\n")
      Dir.chdir(first) { Pray::CLI.run(["install"]) }
      FileUtils.rm_rf(first)
      expect(Pray::Invocation.context).to be_nil

      Dir.chdir(second) do
        Pray.materialize_project(manifest_path: File.join(second, "Prayfile"))
      end
      expect(File.read(File.join(second, ".zshrc"))).to eq("second aliases\n")
    ensure
      FileUtils.rm_rf(first)
      FileUtils.rm_rf(second)
    end
  end
end
