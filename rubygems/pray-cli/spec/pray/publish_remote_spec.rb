# frozen_string_literal: true

require "spec_helper"

RSpec.describe "Prayfile publish remotes" do
  describe Pray::PublishRemote do
    it "parses path and url remotes" do
      manifest = Pray.parse_manifest(<<~PRAY)
        prayfile "1"
        source "local", path: "guidance"
        publish "prayers", path: "prayers"
        publish "public", "https://prayers.example"
        compose "AGENTS.md" do
          pray "local/project"
        end
      PRAY
      expect(manifest.publish_remotes.length).to eq(2)
      expect(manifest.publish_remotes[0].name).to eq("prayers")
      expect(manifest.publish_remotes[0].path).to eq("prayers")
      expect(manifest.publish_remotes[1].url).to eq("https://prayers.example")
    end

    it "parses a package block" do
      manifest = Pray.parse_manifest(<<~PRAY)
        prayfile "1"
        publish "prayers", path: "prayers" do
          pray "local/project"
        end
        pray "local/project", path: "guidance/project"
      PRAY
      expect(manifest.publish_remotes[0].packages).to eq(["local/project"])
    end

    it "parses a URL that ends with do" do
      manifest = Pray.parse_manifest(<<~PRAY)
        prayfile "1"
        publish "public", "https://prayers.example/do"
      PRAY
      expect(manifest.publish_remotes[0].url).to eq("https://prayers.example/do")
      expect(manifest.publish_remotes[0].packages).to eq([])
    end

    it "rejects signing_key on publish" do
      expect {
        Pray.parse_manifest(<<~PRAY)
          prayfile "1"
          publish "prayers", path: "prayers", signing_key: "secret.pem"
        PRAY
      }.to raise_error(Pray::Error, /does not take git:/)
    end

    it "publishes path packages only from a mixed manifest" do
      manifest = Pray.parse_manifest(<<~PRAY)
        prayfile "1"
        source "local", path: "guidance"
        pray "local/project"
        pray "amkisko/rules", git: "https://example.com/rules.git"
      PRAY
      expect(described_class.path_owned_package_names(manifest)).to eq(["local/project"])
    end
  end

  describe Pray::PublishSelect do
    def remote(name, path: nil, url: nil)
      Pray::ManifestPublishRemote.new(name: name, path: path, url: url, packages: [])
    end

    it "requires dest flags when no remotes are declared" do
      expect {
        described_class.resolve_publish_destinations(
          [],
          described_class::PublishCliDest.new,
          "/tmp/project"
        )
      }.to raise_error(Pray::Error, /--root PATH or --server URL/)
    end

    it "fills declared remotes without flags" do
      dests = described_class.resolve_publish_destinations(
        [remote("prayers", path: "prayers"), remote("public", url: "https://prayers.example")],
        described_class::PublishCliDest.new,
        "/tmp/project"
      )
      expect(dests.length).to eq(2)
      expect(dests[0].root).to eq("/tmp/project/prayers")
      expect(dests[1].server).to eq("https://prayers.example")
    end

    it "rejects an undeclared root" do
      expect {
        described_class.resolve_publish_destinations(
          [remote("prayers", path: "prayers")],
          described_class::PublishCliDest.new(roots: ["other"]),
          "/tmp/project"
        )
      }.to raise_error(Pray::Error, /not a declared publish remote/)
    end
  end
end
