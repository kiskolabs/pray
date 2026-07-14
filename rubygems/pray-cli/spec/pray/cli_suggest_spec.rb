# frozen_string_literal: true

require "spec_helper"
require "open3"

RSpec.describe "pray CLI suggestions" do
  def run_pray(*arguments)
    Open3.capture3("ruby", File.expand_path("../../bin/pray", __dir__), *arguments)
  end

  it "suggests install for instal typo" do
    _stdout, stderr, status = run_pray("instal")
    expect(status.exitstatus).to eq(2)
    expect(stderr).to include("usage error:")
    expect(stderr).to include("Did you mean `install`?")
    expect(stderr).not_to include("unsupported feature")
  end

  describe Pray::CLI::Suggest do
    it "suggests install for instal" do
      expect(described_class.suggest_command("instal", described_class::TOP_LEVEL_COMMANDS)).to eq("install")
    end
  end
end
