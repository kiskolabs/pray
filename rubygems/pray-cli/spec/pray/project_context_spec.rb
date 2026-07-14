# frozen_string_literal: true

require "spec_helper"
require "fileutils"

RSpec.describe Pray::ProjectContext do
  let(:workspace) { Dir.mktmpdir("pray-project-context-") }

  after do
    FileUtils.rm_rf(workspace)
  end

  it "lets CLI options override process environment" do
    project_root = File.join(workspace, "project")
    FileUtils.mkdir_p(project_root)
    File.write(File.join(project_root, "Prayfile"), "prayfile \"1\"\n")

    Dir.chdir(workspace) do
      ENV["PRAY_PATH"] = "ignored"
      ENV["PRAY_ENV"] = "ignored"

      context = described_class.from_options(
        Pray::ProjectInvocationOptions.new(
          project_root: project_root,
          environment: "development"
        )
      )

      expect(context.manifest_path).to eq(File.join(context.project_root, "Prayfile"))
      expect(context.environment).to eq("development")
    end
  ensure
    ENV.delete("PRAY_PATH")
    ENV.delete("PRAY_ENV")
  end

  it "prefers process environment over dotenv values" do
    File.write(File.join(workspace, ".env"), "PRAY_ENV=from-dotenv\n")
    File.write(File.join(workspace, "Prayfile"), "prayfile \"1\"\n")

    Dir.chdir(workspace) do
      ENV["PRAY_ENV"] = "from-process"

      context = described_class.from_options(Pray::ProjectInvocationOptions.new)

      expect(context.environment).to eq("from-process")
    end
  ensure
    ENV.delete("PRAY_ENV")
  end
end
