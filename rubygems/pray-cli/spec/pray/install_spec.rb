# frozen_string_literal: true

require "spec_helper"
require "fileutils"
require "open3"

RSpec.describe "pray install" do
  let(:simple_project) { File.expand_path("../../../../examples/simple-project", __dir__) }
  let(:pray_executable) { File.expand_path("../../bin/pray", __dir__) }
  let(:workspace) { Dir.mktmpdir("pray-install-") }

  after do
    FileUtils.rm_rf(workspace)
  end

  def run_pray(directory, arguments)
    Open3.capture3(pray_executable, *arguments, chdir: directory)
  end

  before do
    system("cp", "-a", "#{simple_project}/.", workspace)
  end

  it "installs on a copy of simple-project with matching manifest hash" do
    manifest = Pray.parse_manifest(File.read(File.join(workspace, "Prayfile")))
    expect(manifest.manifest_hash).to eq(
      "sha256:340c8fc15fa0196aadea58a50834d4f726698fb74a4967cdc340e3e653950326"
    )

    _stdout, stderr, status = run_pray(workspace, ["install"])
    expect(status.success?).to be(true), "install failed: #{stderr}"

    lockfile = Pray.read_lockfile(File.join(workspace, "Prayfile.lock"))
    expect(lockfile.manifest_hash).to eq(manifest.manifest_hash)
    expect(lockfile.generated_by).to eq("pray 1.0.0")
  end

  it "installs, renders, and verifies the simple-project example" do
    _stdout, stderr, status = run_pray(workspace, ["install"])
    expect(status.success?).to be(true), "install failed: #{stderr}"

    rendered = File.read(File.join(workspace, "INSTRUCTIONS.md"))
    expect(rendered).to include("<!-- pray:")
    expect(rendered).to include("### .agents/project.md")
    expect(rendered).not_to include("/Users/")

    lockfile_path = File.join(workspace, "Prayfile.lock")
    expect(File).to exist(lockfile_path)
    lockfile_bytes = File.binread(lockfile_path)

    _stdout, stderr, status = run_pray(workspace, ["install"])
    expect(status.success?).to be(true), "reinstall failed: #{stderr}"
    expect(File.binread(lockfile_path)).to eq(lockfile_bytes)

    _stdout, stderr, status = run_pray(workspace, ["verify"])
    expect(status.success?).to be(true), "verify failed: #{stderr}"
  end
end
