# frozen_string_literal: true

require "spec_helper"
require "rubocop"
require_relative "../../rubocop/cop/pray/circular_require"

RSpec.describe RuboCop::Cop::Pray::CircularRequire do
  around do |example|
    previous_directory = Dir.pwd
    Dir.mktmpdir do |directory|
      @project_root = directory
      Dir.chdir(directory) do
        example.run
      end
    end
  ensure
    Dir.chdir(previous_directory)
    described_class.reset!
  end

  def write_lib_file(relative_path, content)
    path = File.join(@project_root, "lib", relative_path)
    FileUtils.mkdir_p(File.dirname(path))
    File.write(path, content)
  end

  it "reports no offenses when lib requires are acyclic" do
    write_lib_file("a.rb", <<~RUBY)
      # frozen_string_literal: true

      require_relative "b"
    RUBY
    write_lib_file("b.rb", <<~RUBY)
      # frozen_string_literal: true
    RUBY

    offenses = collect_offenses
    expect(offenses).to be_empty
  end

  it "reports a circular require_relative chain" do
    write_lib_file("a.rb", <<~RUBY)
      # frozen_string_literal: true

      require_relative "b"
    RUBY
    write_lib_file("b.rb", <<~RUBY)
      # frozen_string_literal: true

      require_relative "a"
    RUBY

    offenses = collect_offenses
    expect(offenses.size).to eq(1)
    expect(offenses.first.message).to include("lib/a.rb -> lib/b.rb -> lib/a.rb")
  end

  def collect_offenses
    described_class.reset!
    config = RuboCop::Config.new(
      "Pray/CircularRequire" => {"Enabled" => true},
      "AllCops" => {"Include" => ["lib/**/*.rb"]}
    )
    team = RuboCop::Cop::Team.new([described_class.new(config)], config)
    offenses = []

    Dir.glob(File.join(@project_root, "lib", "**", "*.rb")).sort.each do |path|
      source = RuboCop::ProcessedSource.from_file(path, RUBY_VERSION.to_f)
      report = team.investigate(source)
      offenses.concat(report.offenses)
    end

    offenses
  end
end
