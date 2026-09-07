# frozen_string_literal: true

require "spec_helper"

RSpec.describe "destination recovery" do
  let(:root) { Dir.mktmpdir("pray-destination-recovery-") }
  let(:manifest_path) { File.join(root, "Prayfile") }

  before do
    FileUtils.mkdir_p(File.join(root, "package/files"))
    File.write(manifest_path, <<~PRAYFILE)
      prayfile "1"
      tree "skills" do
        pray "sample/files", "~> 1.0", path: "package"
      end
      compose "INSTRUCTIONS.md" do
        pray "local.md"
      end
    PRAYFILE
    File.write(File.join(root, "local.md"), "old rules\n")
    File.write(File.join(root, "package/files.prayspec"), <<~SPEC)
      Package::Specification.new do |spec|
        spec.name = "sample/files"
        spec.version = "1.0.0"
        spec.files = ["files/a.md", "files/b.md"]
        spec.exports = { "files" => { type: "folder", path: "files" } }
      end
    SPEC
    %w[a.md b.md].each { |name| File.write(File.join(root, "package/files", name), "old content") }
    Pray::CLI.run(["--path", root, "install"])
  end

  after do
    FileUtils.rm_rf(root)
  end

  it "reports every collision before changing compose output" do
    lock_path = File.join(root, "Prayfile.lock")
    lockfile = Pray.read_lockfile(lock_path)
    destinations = lockfile.provisioned.map(&:path)
    lockfile.provisioned = []
    Pray.write_lockfile(lock_path, lockfile)
    previous_lock = File.read(lock_path)
    compose = File.read(File.join(root, "INSTRUCTIONS.md"))
    File.write(File.join(root, "local.md"), "new rules\n")
    %w[a.md b.md].each { |name| File.write(File.join(root, "package/files", name), "new content") }

    expect { Pray::CLI.run(["--path", root, "install"]) }.to raise_error(Pray::Error) { |error|
      destinations.each { |path| expect(error.message).to include(path) }
      expect(error.message).to include("sample/files", "move")
    }
    expect(File.read(lock_path)).to eq(previous_lock)
    expect(File.read(File.join(root, "INSTRUCTIONS.md"))).to eq(compose)
  end

  it "suggests a recovery that keeps an operator's edits" do
    lockfile = Pray.read_lockfile(File.join(root, "Prayfile.lock"))
    destination = File.join(root, lockfile.provisioned.first.path)
    File.write(destination, "operator changes")
    expect { Pray::CLI.run(["--path", root, "verify"]) }.to raise_error(Pray::Error, /move/)
    FileUtils.mv(destination, "#{destination}.saved")
    Pray::CLI.run(["--path", root, "install"])
    Pray::CLI.run(["--path", root, "verify"])
    expect(File.read("#{destination}.saved")).to eq("operator changes")
  end

  it "rejects unavailable update options before changing the project" do
    manifest = File.read(manifest_path)
    lockfile = File.read(File.join(root, "Prayfile.lock"))
    %w[--latest --major --dry-run --json].each do |option|
      expect { Pray::CLI.run(["--path", root, "update", option]) }
        .to raise_error(Pray::Error)
      expect(File.read(manifest_path)).to eq(manifest)
      expect(File.read(File.join(root, "Prayfile.lock"))).to eq(lockfile)
    end
  end
end
