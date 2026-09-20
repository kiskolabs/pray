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
    expect(stdout).to include("spec.upstream")
    expect(stdout).not_to include("Documentation:")
  end

  it "prints install help via pray install --help" do
    stdout, _stderr, status = run_pray("install", "--help")
    expect(status).to be_success
    expect(stdout).to include("--offline")
  end

  it "prints update help for constraint rewrites" do
    stdout, _stderr, status = run_pray("help", "update")
    expect(status).to be_success
    expect(stdout).to include("latest package versions")
    expect(stdout).to include("spec.upstream")
    expect(stdout).to include("pray install")
    expect(stdout).to include("local compose")
  end

  it "describes update on the packages list" do
    stdout, _stderr, status = run_pray("--help")
    expect(status).to be_success
    expect(stdout).to include("re-resolve packages and git sources within constraints")
  end

  it "names dest versus lock on verify and dest write on render" do
    stdout, _stderr, status = run_pray("help", "verify")
    expect(status).to be_success
    expect(stdout).to include("dest managed spans versus Prayfile.lock")
    stdout, _stderr, status = run_pray("help", "render")
    expect(status).to be_success
    expect(stdout).not_to include("without updating the lockfile")
    expect(stdout).to include("write dest")
    stdout, _stderr, status = run_pray("help", "plan")
    expect(status).to be_success
    expect(stdout).to include("dry-run")
  end

  it "documents local prayer init" do
    stdout, _stderr, status = run_pray("help", "prayer")
    expect(status).to be_success
    expect(stdout).to include("prayers/")
    expect(stdout).to include("pray prayer init")
    expect(stdout).to include("prayers/<name>/")
    expect(stdout).not_to include("root packages/")
  end

  it "documents product and catalog layouts on repo init" do
    stdout, _stderr, status = run_pray("help", "repo")
    expect(status).to be_success
    expect(stdout).to include("prayers/v1")
    expect(stdout).to include("prayers/<name>/")
    expect(stdout).not_to include("root packages/")
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
