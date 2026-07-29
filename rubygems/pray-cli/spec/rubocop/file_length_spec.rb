# frozen_string_literal: true

require "spec_helper"
require "rubocop"
require_relative "../../rubocop/cop/pray/file_length"

RSpec.describe RuboCop::Cop::Pray::FileLength do
  def offenses_for(source, max:)
    config = RuboCop::Config.new(
      "Pray/FileLength" => {"Enabled" => true, "Max" => max},
      "AllCops" => {"Include" => ["**/*.rb"]}
    )
    team = RuboCop::Cop::Team.new([described_class.new(config)], config)
    processed = RuboCop::ProcessedSource.new(source, RUBY_VERSION.to_f, "example.rb")
    team.investigate(processed).offenses
  end

  it "reports no offense when the file is within Max" do
    source = (["# line"] * 3).join("\n") + "\n"
    expect(offenses_for(source, max: 3)).to be_empty
  end

  it "reports an offense when the file exceeds Max" do
    source = (["# line"] * 4).join("\n") + "\n"
    offenses = offenses_for(source, max: 3)
    expect(offenses.size).to eq(1)
    expect(offenses.first.message).to eq("File has too many lines (4/3).")
  end
end
