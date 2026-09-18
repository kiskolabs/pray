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
end
