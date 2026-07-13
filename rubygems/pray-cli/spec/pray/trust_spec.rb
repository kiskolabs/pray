# frozen_string_literal: true

require "spec_helper"
require "fileutils"

RSpec.describe Pray::Trust do
  let(:workspace) { Dir.mktmpdir("pray-trust-") }

  after do
    FileUtils.rm_rf(workspace)
  end

  around do |example|
    original_home = ENV["PRAY_HOME"]
    ENV["PRAY_HOME"] = workspace
    example.run
  ensure
    if original_home
      ENV["PRAY_HOME"] = original_home
    else
      ENV.delete("PRAY_HOME")
    end
  end

  def write_policy(contents)
    FileUtils.mkdir_p(workspace)
    File.write(File.join(workspace, "trust.toml"), contents)
  end

  describe ".prepare_source_host_keys" do
    it "records configured host keys for pray_ssh sources" do
      write_policy(<<~TOML)
        [[rules]]
        match_prefix = "pray+ssh://prayers.internal"
        allowed_host_keys = ["SHA256:abc123"]
      TOML

      sources = [
        Pray::ManifestSource.new(
          name: "team",
          kind: "pray_ssh",
          url: "pray+ssh://prayers.internal:2222/var/lib/pray"
        )
      ]

      expect(described_class.prepare_source_host_keys(sources)).to eq(
        "team" => "SHA256:abc123"
      )
    end
  end

  describe ".verify_publisher_fingerprint!" do
    let(:selected) do
      Pray::RegistryPackageVersion.new(
        version: "1.0.0",
        signature: "sha256:signature",
        signer_fingerprint: "SHA256:publisher"
      )
    end

    it "allows trusted publisher fingerprints" do
      write_policy(<<~TOML)
        [[rules]]
        match_prefix = "https://registry.example"
        allowed_publishers = ["SHA256:publisher"]
      TOML

      expect do
        described_class.verify_publisher_fingerprint!("https://registry.example", selected)
      end.not_to raise_error
    end

    it "rejects untrusted publisher fingerprints when policy is configured" do
      write_policy(<<~TOML)
        [[rules]]
        match_prefix = "https://registry.example"
        allowed_publishers = ["SHA256:other"]
      TOML

      expect do
        described_class.verify_publisher_fingerprint!("https://registry.example", selected)
      end.to raise_error(Pray::Error, /not trusted/)
    end
  end
end
