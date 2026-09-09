# frozen_string_literal: true

require "spec_helper"

RSpec.describe Pray::PackageSpec do
  it "matches the shared byte-order fixture" do
    fixture_path = File.expand_path("../../../../testdata/shared/package-tree/byte-order.json", __dir__)
    fixture = JSON.parse(File.read(fixture_path, encoding: "UTF-8"))
    files = fixture.fetch("files").to_h do |file|
      [file.fetch("path"), file.fetch("content")]
    end

    expect(described_class.tree_hash_from_file_bytes(files)).to eq(fixture.fetch("tree_hash"))
  end
end
