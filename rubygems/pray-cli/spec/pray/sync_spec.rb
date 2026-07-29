# frozen_string_literal: true

require "spec_helper"
require "fileutils"

RSpec.describe Pray::Sync do
  let(:workspace) { Dir.mktmpdir("pray-sync-") }

  after { FileUtils.rm_rf(workspace) }

  def build_package_fixture(root)
    FileUtils.mkdir_p(File.join(root, "packages", "base"))
    File.write(File.join(root, "Prayfile"), <<~PRAY)
      prayfile "1"
      compose "AGENTS.md" do
        pray "sample/base", "~> 1.0", path: "packages/base"
      end
    PRAY
    File.write(File.join(root, "packages", "base", "base.prayspec"), <<~SPEC)
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
    File.write(File.join(root, "packages", "base", "README.md"), "base\n")
  end

  it "pulls packages from an HTTP peer into the destination root" do
    source = File.join(workspace, "source")
    upstream = File.join(workspace, "upstream")
    downstream = File.join(workspace, "downstream")
    FileUtils.mkdir_p([source, upstream, downstream])
    build_package_fixture(source)
    project = Pray::Resolve.resolve_project(File.join(source, "Prayfile"))
    Pray::Publish.publish_to_root(project, upstream)

    server = TCPServer.new("127.0.0.1", 0)
    port = server.addr[1]
    thread = Thread.new do
      loop do
        socket = server.accept
        Thread.new do
          Pray::Serve.handle_connection(upstream, socket)
        ensure
          socket.close unless socket.closed?
        end
      rescue
        break
      end
    end

    peer_url = "http://127.0.0.1:#{port}"
    FileUtils.mkdir_p(File.join(downstream, "v1"))
    File.write(
      File.join(downstream, "v1", "peers.json"),
      JSON.pretty_generate([{"name" => "upstream", "url" => peer_url, "public" => true}])
    )

    peers = described_class.load_sync_peers(downstream).map { |peer| peer["url"] }
    summary = described_class.synchronize_registry(downstream, peers)
    expect(summary[:packages]).to eq(1)
    expect(summary[:peers]).to eq(1)

    expect(File.read(File.join(downstream, "v1", "index.json"))).to include("sample/base")
    expect(File).to exist(File.join(downstream, "v1", "packages", "sample", "base.json"))
  ensure
    server&.close
    thread&.kill
  end

  it "requires configured peers and rejects pray_ssh peers" do
    root = File.join(workspace, "empty")
    FileUtils.mkdir_p(root)
    expect { described_class.load_sync_peers(root) }.to raise_error(Pray::Error, /no federation peers/)
    expect do
      described_class.synchronize_registry(root, ["pray+ssh://example/path"])
    end.to raise_error(Pray::Error, /pray_ssh sync peers/)
  end
end
