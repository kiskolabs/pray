# frozen_string_literal: true

require "spec_helper"
require "fileutils"

RSpec.describe "provisioned destination ownership" do
  def write_package(root, directory, package, export)
    package_root = File.join(root, "packages", directory)
    FileUtils.mkdir_p(File.join(package_root, "exports"))
    File.write(
      File.join(package_root, "#{directory}.prayspec"),
      <<~SPEC
        Package::Specification.new do |spec|
          spec.name = "#{package}"
          spec.version = "1.0.0"
          spec.summary = "fixture"
          spec.files = ["exports/shared"]
          spec.exports = {
            "#{export}" => { type: "file", path: "exports/shared" }
          }
        end
      SPEC
    )
    File.write(File.join(package_root, "exports/shared"), package)
  end

  it "rejects different owners of the same provisioned destination" do
    root = Dir.mktmpdir("pray-provisioned-owner-conflict-")
    begin
      write_package(root, "alpha", "sample/alpha", "alpha-file")
      write_package(root, "beta", "sample/beta", "beta-file")
      File.write(
        File.join(root, "Prayfile"),
        <<~PRAYFILE
          prayfile "1"
          pray "sample/alpha", "~> 1.0", path: "packages/alpha", file: ".shared"
          pray "sample/beta", "~> 1.0", path: "packages/beta", file: ".shared"
        PRAYFILE
      )
      project = Pray::Resolve.resolve_project(File.join(root, "Prayfile"))

      expect { Pray::Render.write_rendered_targets(project, []) }.to raise_error(
        Pray::Error,
        /provisioned destination `.shared` has multiple owners.*sample\/alpha.*alpha-file.*sample\/beta.*beta-file/
      )
      expect(File.exist?(File.join(root, ".shared"))).to be(false)
    ensure
      FileUtils.rm_rf(root)
    end
  end
end
