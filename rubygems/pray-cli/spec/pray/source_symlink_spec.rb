# frozen_string_literal: true

require "spec_helper"
require "fileutils"

RSpec.describe "source symlink safety" do
  it "rejects a package directory reached through a symlink" do
    Dir.mktmpdir("pray-package-root-") do |root|
      Dir.mktmpdir("pray-package-root-outside-") do |outside|
        package_root = File.join(outside, "base")
        FileUtils.mkdir_p(File.join(package_root, "exports"))
        File.write(File.join(package_root, "exports/a.md"), "outside\n")
        File.write(File.join(package_root, "base.prayspec"), <<~SPEC)
          Package::Specification.new do |spec|
            spec.name = "sample/base"
            spec.version = "1.0.0"
            spec.files = ["exports/a.md"]
            spec.exports = { "a" => { type: "fragment", path: "exports/a.md" } }
          end
        SPEC
        FileUtils.mkdir_p(File.join(root, "packages"))
        File.symlink(package_root, File.join(root, "packages/base"))
        File.write(File.join(root, "Prayfile"), <<~PRAYFILE)
          prayfile "1"
          compose "AGENTS.md" do
            pray "sample/base", path: "packages/base"
          end
        PRAYFILE

        expect do
          Pray::Resolve.resolve_project(File.join(root, "Prayfile"))
        end.to raise_error(Pray::Error, /package directory uses a symbolic link/)
      end
    end
  end

  it "rejects package files reached through symlinks" do
    Dir.mktmpdir("pray-source-symlink-") do |root|
      package_root = File.join(root, "package")
      FileUtils.mkdir_p(File.join(package_root, "exports"))
      outside = File.join(root, "outside.md")
      File.write(outside, "outside\n")
      File.symlink(outside, File.join(package_root, "exports/a.md"))
      File.write(File.join(package_root, "package.prayspec"), <<~SPEC)
        Package::Specification.new do |spec|
          spec.name = "sample/package"
          spec.version = "1.0.0"
          spec.files = ["exports/a.md"]
          spec.exports = { "a" => { type: "fragment", path: "exports/a.md" } }
        end
      SPEC
      File.write(File.join(root, "Prayfile"), <<~PRAYFILE)
        prayfile "1"
        compose "AGENTS.md" do
          pray "sample/package", path: "package"
        end
      PRAYFILE

      expect do
        Pray::Resolve.resolve_project(File.join(root, "Prayfile"))
      end.to raise_error(Pray::Error, /symbolic link/)
    end
  end

  it "rejects local compose files reached through symlinks" do
    Dir.mktmpdir("pray-local-symlink-") do |root|
      outside = File.join(root, "outside.md")
      File.write(outside, "outside\n")
      File.symlink(outside, File.join(root, "local.md"))
      File.write(File.join(root, "Prayfile"), <<~PRAYFILE)
        prayfile "1"
        compose "AGENTS.md" do
          local "local.md"
        end
      PRAYFILE

      expect do
        Pray::Resolve.resolve_project(File.join(root, "Prayfile"))
      end.to raise_error(Pray::Error, /symbolic link/)
    end
  end

  it "keeps missing and directory package-file errors distinct" do
    Dir.mktmpdir("pray-source-errors-") do |root|
      FileUtils.mkdir_p(File.join(root, "folder"))
      missing = Pray::PackageSpec.new(files: ["missing.md"])
      directory = Pray::PackageSpec.new(files: ["folder"])

      expect do
        Pray::Resolve.load_package_file_bytes(root, missing)
      end.to raise_error(Pray::Error, /package file missing: missing\.md/)
      expect do
        Pray::Resolve.load_package_file_bytes(root, directory)
      end.to raise_error(Pray::Error, /package file is a directory: folder/)
    end
  end

  it "requires exactly one regular prayspec file" do
    Dir.mktmpdir("pray-prayspec-count-") do |root|
      expect do
        Pray::Resolve.find_prayspec_file(root)
      end.to raise_error(Pray::Error, /no prayspec file/)

      File.write(File.join(root, "first.prayspec"), "first")
      File.write(File.join(root, "second.prayspec"), "second")
      expect do
        Pray::Resolve.find_prayspec_file(root)
      end.to raise_error(Pray::Error, /multiple prayspec files/)
    end
  end
end
