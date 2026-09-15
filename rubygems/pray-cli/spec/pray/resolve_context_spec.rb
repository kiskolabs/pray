# frozen_string_literal: true

require "spec_helper"

RSpec.describe Pray::ResolveOptions do
  let(:lockfile) do
    package = Struct.new(:name, :version).new("sample/base", "1.4.3")
    Struct.new(:package).new([package])
  end

  it "keeps the locked version when the package is not unlocked" do
    options = described_class.new
    expect(options.preferred_lock_version(lockfile, "sample/base")).to eq("1.4.3")
  end

  it "ignores locked versions during latest updates" do
    options = described_class.new(ignore_locked_versions: true)
    expect(options.preferred_lock_version(lockfile, "sample/base")).to be_nil
  end

  it "ignores the locked version for an unlocked package" do
    options = described_class.new(unlocked_packages: Set["sample/base"])
    expect(options.preferred_lock_version(lockfile, "sample/base")).to be_nil
  end
end
