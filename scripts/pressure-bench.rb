#!/usr/bin/env ruby
# frozen_string_literal: true

require "json"
require "open3"
require "time"
require_relative "lib/pressure_bench"

ROOT = File.expand_path("..", __dir__)
CEILINGS_PATH = ENV.fetch(
  "PRAY_PRESSURE_CEILINGS",
  File.join(ROOT, "scripts/pressure-ceilings.json")
)
LOADS = ENV.fetch("PRAY_PRESSURE_LOADS", "1,10,100").split(",").map { |part| Integer(part) }
SAMPLES = Integer(ENV.fetch("PRAY_PRESSURE_SAMPLES", "1"))
HEADROOM = Float(ENV.fetch("PRAY_PRESSURE_HEADROOM", "1.15"))
ITERATIONS = ENV.fetch("PRAY_BENCH_ITERATIONS", "50")
WARMUP = ENV.fetch("PRAY_BENCH_WARMUP", "5")
IMPLS = ENV.fetch("PRAY_PRESSURE_IMPLS", PressureBench::DEFAULT_IMPLS.join(",")).split(",").map(&:strip)

gate = ARGV.include?("--gate")
propose = ARGV.include?("--propose")
write = ARGV.include?("--write")
include_ship = ARGV.include?("--include-ship")
self_test = ARGV.include?("--self-test")

if self_test
  sample = <<~TEXT
    parse p50_us=9.0 p95_us=9.5 ops_s=1
    resolve p50_us=100000.0 p95_us=100584.04 ops_s=1
    render p50_us=10.0 p95_us=20.0 ops_s=1
    lock p50_us=4000.0 p95_us=4714.04 ops_s=1
    fixture packages=1 input_bytes=1 output_bytes=1 output_sha256=sha256:abc
    resources wall_ms=3.5 cpu_ms=2.25 peak_rss_bytes=4096000 iterations=50 warmup=5
  TEXT
  parsed = PressureBench.parse_compat_output(sample)
  abort("self-test parse failed") unless parsed["resolve_p95_us"] == 100_584.04
  abort("self-test digest failed") unless parsed["output_sha256"] == "sha256:abc"
  summary = PressureBench.summarize([parsed, parsed.merge("wall_ms" => 4.5)])
  abort("self-test median failed") unless summary["wall_ms"]["median"] == 4.0
  proposed = PressureBench.propose_ceilings(
    {"1" => summary},
    headroom: 1.15,
    previous: nil
  )
  abort("self-test propose failed") unless proposed.dig("loads", "1", "wall_ms") == 6
  puts "pressure-bench self-test ok"
  exit 0
end

COMPAT = {
  "rust" => ->(env) {
    Open3.capture3(
      env,
      "cargo", "run", "--release", "-q", "-p", "pray-bench", "--example", "compat_bench",
      chdir: ROOT
    )
  },
  "ruby" => ->(env) {
    Open3.capture3(
      env,
      "ruby", "rubygems/pray-cli/benchmarks/compat_bench.rb",
      chdir: ROOT
    )
  },
  "npm" => ->(env) {
    Open3.capture3(
      env,
      "bash", "-lc",
      "npm run build --prefix ../pray-core && node scripts/compat-bench.mjs",
      chdir: File.join(ROOT, "npmjs/pray-cli")
    )
  }
}.freeze

def run_compat!(implementation, package_count)
  runner = COMPAT.fetch(implementation) do
    abort("unknown implementation: #{implementation}")
  end
  env = {
    "PRAY_BENCH_PACKAGES" => package_count.to_s,
    "PRAY_BENCH_ITERATIONS" => ITERATIONS,
    "PRAY_BENCH_WARMUP" => WARMUP
  }
  output, error, status = runner.call(env)
  warn(error) unless error.empty?
  abort("#{implementation} compat_bench failed at packages=#{package_count}") unless status.success?

  PressureBench.parse_compat_output(output)
end

def run_size_check!
  output, error, status = Open3.capture3("ruby", "scripts/check-package-sizes.rb", chdir: ROOT)
  warn(error) unless error.empty?
  abort("size-check failed") unless status.success?

  output.each_line.filter_map do |line|
    match = line.match(/\A(\w+) (?:bytes|count)=(\d+) budget=(\d+) status=(\w+)\z/)
    next unless match

    [match[1], {"value" => Integer(match[2]), "budget" => Integer(match[3]), "status" => match[4]}]
  end.to_h
end

started_at = Time.now.utc.iso8601
impl_summaries = {}
parity_samples = Hash.new { |hash, key| hash[key] = {} }

IMPLS.each do |implementation|
  summaries = {}
  LOADS.each do |load|
    samples = Array.new(SAMPLES) { run_compat!(implementation, load) }
    summaries[load] = PressureBench.summarize(samples)
    parity_samples[load][implementation] = samples.first
    median = summaries[load]
    puts(
      "impl=#{implementation} load=#{load} wall_ms=#{median['wall_ms']['median']} " \
      "cpu_ms=#{median['cpu_ms']['median']} " \
      "peak_rss_bytes=#{median['peak_rss_bytes']['median']} " \
      "resolve_p95_us=#{median['resolve_p95_us']['median']} " \
      "network=not_in_scope n=#{SAMPLES}"
    )
  end
  impl_summaries[implementation] = summaries
end

PressureBench.assert_digest_parity!(parity_samples) if IMPLS.length > 1

ship = include_ship ? run_size_check! : nil
previous = File.file?(CEILINGS_PATH) ? JSON.parse(File.read(CEILINGS_PATH)) : nil
proposed = propose ? PressureBench.propose_multi(impl_summaries, headroom: HEADROOM, previous: previous) : nil

if write
  abort("--write requires --propose") unless proposed
  File.write(CEILINGS_PATH, JSON.pretty_generate(proposed) + "\n")
  puts "wrote #{CEILINGS_PATH}"
end

failures = []
if gate
  abort("--gate requires #{CEILINGS_PATH}") unless previous
  if previous["implementations"]
    failures = PressureBench.gate_multi(impl_summaries, previous)
  elsif IMPLS == ["rust"] && previous["loads"]
    failures = PressureBench.gate_failures(impl_summaries.fetch("rust"), previous)
  else
    abort("--gate requires version-2 ceilings with implementations for #{IMPLS.join(",")}")
  end
  failures.each do |failure|
    warn(
      "over ceiling impl=#{failure['implementation'] || 'rust'} " \
      "load=#{failure['load']} series=#{failure['series']} " \
      "measured=#{failure['measured']} ceiling=#{failure['ceiling']}"
    )
  end
end

report = {
  "started_at" => started_at,
  "finished_at" => Time.now.utc.iso8601,
  "implementations" => IMPLS,
  "loads" => LOADS,
  "samples_per_load" => SAMPLES,
  "headroom" => HEADROOM,
  "network" => "not_in_scope",
  "summaries" => impl_summaries.transform_values { |summaries| summaries.transform_keys(&:to_s) },
  "ship" => ship,
  "proposed_ceilings" => proposed,
  "gate_failures" => failures
}
puts JSON.pretty_generate(report)
exit(failures.empty? ? 0 : 1)
