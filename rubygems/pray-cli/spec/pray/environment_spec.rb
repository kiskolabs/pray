# frozen_string_literal: true

require "spec_helper"
require "fileutils"

RSpec.describe Pray::Environment do
  def package_with_groups(groups)
    Pray::ManifestPackage.new(name: "sample/base", groups: groups)
  end

  it "always renders ungrouped packages" do
    package = package_with_groups([])
    expect(described_class.should_render_package?(package, nil)).to be(true)
    expect(described_class.should_render_package?(package, "development")).to be(true)
  end

  it "renders grouped packages only for the selected environment" do
    package = package_with_groups(%w[development test])
    expect(described_class.should_render_package?(package, nil)).to be(false)
    expect(described_class.should_render_package?(package, "development")).to be(true)
    expect(described_class.should_render_package?(package, "test")).to be(true)
    expect(described_class.should_render_package?(package, "production")).to be(false)
  end

  it "rejects unknown environments when groups exist" do
    manifest = Pray::Manifest.new(
      prayfile_version: "1",
      packages: [package_with_groups(["development"])]
    )

    expect do
      described_class.validate_environment(manifest, "production")
    end.to raise_error(Pray::Error, /unknown environment production/)
  end
end
