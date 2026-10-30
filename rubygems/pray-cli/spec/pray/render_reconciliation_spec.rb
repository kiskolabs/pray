# frozen_string_literal: true

require "spec_helper"
require "fileutils"
require "open3"

RSpec.describe "render reconciliation" do
  let(:pray_executable) { File.expand_path("../../bin/pray", __dir__) }
  let(:workspace) { Dir.mktmpdir("pray-render-reconciliation-") }

  after { FileUtils.rm_rf(workspace) }

  it "checks the bytes that render would write" do
    source = File.expand_path("../../../../examples/simple-project", __dir__)
    Dir.children(source).each { |entry| FileUtils.cp_r(File.join(source, entry), workspace) }
    _stdout, stderr, status = run_pray("install")
    expect(status).to be_success, stderr
    destination = File.join(workspace, "INSTRUCTIONS.md")
    expect(run_pray("render", "--check")[2]).to be_success

    File.write(destination, File.read(destination).delete_suffix("\n"))
    expect(run_pray("render", "--check")[2]).not_to be_success
    _stdout, stderr, status = run_pray("render")
    expect(status).to be_success, stderr
    expect(run_pray("render", "--check")[2]).to be_success

    File.write(destination, "#{File.read(destination)}\n")
    before = File.binread(destination)

    _stdout, stderr, status = run_pray("render", "--check")
    expect(status).to be_success, stderr
    expect(File.binread(destination)).to eq(before)

    stdout, stderr, status = run_pray("plan")
    expect(status).to be_success, stderr
    expect(stdout).to include("Target INSTRUCTIONS.md: unchanged")

    _stdout, stderr, status = run_pray("install", "--frozen")
    expect(status).to be_success, stderr
    expect(File.binread(destination)).to eq(before)
  end

  it "removes only an unchanged span retired by the previous lock" do
    write_two_package_project
    _stdout, stderr, status = run_pray("install")
    expect(status).to be_success, stderr
    previous = Pray.read_lockfile(File.join(workspace, "Prayfile.lock"))
    retired_id = previous.managed_span.find { |span| span.package == "sample/one" }.id

    _stdout, stderr, status = run_pray("remove", "sample/one")
    expect(status).to be_success, stderr
    destination = File.read(File.join(workspace, "AGENTS.md"))
    expect(destination).not_to include("pray:#{retired_id}")
    expect(destination).to include("Two body")
    expect(run_pray("verify", "--strict")[2]).to be_success
  end

  it "refuses to overwrite an edited active managed span" do
    write_two_package_project
    _stdout, stderr, status = run_pray("install")
    expect(status).to be_success, stderr
    destination = File.join(workspace, "AGENTS.md")
    edited = File.read(destination).sub("One body", "Manual one body")
    File.write(destination, edited)

    _stdout, _stderr, status = run_pray("render")
    expect(status).not_to be_success
    expect(File.read(destination)).to eq(edited)
  end

  it "preserves an edited span retired by the previous lock" do
    write_two_package_project
    _stdout, stderr, status = run_pray("install")
    expect(status).to be_success, stderr
    destination = File.join(workspace, "AGENTS.md")
    File.write(destination, File.read(destination).sub("One body", "Edited one body"))

    _stdout, stderr, status = run_pray("remove", "sample/one")
    expect(status).to be_success, stderr
    expect(File.read(destination)).to include("Edited one body")
    _stdout, stderr, status = run_pray("verify", "--strict")
    expect(status).not_to be_success
    expect(stderr).to include("Inspect and remove the block explicitly")
  end

  it "makes render agree with render check when a span is retired" do
    write_two_package_project
    _stdout, stderr, status = run_pray("install")
    expect(status).to be_success, stderr
    previous = Pray.read_lockfile(File.join(workspace, "Prayfile.lock"))
    retired_id = previous.managed_span.find { |span| span.package == "sample/one" }.id
    manifest = File.read(File.join(workspace, "Prayfile"))
    File.write(File.join(workspace, "Prayfile"), manifest.lines.reject { |line| line.include?("sample/one") }.join)

    expect(run_pray("render", "--check")[2]).not_to be_success
    _stdout, stderr, status = run_pray("render")
    expect(status).to be_success, stderr
    expect(File.read(File.join(workspace, "AGENTS.md"))).not_to include("pray:#{retired_id}")
    expect(run_pray("render", "--check")[2]).to be_success
  end

  it "preserves an unknown span and gives manual removal guidance" do
    write_two_package_project
    _stdout, stderr, status = run_pray("install")
    expect(status).to be_success, stderr
    destination = File.join(workspace, "AGENTS.md")
    File.write(destination, "#{File.read(destination)}<!-- pray:unknown1 -->\nUnknown\n<!-- pray:unknown1 -->\n")

    expect(run_pray("install")[2]).to be_success
    expect(File.read(destination)).to include("pray:unknown1")
    _stdout, stderr, status = run_pray("verify", "--strict")
    expect(status).not_to be_success
    expect(stderr).to include("Inspect and remove the block explicitly")
    expect(stderr).not_to include("run `pray install` to reconcile")
  end

  def write_two_package_project
    File.write(File.join(workspace, "Prayfile"), <<~PRAYFILE)
      prayfile "1"
      compose "AGENTS.md" do
        pray "sample/one", path: "packages/one"
        pray "sample/two", path: "packages/two"
      end
    PRAYFILE
    write_package("one", "One body")
    write_package("two", "Two body")
  end

  def write_package(name, body)
    root = File.join(workspace, "packages", name)
    FileUtils.mkdir_p(File.join(root, "exports"))
    File.write(File.join(root, "exports", "rules.md"), "#{body}\n")
    File.write(File.join(root, "sample-#{name}.prayspec"), <<~SPEC)
      Package::Specification.new do |spec|
        spec.name = "sample/#{name}"
        spec.version = "1.0.0"
        spec.files = ["exports/rules.md"]
        spec.exports = {"rules" => {type: "fragment", path: "exports/rules.md"}}
      end
    SPEC
  end

  def run_pray(*arguments)
    Open3.capture3("ruby", pray_executable, "--path", workspace, *arguments)
  end
end
