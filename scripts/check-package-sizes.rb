#!/usr/bin/env ruby
# frozen_string_literal: true

require "json"
require "open3"
require "tmpdir"

ROOT = File.expand_path("..", __dir__)
BUDGETS = {
  rust_binary: 9_500_000,
  ruby_gem: 131_072,
  npm_packed: 150_000,
  npm_unpacked: 650_000,
  npm_entries: 360
}.freeze

def run!(*command, chdir:, env: {})
  output, error, status = Open3.capture3(env, *command, chdir: chdir)
  return output if status.success?

  warn(error)
  abort("package-size command failed: #{command.join(" ")}")
end

def check!(name, value)
  budget = BUDGETS.fetch(name)
  status = value <= budget ? "ok" : "over"
  puts "#{name} bytes=#{value} budget=#{budget} status=#{status}" unless name == :npm_entries
  puts "#{name} count=#{value} budget=#{budget} status=#{status}" if name == :npm_entries
  abort("package-size budget exceeded: #{name}") unless status == "ok"
end

run!("cargo", "build", "--release", "--locked", "-p", "pray-cli", chdir: ROOT)
check!(:rust_binary, File.size(File.join(ROOT, "target/release/pray")))

Dir.mktmpdir("pray-package-sizes-") do |temporary|
  gem_path = File.join(temporary, "pray-cli.gem")
  run!("gem", "build", "pray-cli.gemspec", "--output", gem_path, chdir: File.join(ROOT, "rubygems/pray-cli"))
  check!(:ruby_gem, File.size(gem_path))

  npm_core = File.join(ROOT, "npmjs/pray-core")
  run!("npm", "run", "--silent", "build", chdir: npm_core)
  packed = JSON.parse(
    run!(
      "npm", "pack", "--dry-run", "--json", "--ignore-scripts",
      chdir: npm_core,
      env: {"NPM_CONFIG_CACHE" => File.join(temporary, "npm-cache")}
    )
  ).fetch(0)
  check!(:npm_packed, packed.fetch("size"))
  check!(:npm_unpacked, packed.fetch("unpackedSize"))
  check!(:npm_entries, packed.fetch("entryCount"))
end
