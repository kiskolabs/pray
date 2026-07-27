# frozen_string_literal: true

require "spec_helper"

RSpec.describe Pray::StatementSurface do
  it "expands semicolon one-liner" do
    expect(
      described_class.expand_statement_surface(
        'pray do; support_email("a@example.com"); security_email("b@example.com"); end'
      )
    ).to eq([
      "pray do",
      'support_email "a@example.com"',
      'security_email "b@example.com"',
      "end"
    ])
  end

  it "expands brace block" do
    expect(
      described_class.expand_statement_surface(
        'pray{support_email("a@example.com");security_email("b@example.com")}'
      )
    ).to eq([
      "pray do",
      'support_email "a@example.com"',
      'security_email "b@example.com"',
      "end"
    ])
  end

  it "unwraps compose call parentheses" do
    expect(described_class.expand_statement_surface('compose("AGENTS.md") do')).to eq([
      'compose "AGENTS.md" do'
    ])
  end

  it "splits symbol call form" do
    expect(described_class.split_symbol_assignment('support_email("contact@kiskolabs.com")')).to eq([
      "support_email",
      '"contact@kiskolabs.com"'
    ])
  end

  it "leaves assignment map literals alone" do
    statement = 'spec.exports = { "AGENTS.md" => "templates/agents.md" }'
    expect(described_class.expand_statement_surface(statement)).to eq([statement])
  end
end
