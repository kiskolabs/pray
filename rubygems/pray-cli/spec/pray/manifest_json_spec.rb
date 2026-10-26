# frozen_string_literal: true

require "spec_helper"

# The canonical manifest JSON is a cross-implementation contract: manifest_hash
# in Prayfile.lock is computed from these bytes, so key order is part of it.
RSpec.describe Pray::ManifestJson do
  let(:manifest) do
    Pray.parse_manifest(<<~PRAYFILE)
      prayfile "1"

      pray do
        support_email "contact@example.com"
      end

      source "amkisko", git: "https://github.com/amkisko/prayers.git"

      compose "AGENTS.md" do
        pray "amkisko/working-rules", "~> 2.1"
      end
    PRAYFILE
  end

  it "orders symbols before render" do
    keys = described_class.manifest_fields(manifest.canonicalized).keys

    expect(keys.index("symbols")).to be < keys.index("render")
  end

  it "emits valid canonical JSON for every control character" do
    value = "\u0000\b\f\n\r\t\"\\é"
    controlled = Pray::Manifest.new(prayfile_version: "1", symbols: {"value" => value})

    encoded = described_class.encode_compact(controlled.canonicalized)

    expect(JSON.parse(encoded).fetch("symbols").fetch("value")).to eq(value)
    expect(encoded).to eq(JSON.generate(described_class.manifest_fields(controlled.canonicalized)))
  end
end
