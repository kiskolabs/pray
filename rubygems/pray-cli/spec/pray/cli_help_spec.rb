# frozen_string_literal: true

require "spec_helper"
require "open3"

RSpec.describe "pray CLI help" do
  def run_pray(*arguments)
    Open3.capture3("ruby", File.expand_path("../../bin/pray", __dir__), *arguments)
  end

  it "prints concise help for bare invocation" do
    stdout, stderr, status = run_pray
    expect(status).to be_success
    expect(stderr).to be_empty
    expect(stdout).to include("reproducible inference input")
    expect(stdout).to include("pray help")
  end

  it "prints install help via pray help install" do
    stdout, _stderr, status = run_pray("help", "install")
    expect(status).to be_success
    expect(stdout).to include("--offline")
  end

  it "prints install help via pray install --help" do
    stdout, _stderr, status = run_pray("install", "--help")
    expect(status).to be_success
    expect(stdout).to include("--offline")
  end
end
