# frozen_string_literal: true

require "spec_helper"

RSpec.describe Pray::TorrentManifest do
  it "hashes pieces at 16KiB with a sha256 prefix" do
    bytes = "a" * ((16 * 1024) + 1)
    payload = described_class.payload(
      name: "sample/base",
      version: "1.4.3",
      artifact_path: "v1/artifacts/sample/base/1.4.3/sample-base-1.4.3.praypkg",
      archive_bytes: bytes,
      trackers: ["http://tracker.example/announce"]
    )

    expect(payload["spec"]).to eq("pray-torrent-v1")
    expect(payload["piece_size"]).to eq(16_384)
    expect(payload["length"]).to eq(bytes.bytesize)
    expect(payload["pieces"].length).to eq(2)
    expect(payload["pieces"][0]).to eq(Pray::Hashing.sha256_prefixed("a" * 16_384))
    expect(payload["pieces"][1]).to eq(Pray::Hashing.sha256_prefixed("a"))
    expect(payload["artifact_hash"]).to eq(Pray::Hashing.sha256_prefixed(bytes))
    expect(payload["sources"]).to eq(["v1/artifacts/sample/base/1.4.3/sample-base-1.4.3.praypkg"])
    expect(payload["trackers"]).to eq(["http://tracker.example/announce"])
  end

  it "emits no pieces for an empty artifact" do
    payload = described_class.payload(
      name: "sample/base",
      version: "1.0.0",
      artifact_path: "v1/artifacts/empty.praypkg",
      archive_bytes: "",
      trackers: []
    )
    expect(payload["length"]).to eq(0)
    expect(payload["pieces"]).to eq([])
  end
end
