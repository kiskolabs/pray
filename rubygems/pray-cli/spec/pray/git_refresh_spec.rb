# frozen_string_literal: true

require "spec_helper"

RSpec.describe Pray::GitRefresh do
  it "treats a missing catalog file as a git refresh candidate" do
    error = Pray::Error.resolution(
      "package sample/extra not found in distribution. Missing v1/packages/sample/extra.json. Check the package name."
    )
    expect(described_class.resolution_may_benefit_from_git_source_refresh?(error)).to be true
  end

  it "rewrites a git catalog miss to name the revision and pray update" do
    error = Pray::Error.resolution(
      "package sample/extra not found in distribution. Check the package name, version constraint `~> 1.0`, and that the source publishes registry metadata."
    )
    annotated = described_class.annotate_missing_git_catalog(
      error,
      package_name: "sample/extra",
      source_name: "dist",
      revision: "abc123"
    )
    expect(annotated.message).to include("abc123")
    expect(annotated.message).to include("pray update")
    expect(annotated.message).not_to include("check the package name")
  end
end
