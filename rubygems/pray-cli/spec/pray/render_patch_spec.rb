# frozen_string_literal: true

require "spec_helper"

RSpec.describe Pray::RenderPatch do
  it "appends a trailing managed span with the blank line from fresh" do
    existing = <<~TEXT
      <!-- pray:abc123 -->
      package body
      <!-- pray:abc123 -->
    TEXT
    fresh = <<~TEXT
      <!-- pray:abc123 -->
      package body
      <!-- pray:abc123 -->

      <!-- pray:local001 -->
      new local body
      <!-- pray:local001 -->
    TEXT
    expect(described_class.patch_rendered_content(existing, fresh)).to eq(fresh)
  end

  it "restores a missing blank line between adjacent managed spans" do
    existing = <<~TEXT
      <!-- pray:abc123 -->
      package body
      <!-- pray:abc123 -->
      <!-- pray:local001 -->
      new local body
      <!-- pray:local001 -->
    TEXT
    fresh = <<~TEXT
      <!-- pray:abc123 -->
      package body
      <!-- pray:abc123 -->

      <!-- pray:local001 -->
      new local body
      <!-- pray:local001 -->
    TEXT
    expect(described_class.patch_rendered_content(existing, fresh)).to eq(fresh)
  end

  it "drops an unchanged managed span retired by the previous lock" do
    existing = managed_pair("kept001", "kept body") + managed_pair("retired1", "retired body")
    fresh = managed_pair("kept001", "kept body")
    retired = {"retired1" => Pray::Hashing.checksum_managed_span_content("retired body\n")}

    expect(described_class.patch_rendered_content(existing, fresh, retired)).to eq(fresh)
  end

  it "preserves edited retired spans and unknown spans" do
    fresh = managed_pair("kept001", "kept body")
    existing = fresh + managed_pair("retired1", "edited body") + managed_pair("unknown1", "unknown body")
    retired = {"retired1" => Pray::Hashing.checksum_managed_span_content("original body\n")}

    patched = described_class.patch_rendered_content(existing, fresh, retired)

    expect(patched).to include("edited body")
    expect(patched).to include("unknown body")
  end

  it "preserves edited and unknown spans when every current span was retired" do
    existing = managed_pair("retired1", "edited body") + managed_pair("unknown1", "unknown body")
    retired = {"retired1" => Pray::Hashing.checksum_managed_span_content("original body\n")}

    patched = described_class.patch_rendered_content(existing, "", retired)

    expect(patched).to include("edited body")
    expect(patched).to include("unknown body")
  end

  def managed_pair(identifier, body)
    "<!-- pray:#{identifier} -->\n#{body}\n<!-- pray:#{identifier} -->\n"
  end
end
