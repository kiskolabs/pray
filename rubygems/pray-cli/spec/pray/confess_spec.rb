# frozen_string_literal: true

require "spec_helper"
require "fileutils"
require_relative "../support/http_fixture_server"

RSpec.describe Pray::Confess do
  let(:workspace) { Dir.mktmpdir("pray-confess-") }

  after { FileUtils.rm_rf(workspace) }

  def write_project
    FileUtils.mkdir_p(File.join(workspace, "packages", "base"))
    File.write(File.join(workspace, "Prayfile"), <<~PRAY)
      prayfile "1"
      compose "AGENTS.md" do
        pray "sample/base", "~> 1.0", path: "packages/base"
      end
    PRAY
    File.write(File.join(workspace, "packages", "base", "base.prayspec"), <<~SPEC)
      Package::Specification.new do |spec|
        spec.name = "sample/base"
        spec.version = "1.0.0"
        spec.summary = "fixture"
        spec.files = ["README.md"]
        spec.exports = {
          "default" => {
            type: "fragment",
            path: "README.md",
            summary: "default"
          }
        }
      end
    SPEC
    File.write(File.join(workspace, "packages", "base", "README.md"), "hello\n")
  end

  it "submits an accepted confession over HTTP" do
    submissions = []
    fixture = HttpFixtureServer.start(
      "POST /v1/confessions" => lambda { |body, _headers|
        submissions << JSON.parse(body)
        ["200 OK", "application/json", JSON.generate({"status" => "ok"})]
      }
    )
    write_project

    expect do
      described_class.submit(
        package: "sample/base", from_lock: nil, version: nil,
        accepted: true, rejected: false, note: "ok", url: fixture[:url],
        project_root: workspace
      )
    end.to output(%r{Confession submitted for sample/base 1.0.0}).to_stdout

    expect(submissions.length).to eq(1)
    expect(submissions.first["status"]).to eq("accepted")
    expect(submissions.first["signature"]).to start_with("sha256:")
  ensure
    HttpFixtureServer.stop(fixture) if fixture
  end

  it "rejects missing status flags and missing packages" do
    write_project
    expect do
      described_class.submit(
        package: "sample/base", from_lock: nil, version: nil,
        accepted: false, rejected: false, note: nil, url: "http://example",
        project_root: workspace
      )
    end.to raise_error(Pray::Error, /accepted or --rejected/)

    expect do
      described_class.submit(
        package: "missing/pkg", from_lock: nil, version: nil,
        accepted: true, rejected: false, note: nil, url: "http://example",
        project_root: workspace
      )
    end.to raise_error(Pray::Error, /package missing\/pkg not found/)
  end
end
