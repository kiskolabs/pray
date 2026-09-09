# frozen_string_literal: true

require "spec_helper"

RSpec.describe Pray::Render::ContentBuilder do
  describe "#finish" do
    it "collapses trailing blank lines to a single newline" do
      builder = described_class.new
      builder.append_line("body")
      builder.append_empty_line

      expect(builder.finish).to eq("body\n")
    end

    it "terminates unterminated content" do
      builder = described_class.new
      builder.append_body("body")

      expect(builder.finish).to eq("body\n")
    end
  end
end
