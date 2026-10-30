# frozen_string_literal: true

require "spec_helper"
require "fileutils"
require "open3"

RSpec.describe "related prayer references" do
  it "shows canonical references in package explanation" do
    root = Dir.mktmpdir("pray-related-")
    FileUtils.mkdir_p(File.join(root, "prayers", "base"))
    File.write(
      File.join(root, "Prayfile"),
      "prayfile \"1\"\nsource \"local\", path: \"prayers\"\npray \"sample/base\", path: \"prayers/base\"\n"
    )
    File.write(File.join(root, "prayers", "base", "README.md"), "base\n")
    File.write(
      File.join(root, "prayers", "base", "sample-base.prayspec"),
      <<~PRAYSPEC
        Package::Specification.new do |spec|
          spec.name = "sample/base"
          spec.related = ["sample/testing", " ../security "]
          spec.files = ["README.md"]
        end
      PRAYSPEC
    )
    executable = File.expand_path("../../bin/pray", __dir__)

    stdout, stderr, status = Open3.capture3("ruby", executable, "explain", "sample/base", chdir: root)

    expect(status).to be_success, stderr
    expect(stdout).to include("related prayers: ../security, sample/testing")
  ensure
    FileUtils.rm_rf(root) if root
  end
end
