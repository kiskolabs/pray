# frozen_string_literal: true

require "spec_helper"

RSpec.describe Pray::Upstream do
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

  it "takes the new tree for a clean replica" do
    old = {"exports/a.md" => "old"}
    new = {"exports/a.md" => "new"}
    merged = described_class.merge_content_files(old, new, old)
    expect(merged["exports/a.md"]).to eq("new")
  end

  it "conflicts when a local edit overlaps an upstream change" do
    old = {"exports/a.md" => "old"}
    new = {"exports/a.md" => "new"}
    local = {"exports/a.md" => "edit"}
    expect do
      described_class.merge_content_files(old, new, local)
    end.to raise_error(Pray::Error, /exports\/a.md/)
  end

  it "names the fork and every conflicting path" do
    old = {"README.md" => "old readme", "exports/a.md" => "old"}
    new = {"README.md" => "new readme", "exports/a.md" => "new"}
    local = {"README.md" => "local readme", "exports/a.md" => "edit"}
    _merged, paths = described_class.try_merge_content_files(old, new, local)
    expect(paths).to include("README.md", "exports/a.md")
    message = described_class.merge_conflict_message(
      "fork/base", "sample/base", "1.4.3", "1.4.4", paths
    )
    expect(message).to include("fork/base")
    expect(message).to include("sample/base 1.4.3 to 1.4.4")
    expect(message).to include("README.md")
    expect(message).to include("exports/a.md")
  end

  it "keeps a local edit when upstream is unchanged" do
    old = {"exports/a.md" => "old"}
    local = {"exports/a.md" => "edit"}
    merged = described_class.merge_content_files(old, old, local)
    expect(merged["exports/a.md"]).to eq("edit")
  end

  it "rewrites an exact upstream pin to the new version" do
    expect(described_class.next_upstream_constraint("= 1.4.3", "1.4.4")).to eq("= 1.4.4")
    expect(described_class.next_upstream_constraint("1.4.3", "1.4.4")).to eq("= 1.4.4")
    expect(described_class.next_upstream_constraint("~> 1.4", "1.4.4")).to eq("~> 1.4")
  end

  it "uses a spaced equals pin for --latest" do
    expect(described_class.latest_spec_upstream_constraint("= 1.4.3", "1.4.4")).to eq("= 1.4.4")
    expect(described_class.latest_spec_upstream_constraint("~> 1.4", "2.0.0")).to eq("~> 2.0")
  end

  it "omits a file removed from upstream when local still matches old" do
    old = {"gone.md" => "old"}
    merged = described_class.merge_content_files(old, {}, old)
    expect(merged).not_to have_key("gone.md")
  end
end
