# frozen_string_literal: true

require "spec_helper"
require "fileutils"

RSpec.describe Pray::ServeFederation do
  let(:workspace) { Dir.mktmpdir("pray-federation-") }

  after { FileUtils.rm_rf(workspace) }

  it "serves discovery index package and confession endpoints" do
    FileUtils.mkdir_p(File.join(workspace, "v1", "packages", "sample"))
    File.write(
      File.join(workspace, "v1", "index.json"),
      JSON.generate({"spec" => "prayfile-distribution-1", "packages" => ["sample/base"]})
    )
    File.write(
      File.join(workspace, "v1", "packages", "sample", "base.json"),
      JSON.generate(
        "name" => "sample/base",
        "versions" => [{
          "version" => "1.0.0",
          "artifact" => "v1/artifacts/sample/base/1.0.0/pkg.praypkg",
          "artifact_hash" => "sha256:abc",
          "tree_hash" => "sha256:def",
          "yanked" => false,
          "targets" => [],
          "exports" => [],
          "published_at" => "2026-01-01T00:00:00Z",
          "signer" => "local",
          "signature" => "sha256:sig"
        }]
      )
    )
    File.write(
      File.join(workspace, "v1", "peers.json"),
      JSON.generate([{"name" => "self", "url" => "http://example", "public" => true}])
    )

    discovery = described_class.discovery_response(workspace)
    expect(discovery).to include("pray-federation-v1")
    expect(discovery).to include("http://example")

    index = described_class.index_response(workspace)
    expect(index).to include("sample/base")

    package = described_class.package_response(workspace, "/v1/sync/package/sample/base")
    expect(package).to include("1.0.0")

    confession = described_class.append_confession(
      workspace, JSON.generate({"package" => "sample/base", "status" => "accepted"})
    )
    expect(confession).to include("200 OK")
    expect(File.read(File.join(workspace, "v1", "confessions.jsonl"))).to include("accepted")

    via_dispatch = Pray::Serve.dispatch_request(
      workspace, "GET", "/.well-known/pray-federation.json"
    )
    expect(via_dispatch).to include("pray-federation-v1")
    expect(Pray::Serve.dispatch_request(workspace, "GET", "/v1/sync/index")).to include(
      "sample/base"
    )
    expect(
      Pray::Serve.dispatch_request(workspace, "GET", "/v1/sync/package/sample/base")
    ).to include("artifact_hash")
    expect(
      Pray::Serve.dispatch_request(
        workspace, "POST", "/v1/confessions",
        JSON.generate({"package" => "sample/base", "status" => "rejected"})
      )
    ).to include("200 OK")
  end
end
