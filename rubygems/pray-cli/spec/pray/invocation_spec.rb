# frozen_string_literal: true

require "spec_helper"

RSpec.describe Pray::Invocation do
  it "strips global flags before command parsing" do
    remaining = described_class.initialize(
      ["--path", "/tmp/project", "--env", "development", "install", "--locked"]
    )

    expect(remaining).to eq(["install", "--locked"])
    expect(described_class.invocation_context.environment).to eq("development")
    expect(described_class.invocation_context.project_root).to eq("/tmp/project")
  end
end
