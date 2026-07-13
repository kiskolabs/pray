# frozen_string_literal: true

require_relative "lib/pray/version"

Gem::Specification.new do |spec|
  spec.name = "pray-cli"
  spec.version = Pray::VERSION
  spec.authors = ["Andrei Makarov"]
  spec.email = ["contact@kiskolabs.com"]

  spec.summary = "Ruby library and CLI for the Prayfile workflow"
  spec.description = "Resolves Prayfile dependencies, locks versions, renders managed agent guidance, and consumes git and registry distribution points."
  spec.homepage = "https://pray.kisko.dev"
  spec.license = "MIT"
  spec.required_ruby_version = ">= 3.1.0"

  spec.files = Dir.chdir(File.expand_path(__dir__)) do
    Dir[
      "lib/**/*",
      "bin/**/*",
      "README.md",
      "LICENSE*",
      "CHANGELOG.md",
      "SECURITY.md",
      "pray-cli.gemspec"
    ].select { |path| File.file?(path) }
  end

  spec.bindir = "bin"
  spec.executables = ["pray"]
  spec.require_paths = ["lib"]

  spec.metadata = {
    "homepage_uri" => spec.homepage,
    "source_code_uri" => "https://github.com/kiskolabs/pray/tree/main/rubygems/pray-cli",
    "changelog_uri" => "https://github.com/kiskolabs/pray/blob/main/rubygems/pray-cli/CHANGELOG.md",
    "bug_tracker_uri" => "https://github.com/kiskolabs/pray/issues",
    "documentation_uri" => "https://pray.kisko.dev",
    "rubygems_mfa_required" => "true"
  }

  spec.add_dependency "toml-rb", "~> 4.0"

  spec.add_development_dependency "rspec", "~> 3.13"
end
