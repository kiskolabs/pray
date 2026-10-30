# frozen_string_literal: true

require "fileutils"
require "json"
require "tmpdir"
require_relative "../lib/pray"

WARMUP = Integer(ENV.fetch("PRAY_BENCH_WARMUP", "2"))
ITERATIONS = Integer(ENV.fetch("PRAY_BENCH_ITERATIONS", "7"))
CLOCK = Process.const_defined?(:CLOCK_MONOTONIC_RAW) ? Process::CLOCK_MONOTONIC_RAW : Process::CLOCK_MONOTONIC

raise ArgumentError, "warmup must not be negative" if WARMUP.negative?
raise ArgumentError, "iterations must be positive" unless ITERATIONS.positive?

def measure
  WARMUP.times { yield }
  samples = Array.new(ITERATIONS) do
    started = Process.clock_gettime(CLOCK, :nanosecond)
    yield
    (Process.clock_gettime(CLOCK, :nanosecond) - started) / 1_000_000.0
  end.sort
  {p50: samples[samples.length / 2], p95: samples[(samples.length * 0.95).ceil - 1]}
end

def report(name, measurement, details)
  puts format("%s p50_ms=%.3f p95_ms=%.3f %s", name, measurement.fetch(:p50), measurement.fetch(:p95), details)
end

def write_registry(root, count)
  names = Array.new(count) { |index| format("sample/package-%04d", index) }
  FileUtils.mkdir_p(File.join(root, "v1", "packages", "sample"))
  File.write(File.join(root, "v1", "index.json"), JSON.generate("spec" => "prayfile-distribution-1", "packages" => names))
  names.each do |name|
    metadata = {
      "name" => name,
      "versions" => [{
        "version" => "1.0.0", "artifact" => "v1/artifacts/#{name}/1.0.0/package.praypkg",
        "yanked" => false, "derived_metadata" => {"summary" => "Shared guidance"}
      }]
    }
    File.write(File.join(root, "v1", "packages", "#{name}.json"), JSON.generate(metadata))
  end
end

def benchmark_search
  [10, 100, 1_000].each do |count|
    Dir.mktmpdir("pray-search-bench-") do |root|
      write_registry(root, count)
      names = measure { Pray::RegistrySearch.search_local_registry(root, "package", include_summary: false) }
      summaries = measure { Pray::RegistrySearch.search_local_registry(root, "package", include_summary: true) }
      report("search_names", names, "packages=#{count} reads=1")
      report("search_summaries", summaries, "packages=#{count} reads=#{count + 1}")
    end
  end
end

def benchmark_reconciliation
  [65_536, 1_048_576].each do |size|
    prefix = "<!-- pray:bench001 -->\n"
    suffix = "\n<!-- pray:bench001 -->\n"
    body_size = size - prefix.bytesize - suffix.bytesize
    line = ("x" * 79) + "\n"
    body = (line * (body_size / line.bytesize)) + ("x" * (body_size % line.bytesize))
    fresh = prefix + body + suffix
    existing = "#{fresh}\n"
    comparison = measure { Pray::RenderPatch.patch_rendered_content(existing, fresh) == existing }
    report("render_reconcile", comparison, "bytes=#{existing.bytesize} scans=1")
  end
end

benchmark_search
benchmark_reconciliation
