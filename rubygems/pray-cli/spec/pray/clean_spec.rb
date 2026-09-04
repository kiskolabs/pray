# frozen_string_literal: true

require "spec_helper"
require "fileutils"

RSpec.describe Pray::CLI do
  let(:workspace) { Dir.mktmpdir("pray-clean-unused-") }

  after do
    Pray::Invocation.context = nil
    FileUtils.rm_rf(workspace)
  end

  def write_lockfile(package_path)
    File.write(
      File.join(workspace, "Prayfile.lock"),
      <<~TOML
        prayfile_lock = "1"
        spec = "0.1"
        generated_by = "pray test"
        manifest_hash = "sha256:0000000000000000000000000000000000000000000000000000000000000000"
        source = []
        target = []
        managed_span = []
        provisioned = []

        [[package]]
        name = "sample/base"
        version = "1.4.3"
        path = "#{package_path}"
        tree_hash = "sha256:1111111111111111111111111111111111111111111111111111111111111111"
        artifact_hash = "sha256:2222222222222222222222222222222222222222222222222222222222222222"
        artifact = "path:#{package_path}"
        exports = []
        dependencies = []
      TOML
    )
  end

  def create_cache(path)
    FileUtils.mkdir_p(path)
    File.write(File.join(path, "entry"), "cached")
  end

  it "retains only locked registry entries without touching other state" do
    locked = File.join(workspace, ".pray/cache/registry/sample/base/1.4.3/source")
    stale_version = File.join(workspace, ".pray/cache/registry/sample/base/1.4.2/source")
    stale_source = File.join(workspace, ".pray/cache/registry/sample/base/1.4.3/other")
    legacy = File.join(workspace, ".pray/cache/registry/legacy/sample/base/1.4.3")
    staging = File.join(workspace, ".pray/cache/registry/sample/base/1.4.3/source.staging")
    [locked, stale_version, stale_source, legacy, staging].each { |path| create_cache(path) }
    create_cache(File.join(workspace, ".pray/cache/git/repository"))
    create_cache(File.join(workspace, ".pray/vendor/sample-base"))
    File.write(File.join(workspace, ".pray/state.json"), "{}")
    global_cache = File.join(workspace, "global/cache/entry")
    create_cache(global_cache)
    write_lockfile("./.pray/cache/registry/sample/base/1.4.3/source")

    Dir.chdir(workspace) { described_class.clean_command(unused: true) }

    expect(File).to exist(locked)
    expect(File).not_to exist(stale_version)
    expect(File).not_to exist(stale_source)
    expect(File).not_to exist(legacy)
    expect(File).not_to exist(staging)
    expect(File).to exist(File.join(workspace, ".pray/cache/git/repository"))
    expect(File).to exist(File.join(workspace, ".pray/vendor/sample-base"))
    expect(File).to exist(File.join(workspace, ".pray/state.json"))
    expect(File).to exist(global_cache)
  end

  it "requires a readable and parseable lockfile" do
    [nil, "not valid = [\n"].each do |contents|
      FileUtils.rm_rf(File.join(workspace, "Prayfile.lock"))
      cache = File.join(workspace, ".pray/cache/registry/sample/base/1.0.0/source")
      create_cache(cache)
      File.write(File.join(workspace, "Prayfile.lock"), contents) if contents

      expect do
        Dir.chdir(workspace) { described_class.clean_command(unused: true) }
      end.to raise_error(StandardError)
      expect(File).to exist(cache)
    end
  end

  it "rejects an incomplete lockfile before deleting" do
    cache = File.join(workspace, ".pray/cache/registry/sample/base/1.0.0/source")
    create_cache(cache)
    write_lockfile("./.pray/cache/registry/sample/base/1.0.0/source")
    lockfile_path = File.join(workspace, "Prayfile.lock")
    contents = File.read(lockfile_path).sub(
      "sha256:0000000000000000000000000000000000000000000000000000000000000000",
      "sha256:incomplete"
    )
    File.write(lockfile_path, contents)

    expect do
      Dir.chdir(workspace) { described_class.clean_command(unused: true) }
    end.to raise_error(Pray::Error, /manifest_hash/)
    expect(File).to exist(cache)
  end

  it "cleans the selected project root" do
    nested = File.join(workspace, "nested")
    FileUtils.mkdir_p(nested)
    create_cache(File.join(workspace, ".pray/cache/entry"))
    create_cache(File.join(workspace, ".pray/vendor/entry"))
    File.write(File.join(workspace, ".pray/state.json"), "{}")

    Dir.chdir(nested) { described_class.run(["--path", workspace, "clean"]) }

    expect(File).not_to exist(File.join(workspace, ".pray/cache"))
    expect(File).not_to exist(File.join(workspace, ".pray/vendor"))
    expect(File).not_to exist(File.join(workspace, ".pray/state.json"))
  end

  it "does not follow registry symlinks" do
    outside = Dir.mktmpdir("pray-clean-outside-")
    File.write(File.join(outside, "keep"), "outside")
    FileUtils.mkdir_p(File.join(workspace, ".pray/cache/registry"))
    File.symlink(outside, File.join(workspace, ".pray/cache/registry/stale"))
    write_lockfile("./packages/base")

    Dir.chdir(workspace) { described_class.clean_command(unused: true) }

    expect(File).not_to exist(File.join(workspace, ".pray/cache/registry/stale"))
    expect(File).to exist(File.join(outside, "keep"))
  ensure
    FileUtils.rm_rf(outside)
  end
end
