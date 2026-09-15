# frozen_string_literal: true

require "spec_helper"

RSpec.describe Pray::Constraint do
  describe ".normalize_version_constraint" do
    it "pins bare semver versions" do
      expect(described_class.normalize_version_constraint("1.4.3")).to eq("=1.4.3")
    end

    it "preserves operator-prefixed constraints" do
      expect(described_class.normalize_version_constraint("~> 1.4")).to eq("~> 1.4")
    end
  end

  describe ".version_satisfies" do
    it "accepts versions inside pessimistic constraints" do
      expect(described_class.version_satisfies("1.4.3", "~> 1.4")).to be(true)
    end

    it "rejects versions outside pessimistic constraints" do
      expect(described_class.version_satisfies("2.0.0", "~> 1.4")).to be(false)
    end

    it "accepts any version for wildcard constraints" do
      expect(described_class.version_satisfies("9.9.9", "*")).to be(true)
    end
  end

  describe ".latest_constraint_for_package" do
    it "keeps the operator family for a newer registry version" do
      expect(described_class.latest_constraint_for_package("~> 1.0", "2.0.0")).to eq("~> 2.0")
      expect(described_class.latest_constraint_for_package("1.0.0", "2.0.0")).to eq("=2.0.0")
      expect(described_class.latest_constraint_for_package("^1.0", "2.1.0")).to eq("^2.1")
    end
  end
end
