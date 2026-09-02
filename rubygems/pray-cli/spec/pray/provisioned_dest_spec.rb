# frozen_string_literal: true

require "spec_helper"
require "fileutils"

RSpec.describe "provisioned destination safety" do
  def write_file_package(root, body)
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

  def resolve_root(root)
    Pray::Resolve.resolve_project(File.join(root, "Prayfile"))
  end

  def lockfile_with_provisioned(project)
    lockfile = Pray.build_lockfile(
      project.manifest_hash,
      project.environment,
      project.project_root,
      project.manifest.sources,
      project.manifest.targets,
      [],
      project.packages,
      project.source_revisions,
      project.source_host_keys
    )
    lockfile.provisioned = Pray::RenderDest.provisioned_records(project)
    lockfile.canonicalized
  end

  it "rejects a leading tilde only in a destination path" do
    expect(Pray::PathSafety.validate_project_relative_path!("~fixtures/shell")).to eq("~fixtures/shell")
    expect { Pray::PathSafety.validate_destination_path!("~/.zshrc") }.to raise_error(
      Pray::Error, /repository-relative/
    )
  end

  it "writes an exclusive file when the dest is missing" do
    root = Dir.mktmpdir("pray-provisioned-missing-")
    begin
      write_file_package(root, "alias ll=ls\n")
      project = resolve_root(root)
      Pray::Render.write_rendered_targets(project, [])
      expect(File.read(File.join(root, ".zshrc"))).to eq("alias ll=ls\n")
    ensure
      FileUtils.rm_rf(root)
    end
  end

  it "adopts a dest whose bytes already match expected" do
    root = Dir.mktmpdir("pray-provisioned-adopt-")
    begin
      write_file_package(root, "alias ll=ls\n")
      File.write(File.join(root, ".zshrc"), "alias ll=ls\n")
      project = resolve_root(root)
      Pray::Render.write_rendered_targets(project, [])
      expect(File.read(File.join(root, ".zshrc"))).to eq("alias ll=ls\n")
    ensure
      FileUtils.rm_rf(root)
    end
  end

  it "refuses to clobber unmanaged dest bytes" do
    root = Dir.mktmpdir("pray-provisioned-clobber-")
    begin
      write_file_package(root, "alias ll=ls\n")
      File.write(File.join(root, ".zshrc"), "keep me\n")
      project = resolve_root(root)
      expect { Pray::Render.write_rendered_targets(project, []) }.to raise_error(
        Pray::Error, /\.zshrc/
      )
      expect(File.read(File.join(root, ".zshrc"))).to eq("keep me\n")
    ensure
      FileUtils.rm_rf(root)
    end
  end

  it "refuses a symlink destination" do
    root = Dir.mktmpdir("pray-provisioned-symlink-")
    begin
      write_file_package(root, "alias ll=ls\n")
      target = File.join(root, "real-zshrc")
      File.write(target, "keep link target\n")
      File.symlink(target, File.join(root, ".zshrc"))
      project = resolve_root(root)
      expect { Pray::Render.write_rendered_targets(project, []) }.to raise_error(
        Pray::Error, /symbolic link/
      )
      expect(File.read(target)).to eq("keep link target\n")
    ensure
      FileUtils.rm_rf(root)
    end
  end

  it "refuses a symlinked parent directory" do
    base = Dir.mktmpdir("pray-provisioned-parent-link-")
    begin
      root = File.join(base, "project")
      outside = File.join(base, "outside")
      FileUtils.mkdir_p(root)
      FileUtils.mkdir_p(outside)
      write_file_package(root, "alias ll=ls\n")
      File.write(
        File.join(root, "Prayfile"),
        <<~PRAYFILE
          prayfile "1"
          pray "sample/shell", "~> 1.0", path: "packages/shell", file: "linked/zshrc"
        PRAYFILE
      )
      File.symlink(outside, File.join(root, "linked"))
      project = resolve_root(root)

      expect { Pray::Render.write_rendered_targets(project, []) }.to raise_error(
        Pray::Error, /symbolic link/
      )
      expect(File.exist?(File.join(outside, "zshrc"))).to be(false)
    ensure
      FileUtils.rm_rf(base)
    end
  end

  it "updates when the previous lock hash still matches" do
    root = Dir.mktmpdir("pray-provisioned-update-")
    begin
      write_file_package(root, "alias ll=ls\n")
      project = resolve_root(root)
      Pray::Render.write_rendered_targets(project, [])
      lockfile = lockfile_with_provisioned(project)
      File.write(File.join(root, "packages/shell/exports/zshrc"), "alias la=ls\n")
      updated = resolve_root(root)
      Pray::Render.write_rendered_targets(updated, [], lockfile)
      expect(File.read(File.join(root, ".zshrc"))).to eq("alias la=ls\n")
    ensure
      FileUtils.rm_rf(root)
    end
  end

  it "refuses a user-edited managed dest" do
    root = Dir.mktmpdir("pray-provisioned-edited-")
    begin
      write_file_package(root, "alias ll=ls\n")
      project = resolve_root(root)
      Pray::Render.write_rendered_targets(project, [])
      lockfile = lockfile_with_provisioned(project)
      File.write(File.join(root, ".zshrc"), "my aliases\n")
      File.write(File.join(root, "packages/shell/exports/zshrc"), "alias la=ls\n")
      updated = resolve_root(root)
      expect { Pray::Render.write_rendered_targets(updated, [], lockfile) }.to raise_error(
        Pray::Error, /\.zshrc/
      )
      expect(File.read(File.join(root, ".zshrc"))).to eq("my aliases\n")
    ensure
      FileUtils.rm_rf(root)
    end
  end

  it "prunes a matching leaf and keeps an edited dest" do
    root = Dir.mktmpdir("pray-provisioned-prune-")
    begin
      write_file_package(root, "alias ll=ls\n")
      project = resolve_root(root)
      Pray::Render.write_rendered_targets(project, [])
      lockfile = lockfile_with_provisioned(project)
      File.write(File.join(root, "Prayfile"), %(prayfile "1"\n))
      empty = resolve_root(root)
      Pray::Render.write_rendered_targets(empty, [], lockfile)
      expect(File.exist?(File.join(root, ".zshrc"))).to be(false)

      write_file_package(root, "alias ll=ls\n")
      project = resolve_root(root)
      Pray::Render.write_rendered_targets(project, [])
      lockfile = lockfile_with_provisioned(project)
      File.write(File.join(root, ".zshrc"), "my aliases\n")
      File.write(File.join(root, "Prayfile"), %(prayfile "1"\n))
      empty = resolve_root(root)
      Pray::Render.write_rendered_targets(empty, [], lockfile)
      expect(File.read(File.join(root, ".zshrc"))).to eq("my aliases\n")
    ensure
      FileUtils.rm_rf(root)
    end
  end

  it "rejects a lock path outside the project before pruning" do
    base = Dir.mktmpdir("pray-provisioned-lock-escape-")
    begin
      root = File.join(base, "project")
      FileUtils.mkdir_p(root)
      outside = File.join(base, "outside.txt")
      File.write(outside, "keep me\n")
      project = Struct.new(:project_root).new(root)
      record = Pray::ProvisionedFileRecord.new(
        path: "../outside.txt",
        content_hash: Pray::Hashing.sha256_prefixed("keep me\n"),
        package: "sample/shell",
        export: "zshrc"
      )
      previous = Struct.new(:provisioned).new([record])

      expect { Pray::RenderDest.prune_dropped(project, previous, {}) }.to raise_error(
        Pray::Error, /escapes/
      )
      expect(File.read(outside)).to eq("keep me\n")
    ensure
      FileUtils.rm_rf(base)
    end
  end

  it "reports a provisioned refusal instead of an update" do
    root = Dir.mktmpdir("pray-provisioned-plan-refusal-")
    begin
      write_file_package(root, "package aliases\n")
      File.write(File.join(root, ".zshrc"), "operator aliases\n")
      project = resolve_root(root)
      file = Pray::Render.planned_provisioned_files(project).first

      expect { Pray::Plan.provisioned_change(project, file, nil) }.to raise_error(
        Pray::Error, /refusing to overwrite `\.zshrc`/
      )
    ensure
      FileUtils.rm_rf(root)
    end
  end

  it "keeps the previous lock when destination materialization fails" do
    root = Dir.mktmpdir("pray-provisioned-retry-")
    begin
      write_file_package(root, "old aliases\n")
      Dir.chdir(root) { Pray.materialize_project(manifest_path: File.join(root, "Prayfile")) }
      lockfile_path = File.join(root, "Prayfile.lock")
      previous_lock = File.binread(lockfile_path)
      File.write(File.join(root, "packages/shell/exports/zshrc"), "new aliases\n")
      File.delete(File.join(root, ".zshrc"))
      FileUtils.mkdir_p(File.join(root, ".zshrc"))

      expect do
        Dir.chdir(root) { Pray.materialize_project(manifest_path: File.join(root, "Prayfile")) }
      end.to raise_error(Pray::Error)
      expect(File.binread(lockfile_path)).to eq(previous_lock)

      FileUtils.rm_rf(File.join(root, ".zshrc"))
      File.write(File.join(root, ".zshrc"), "old aliases\n")
      Dir.chdir(root) { Pray.materialize_project(manifest_path: File.join(root, "Prayfile")) }
      expect(File.read(File.join(root, ".zshrc"))).to eq("new aliases\n")
    ensure
      FileUtils.rm_rf(root)
    end
  end

  it "records path, hash, package, and export" do
    root = Dir.mktmpdir("pray-provisioned-lock-")
    begin
      write_file_package(root, "alias ll=ls\n")
      project = resolve_root(root)
      records = Pray::RenderDest.provisioned_records(project)
      expect(records.length).to eq(1)
      expect(records.first.path).to eq(".zshrc")
      expect(records.first.content_hash).to eq(Pray::Hashing.sha256_prefixed("alias ll=ls\n"))
      expect(records.first.package).to eq("sample/shell")
      expect(records.first.export).to eq("zshrc")
    ensure
      FileUtils.rm_rf(root)
    end
  end
end
