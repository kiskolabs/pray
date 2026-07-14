# frozen_string_literal: true

require "spec_helper"

RSpec.describe Pray::Dotenv do
  it "parses common dotenv forms and keeps only PRAY_* variables" do
    variables = described_class.parse_dotenv_text(<<~DOTENV)
      # comment
      export PRAY_ENV=development
      PRAY_PATH="/tmp/project"
      PRAY_FILE_PATH='configs/Prayfile'
      OTHER=value
    DOTENV

    expect(variables).to eq(
      {
        "PRAY_ENV" => "development",
        "PRAY_PATH" => "/tmp/project",
        "PRAY_FILE_PATH" => "configs/Prayfile"
      }
    )
  end
end
