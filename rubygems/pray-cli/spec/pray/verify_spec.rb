# frozen_string_literal: true

require "spec_helper"
require "fileutils"

RSpec.describe Pray::Verify do
  let(:simple_project) { File.expand_path("../../../../examples/simple-project", __dir__) }
  let(:workspace) { Dir.mktmpdir("pray-verify-") }

  before do
    FileUtils.cp_r("#{simple_project}/.", workspace)
    system(File.expand_path("../../bin/pray", __dir__), "install", chdir: workspace, out: File::NULL, err: File::NULL)
  end

  after do
    FileUtils.rm_rf(workspace)
  end

  it "reports a clean project after install" do
    project = Pray::Resolve.resolve_project(File.join(workspace, "Prayfile"))
    lockfile = Pray.read_lockfile(File.join(workspace, "Prayfile.lock"))
    report = described_class.verify_project(project, lockfile)

    expect(report.clean?).to be(true)
  end

  it "detects manifest hash drift against the lockfile" do
    project = Pray::Resolve.resolve_project(File.join(workspace, "Prayfile"))
    lockfile = Pray.read_lockfile(File.join(workspace, "Prayfile.lock"))
    lockfile.manifest_hash = "sha256:stale"

    report = described_class.inspect_project(project, lockfile)

    expect(report.clean?).to be(false)
    expect(report.findings.map(&:message).join).to include("Prayfile changed")
  end
end
