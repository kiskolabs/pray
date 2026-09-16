# frozen_string_literal: true

require "spec_helper"
require "fileutils"

RSpec.describe Pray::Distribution do
  it "treats a missing file as empty protocols" do
    settings = described_class.read_settings("/no/such/root")
    expect(settings.allows_torrent?).to be(false)
  end

  it "opts in to torrent when listed" do
    settings = described_class.parse_settings(
      '{"spec":"pray-distribution-config-1","protocols":["torrent"]}'
    )
    expect(settings.allows_torrent?).to be(true)
  end

  it "rejects an unknown protocol" do
    expect do
      described_class.parse_settings(
        '{"spec":"pray-distribution-config-1","protocols":["ipfs"]}'
      )
    end.to raise_error(Pray::Error, /unsupported protocol/)
  end

  it "rejects a leftover sidecars field" do
    expect do
      described_class.parse_settings(
        '{"spec":"pray-distribution-config-1","sidecars":["torrent"]}'
      )
    end.to raise_error(Pray::Error, /unsupported field: sidecars/)
  end

  it "writes empty protocols" do
    root = Dir.mktmpdir("pray-distribution-")
    begin
      described_class.write_settings(root)
      settings = described_class.read_settings(root)
      expect(settings.protocols).to eq([])
      expect(settings.allows_torrent?).to be(false)
    ensure
      FileUtils.rm_rf(root)
    end
  end

  it "repo init writes empty protocols" do
    root = Dir.mktmpdir("pray-repo-init-")
    begin
      Dir.chdir(root) { Pray::CLI.run(["repo", "init"]) }
      path = File.join(root, "prayers", "v1", "distribution.json")
      data = JSON.parse(File.read(path))
      expect(data["spec"]).to eq("pray-distribution-config-1")
      expect(data["protocols"]).to eq([])
    ensure
      FileUtils.rm_rf(root)
    end
  end
end
