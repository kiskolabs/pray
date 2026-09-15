# frozen_string_literal: true

require "spec_helper"
require "fileutils"

RSpec.describe "RFC 0100 resolver fixtures" do
  RESOLVER_FIXTURES = File.expand_path("../../../../fixtures", __dir__)

  it "resolves the path-package fixture into a lock slice" do
    assert_lock_slice("path-package", offline: true)
  end

  it "rejects the constraint-mismatch resolver fixture" do
    dir = File.join(RESOLVER_FIXTURES, "resolver", "constraint-mismatch")
    expect { Pray.resolve_project(File.join(dir, "Prayfile"), offline: true) }.to raise_error(Pray::Error)
  end

  it "rejects the dependency-cycle resolver fixture" do
    dir = File.join(RESOLVER_FIXTURES, "resolver", "dependency-cycle")
    expect { Pray.resolve_project(File.join(dir, "Prayfile"), offline: true) }.to raise_error(Pray::Error)
  end

  it "resolves the git-distribution fixture into a lock slice" do
    assert_copied_lock_slice("git-distribution")
  end

  it "resolves the registry-distribution fixture into a lock slice" do
    assert_copied_lock_slice("registry-distribution")
  end

  it "resolves the tarball-package fixture into a lock slice" do
    assert_copied_lock_slice("tarball-package", offline: true)
  end

  def lock_slice(project, keys)
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
    lockfile.package.map do |package|
      keys.to_h { |key| [key, package.public_send(key)] }
    end.sort_by { |package| package["name"] }
  end

  def assert_lock_slice(pack, offline:)
    dir = File.join(RESOLVER_FIXTURES, "resolver", pack)
    expected = JSON.parse(File.read(File.join(dir, "expected.json")))
    keys = expected["packages"].first.keys
    packages = lock_slice(Pray.resolve_project(File.join(dir, "Prayfile"), offline: offline), keys)
    expect(packages).to eq(expected["packages"].sort_by { |package| package["name"] })
  end

  def assert_copied_lock_slice(pack, offline: false)
    source = File.join(RESOLVER_FIXTURES, "resolver", pack)
    expected = JSON.parse(File.read(File.join(source, "expected.json")))
    keys = expected["packages"].first.keys
    Dir.mktmpdir("pray-#{pack}-") do |copied|
      FileUtils.cp_r(File.join(source, "."), copied)
      packages = lock_slice(Pray.resolve_project(File.join(copied, "Prayfile"), offline: offline), keys)
      expect(packages).to eq(expected["packages"].sort_by { |package| package["name"] })
    end
  end
end
