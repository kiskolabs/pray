# frozen_string_literal: true

require "spec_helper"

RSpec.describe Pray::LocalPrayer do
  it "accepts a single folder name" do
    expect(described_class.validate_name!("project")).to eq("project")
  end

  it "strips a matching source handle from a path package directory" do
    expect(described_class.path_source_package_directory("local", "local/project")).to eq("project")
    expect(described_class.path_source_package_directory("amkisko", "amkisko/rules")).to eq("rules")
    expect(described_class.path_source_package_directory("local", "amkisko/rules")).to eq("amkisko-rules")
  end

  it "refuses the distribution layout name" do
    expect { described_class.validate_name!("v1") }.to raise_error(Pray::Error, /reserved/)
  end
end
