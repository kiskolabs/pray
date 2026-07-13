# frozen_string_literal: true

require "spec_helper"
require "fileutils"
require "json"

RSpec.describe Pray::Registry do
  let(:workspace) { Dir.mktmpdir("pray-registry-") }
  let(:distribution_root) { File.join(workspace, "dist") }

  after do
    FileUtils.rm_rf(workspace)
  end

  describe ".fetch_package_metadata" do
    before do
      metadata_dir = File.join(distribution_root, "v1", "packages", "sample")
      FileUtils.mkdir_p(metadata_dir)
      File.write(
        File.join(metadata_dir, "base.json"),
        JSON.generate("name" => "sample/base", "versions" => [])
      )
    end

    it "reads scoped package metadata under the distribution root" do
      metadata = described_class.fetch_package_metadata(distribution_root, "sample/base")
      expect(metadata.name).to eq("sample/base")
    end

    it "rejects package names that escape the metadata directory" do
      expect do
        described_class.fetch_package_metadata(distribution_root, "../../outside")
      end.to raise_error(Pray::Error, /invalid package name/)
    end
  end

  describe ".validate_and_unpack" do
    let(:declaration) { Pray::ManifestPackage.new(name: "demo", constraint: "1.0.0") }
    let(:cache_directory) { File.join(workspace, "cache") }
    let(:package_root) do
      root = File.join(workspace, "package")
      FileUtils.mkdir_p(root)
      File.write(
        File.join(root, "demo.prayspec"),
        <<~PRAYSPEC
          Package::Specification.new do |spec|
            spec.name = "demo"
            spec.version = "1.0.0"
            spec.files = []
          end
        PRAYSPEC
      )
      root
    end
    let(:spec) { Pray.parse_package_spec(File.read(File.join(package_root, "demo.prayspec"))).canonicalized }
    let(:tree_hash) { spec.tree_hash_for_root(package_root) }
    let(:package) do
      Pray::ResolvedPackage.new(
        declaration: declaration,
        root: package_root,
        spec: spec,
        tree_hash: tree_hash,
        selected_exports: []
      )
    end
    let(:artifact_bytes) { Pray::Archive.build_package_archive_bytes(package) }
    let(:selected) do
      Pray::RegistryPackageVersion.new(
        version: "1.0.0",
        artifact_hash: Pray::Hashing.sha256_prefixed(artifact_bytes),
        tree_hash: tree_hash,
        signer: "local",
        signature: described_class.registry_artifact_signature(artifact_bytes, tree_hash, "local")
      )
    end

    before do
      package_root
      FileUtils.mkdir_p(cache_directory)
    end

    it "accepts matching registry signatures" do
      expect do
        described_class.validate_and_unpack(cache_directory, declaration, selected, artifact_bytes)
      end.not_to raise_error
    end

    it "rejects registry signatures that do not match the artifact" do
      tampered = selected.dup
      tampered.signature = "sha256:deadbeef"

      expect do
        described_class.validate_and_unpack(cache_directory, declaration, tampered, artifact_bytes)
      end.to raise_error(Pray::Error, /signature mismatch/)
    end

    it "rejects untrusted publisher fingerprints when policy is configured" do
      original_home = ENV["PRAY_HOME"]
      trust_home = Dir.mktmpdir("pray-registry-trust-")
      ENV["PRAY_HOME"] = trust_home
      FileUtils.mkdir_p(trust_home)
      File.write(
        File.join(trust_home, "trust.toml"),
        <<~TOML
          [[rules]]
          match_prefix = "local"
          allowed_publishers = ["SHA256:other"]
        TOML
      )
      signed = selected.dup
      signed.signer_fingerprint = "SHA256:publisher"

      expect do
        described_class.validate_and_unpack(
          cache_directory,
          declaration,
          signed,
          artifact_bytes,
          source_url: "local"
        )
      end.to raise_error(Pray::Error, /not trusted/)
    ensure
      FileUtils.rm_rf(trust_home)
      if original_home
        ENV["PRAY_HOME"] = original_home
      else
        ENV.delete("PRAY_HOME")
      end
    end
  end
end
