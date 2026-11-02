# frozen_string_literal: true

require "spec_helper"
require "fileutils"
require "open3"

RSpec.describe "optional dependency referrals" do
  def write_package(root, leaf, version, optional_name = nil, constraint = nil)
    package_root = File.join(root, "packages", leaf)
    FileUtils.mkdir_p(package_root)
    dependency = if optional_name
      %(  spec.add_optional_dependency "#{optional_name}", "#{constraint}"\n)
    else
      ""
    end
    File.write(File.join(package_root, "README.md"), "#{leaf}\n")
    File.write(File.join(package_root, "#{leaf}.prayspec"), <<~PRAYSPEC)
      Package::Specification.new do |spec|
        spec.name = "sample/#{leaf}"
        spec.version = "#{version}"
        spec.files = ["README.md"]
      #{dependency}end
    PRAYSPEC
  end

  def write_manifest(root, leaves)
    declarations = leaves.map { |leaf| %(pray "sample/#{leaf}", path: "packages/#{leaf}") }
    File.write(File.join(root, "Prayfile"), ([%(prayfile "1")] + declarations).join("\n") + "\n")
  end

  it "accepts mutual optional referrals" do
    root = Dir.mktmpdir("pray-mutual-optional-")
    write_package(root, "alpha", "1.0.0", "sample/beta", "~> 1.0")
    write_package(root, "beta", "1.0.0", "sample/alpha", "~> 1.0")
    write_manifest(root, %w[alpha beta])

    expect(Pray::Resolve.resolve_project(File.join(root, "Prayfile")).packages.length).to eq(2)
    executable = File.expand_path("../../bin/pray", __dir__)
    stdout, stderr, status = Open3.capture3("ruby", executable, "tree", chdir: root)
    expect(status).to be_success, stderr
    expect(stdout).not_to include("(cycle)")
  ensure
    FileUtils.rm_rf(root) if root
  end

  it "checks an optional constraint when the target is selected" do
    root = Dir.mktmpdir("pray-optional-constraint-")
    write_package(root, "alpha", "1.0.0", "sample/beta", "~> 2.0")
    write_package(root, "beta", "1.0.0")
    write_manifest(root, %w[alpha beta])

    expect do
      Pray::Resolve.resolve_project(File.join(root, "Prayfile"))
    end.to raise_error(
      Pray::Error,
      /optional referral sample\/alpha -> sample\/beta requires ~> 2\.0, but selected version is 1\.0\.0/
    )
  ensure
    FileUtils.rm_rf(root) if root
  end

  it "does not select an absent optional target" do
    root = Dir.mktmpdir("pray-optional-absent-")
    write_package(root, "alpha", "1.0.0", "sample/missing", "~> 1.0")
    write_manifest(root, %w[alpha])

    expect(Pray::Resolve.resolve_project(File.join(root, "Prayfile")).packages.length).to eq(1)
  ensure
    FileUtils.rm_rf(root) if root
  end
end
