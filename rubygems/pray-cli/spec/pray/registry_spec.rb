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

  describe ".registry_cache_directory" do
    it "matches the shared cache identity fixture" do
      fixture_path = File.expand_path("../../../../testdata/shared/registry-cache/identity-first.json", __dir__)
      fixture = JSON.parse(File.read(fixture_path))

      path = described_class.registry_cache_directory(
        workspace,
        fixture.fetch("source_key"),
        fixture.fetch("package_name"),
        fixture.fetch("version")
      )

      expect(path).to eq(File.join(workspace, fixture.fetch("relative_path")))
    end

    it "rejects unsafe package and version segments" do
      %w[sample sample/base/extra sample//base ./base ../base sample/.. sample\\base].each do |package_name|
        expect do
          described_class.registry_cache_directory(workspace, "source", package_name, "1.0.0")
        end.to raise_error(Pray::Error)
      end

      ["", ".", "..", "1/2", "1\\2"].each do |version|
        expect do
          described_class.registry_cache_directory(workspace, "source", "sample/base", version)
        end.to raise_error(Pray::Error)
      end
    end
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
    let(:declaration) { Pray::ManifestPackage.new(name: "sample/demo", constraint: "1.0.0") }
    let(:cache_directory) { File.join(workspace, "cache") }
    let(:package_root) do
      root = File.join(workspace, "package")
      FileUtils.mkdir_p(root)
      File.write(
        File.join(root, "demo.prayspec"),
        <<~PRAYSPEC
          Package::Specification.new do |spec|
            spec.name = "sample/demo"
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

    it "rejects missing mandatory integrity hashes" do
      missing_artifact_hash = selected.dup
      missing_artifact_hash.artifact_hash = nil
      missing_tree_hash = selected.dup
      missing_tree_hash.tree_hash = nil

      expect do
        described_class.validate_and_unpack(
          cache_directory, declaration, missing_artifact_hash, artifact_bytes, source_url: "https://registry.example"
        )
      end.to raise_error(Pray::Error, /missing artifact_hash/)
      expect do
        described_class.validate_and_unpack(
          cache_directory, declaration, missing_tree_hash, artifact_bytes, source_url: "https://registry.example"
        )
      end.to raise_error(Pray::Error, /missing tree_hash/)
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

    it "retries unpack after a failed install left an empty cache directory" do
      artifact_path = File.join(distribution_root, "artifacts", "demo-1.0.0.praypkg")
      FileUtils.mkdir_p(File.dirname(artifact_path))
      File.binwrite(artifact_path, artifact_bytes)
      metadata_dir = File.join(distribution_root, "v1", "packages", "sample")
      FileUtils.mkdir_p(metadata_dir)
      File.write(
        File.join(metadata_dir, "demo.json"),
        JSON.generate(
          "name" => "sample/demo",
          "versions" => [{
            "version" => "1.0.0",
            "artifact" => "artifacts/demo-1.0.0.praypkg",
            "artifact_hash" => selected.artifact_hash,
            "tree_hash" => tree_hash
          }]
        )
      )

      cache_directory = described_class.registry_cache_directory(
        workspace, "local", "sample/demo", "1.0.0"
      )
      FileUtils.mkdir_p(cache_directory)

      resolved = described_class.resolve_local_registry_package_root(
        workspace, "local", distribution_root, declaration
      )
      expect(File).to exist(File.join(resolved.root, "demo.prayspec"))
      expect(File).not_to exist("#{resolved.root}.staging")
    end
  end

  describe ".cache_ready?" do
    it "treats an empty cache directory as not ready" do
      cache_directory = File.join(workspace, "empty-cache")
      FileUtils.mkdir_p(cache_directory)
      selected = Pray::RegistryPackageVersion.new(version: "1.0.0")

      expect(described_class.cache_ready?(cache_directory, selected)).to be(false)
    end

    it "rejects cached content whose tree hash changed" do
      cache_directory = File.join(workspace, "changed-cache")
      FileUtils.mkdir_p(cache_directory)
      File.write(
        File.join(cache_directory, "demo.prayspec"),
        "Package::Specification.new do |spec|\n  spec.name = \"demo\"\n  spec.version = \"1.0.0\"\n  spec.files = []\nend\n"
      )
      selected = Pray::RegistryPackageVersion.new(version: "1.0.0", tree_hash: "sha256:wrong")

      expect(described_class.cache_ready?(cache_directory, selected)).to be(false)
    end
  end

  describe ".read_artifact_bytes" do
    it "rejects absolute remote artifact URLs" do
      expect do
        described_class.read_artifact_bytes("https://registry.example", "https://evil.example/pkg.praypkg")
      end.to raise_error(Pray::Error, /must be relative/)
    end
  end
end
