# frozen_string_literal: true

require "spec_helper"

RSpec.describe "Pray::CLI#remove_manifest_statement" do
  let(:helpers) { Object.new.extend(Pray::CLI) }

  it "removes a pray declaration written by add" do
    text = <<~PRAYFILE
      prayfile "1"
      pray "sample/base", path: "packages/base"
      render mode: :managed
    PRAYFILE

    updated = helpers.remove_manifest_statement(text, "sample/base")
    expect(updated).not_to include("sample/base")
    expect(updated).to include("render mode: :managed")
  end
end
