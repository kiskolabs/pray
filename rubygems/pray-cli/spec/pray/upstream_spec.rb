# frozen_string_literal: true

require "spec_helper"

RSpec.describe Pray::Upstream do
  it "refuses an update before writing an unsupported path refresh" do
    declaration = Pray::ManifestPackage.new(name: "fork/base", path: "packages/base")
    spec = Pray::PackageSpec.new(
      name: "fork/base",
      upstream: Pray::PackageUpstream.new(name: "sample/base", constraint: "~> 1.4")
    )
    package = Pray::ResolvedPackage.new(declaration: declaration, spec: spec)

    expect do
      described_class.ensure_update_supported!([package])
    end.to raise_error(Pray::Error, /cannot refresh upstream package fork\/base/)
  end

  it "rejects content that no longer matches the locked upstream" do
    locked = Pray::LockedUpstream.new(
      name: "sample/base", version: "1.4.3", source: "sample",
      tree_hash: "sha256:old", artifact_hash: "sha256:artifact"
    )
    resolved = locked.dup
    resolved.tree_hash = "sha256:changed"

    expect do
      described_class.ensure_locked_match!(locked, resolved)
    end.to raise_error(Pray::Error, /locked upstream tree hash mismatch/)
  end
end
