# frozen_string_literal: true

require "spec_helper"

RSpec.describe Pray::Substitute do
  it "replaces known symbols" do
    text = described_class.substitute_pray_symbols(
      "write ((pray:support_email)) or ((pray:security_email))",
      {
        "support_email" => "a@example.com",
        "security_email" => "b@example.com"
      }
    )
    expect(text).to eq("write a@example.com or b@example.com")
  end

  it "rejects unknown symbols" do
    expect do
      described_class.substitute_pray_symbols("((pray:missing))", {})
    end.to raise_error(Pray::Error, /unknown pray symbol/)
  end

  it "ignores spaced placeholder forms" do
    text = "(( pray:email )) ((pray : email))"
    expect(described_class.substitute_pray_symbols(text, {"email" => "a@example.com"})).to eq(text)
  end
end
