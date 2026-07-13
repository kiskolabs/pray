# frozen_string_literal: true

require "spec_helper"

RSpec.describe Pray::Render do
  let(:simple_project) { File.expand_path("../../../../examples/simple-project", __dir__) }

  it "renders managed spans for selected exports" do
    project = Pray::Resolve.resolve_project(File.join(simple_project, "Prayfile"))
    rendered = described_class.render_project(project)
    instructions = rendered.find { |target| target.path == "INSTRUCTIONS.md" }

    expect(instructions).not_to be_nil
    expect(instructions.content).to include("<!-- pray:")
    expect(instructions.content).to include("## Shared instructions")
    expect(instructions.managed_spans.map(&:export)).to include("testing-basics", "security-basics")
  end

  it "records checksums for managed spans" do
    project = Pray::Resolve.resolve_project(File.join(simple_project, "Prayfile"))
    rendered = described_class.render_project(project)
    span = rendered.flat_map(&:managed_spans).find { |entry| entry.export == "testing-basics" }

    expect(span.ideal_checksum).to start_with("sha256:")
    expect(span.package).to eq("sample/base")
  end
end
