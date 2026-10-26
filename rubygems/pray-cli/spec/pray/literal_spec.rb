# frozen_string_literal: true

require "spec_helper"

RSpec.describe Pray::Literal do
  it "finds top-level tokens after Unicode and ignores nested tokens" do
    input = '"ää:ää", metadata: { nested: true }'

    expect(described_class.find_top_level(input, ":")).to eq(17)
    expect(input[17]).to eq(":")
  end
end
