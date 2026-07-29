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
    expect(stdout).to include("Usage: pray [OPTIONS] <COMMAND>")
    expect(stdout).to include("See 'pray help <command>'")
    expect(stdout).to include("Options:")
    expect(stdout).not_to include("Documentation:")
    expect(stdout).not_to include("Exit codes:")
  end

  it "prints install help via pray help install" do
    stdout, _stderr, status = run_pray("help", "install")
    expect(status).to be_success
    expect(stdout).to include("--offline")
    expect(stdout).not_to include("Documentation:")
  end

  it "prints install help via pray install --help" do
    stdout, _stderr, status = run_pray("install", "--help")
    expect(status).to be_success
    expect(stdout).to include("--offline")
  end

  it "prints help for listed commands" do
    %w[remove list format version].each do |command|
      stdout, stderr, status = run_pray("help", command)
      expect(status).to be_success, "help #{command}: #{stderr}"
      expect(stdout).to include("Usage: pray")
      expect(stdout).not_to include("unknown command")
    end
  end
end
