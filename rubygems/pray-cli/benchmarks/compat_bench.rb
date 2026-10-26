# frozen_string_literal: true

require "tmpdir"
require "fileutils"
require "fiddle/import"
require "rbconfig"
require_relative "../lib/pray"

ITERATIONS = Integer(ENV.fetch("PRAY_BENCH_ITERATIONS", "500"))
WARMUP = Integer(ENV.fetch("PRAY_BENCH_WARMUP", "20"))
PACKAGE_COUNT = Integer(ENV.fetch("PRAY_BENCH_PACKAGES", "1"))
CLOCK = Process.const_defined?(:CLOCK_MONOTONIC_RAW) ? Process::CLOCK_MONOTONIC_RAW : Process::CLOCK_MONOTONIC

raise ArgumentError, "iterations must be positive" unless ITERATIONS.positive?
raise ArgumentError, "warmup must not be negative" if WARMUP.negative?
raise ArgumentError, "packages must be positive" unless PACKAGE_COUNT.positive?

def package_identity(index)
  index.zero? ? ["sample/base", "packages/base"] : [format("sample/pkg-%04d", index), format("packages/pkg-%04d", index)]
end

def build_manifest
  lines = ["prayfile \"1\"", "compose \"A.md\" do"]
  PACKAGE_COUNT.times do |index|
    name, path = package_identity(index)
    lines << "  pray \"#{name}\", path: \"#{path}\""
  end
  "#{lines.join("\n")}\nend\n"
end

def build_prayspec(name)
  <<~SPEC
    Package::Specification.new do |spec|
      spec.name = "#{name}"
      spec.version = "1.0.0"
      spec.files = ["exports/a.md"]
      spec.exports = { "a" => { type: "fragment", path: "exports/a.md" } }
    end
  SPEC
end

MANIFEST = build_manifest
PRAYSPECS = PACKAGE_COUNT.times.map { |index| build_prayspec(package_identity(index).first) }

module ResourceUsage
  begin
    extend Fiddle::Importer

    dlload Fiddle.dlopen(nil)
    extern "int getrusage(int, void*)"
    AVAILABLE = true
  rescue Fiddle::DLError
    AVAILABLE = false
  end

  def self.peak_rss_bytes
    return "unsupported" unless AVAILABLE

    buffer = Fiddle::Pointer.malloc(Fiddle::SIZEOF_LONG * 18)
    return 0 unless getrusage(0, buffer).zero?

    maximum_rss = buffer[0, Fiddle::SIZEOF_LONG * 18].unpack("l!*")[4]
    RbConfig::CONFIG.fetch("host_os").include?("darwin") ? maximum_rss : maximum_rss * 1_024
  end
end

def measure
  WARMUP.times { yield }
  samples = Array.new(ITERATIONS) do
    started = Process.clock_gettime(CLOCK, :nanosecond)
    yield
    (Process.clock_gettime(CLOCK, :nanosecond) - started) / 1_000.0
  end.sort
  middle = ITERATIONS / 2
  {
    p50: ITERATIONS.odd? ? samples[middle] : (samples[middle - 1] + samples[middle]) / 2.0,
    p95: samples[(ITERATIONS * 0.95).ceil - 1]
  }
end

def print_measurement(name, measurement)
  puts format(
    "%s p50_us=%.2f p95_us=%.2f ops_s=%.0f",
    name, measurement.fetch(:p50), measurement.fetch(:p95), 1_000_000 / measurement.fetch(:p50)
  )
end

def serialize_lock(project, rendered)
  lockfile = Pray.build_lockfile(
    project.manifest_hash, project.environment, project.project_root,
    project.manifest.sources, project.manifest.targets, rendered,
    project.packages, project.source_revisions, project.source_host_keys
  )
  Pray.serialize_lockfile(lockfile)
end

Dir.mktmpdir("pray-compat-bench") do |root|
  manifest_path = File.join(root, "Prayfile")
  File.write(manifest_path, MANIFEST)
  PACKAGE_COUNT.times do |index|
    _, path = package_identity(index)
    FileUtils.mkdir_p(File.join(root, path, "exports"))
    File.write(File.join(root, path, "package.prayspec"), PRAYSPECS.fetch(index))
    File.write(File.join(root, path, "exports/a.md"), "A\n")
  end
  project = Pray.resolve_project(manifest_path)
  rendered = Pray.render_project(project)
  benchmark_started = Process.clock_gettime(CLOCK, :nanosecond)
  cpu_started = Process.times
  print_measurement("parse", measure { Pray.parse_manifest(MANIFEST) })
  print_measurement("resolve", measure { Pray.resolve_project(manifest_path) })
  print_measurement("render", measure { Pray.render_project(project) })
  print_measurement("lock", measure { serialize_lock(project, rendered) })
  rendered_bytes = rendered.map(&:content).join
  input_bytes = MANIFEST.bytesize + PRAYSPECS.sum(&:bytesize) + (2 * PACKAGE_COUNT)
  puts "fixture packages=#{PACKAGE_COUNT} input_bytes=#{input_bytes} output_bytes=#{rendered_bytes.bytesize} " \
    "output_sha256=#{Pray::Hashing.sha256_prefixed(rendered_bytes)}"
  cpu_finished = Process.times
  cpu_milliseconds = (cpu_finished.utime + cpu_finished.stime - cpu_started.utime - cpu_started.stime) * 1_000
  wall_milliseconds = (Process.clock_gettime(CLOCK, :nanosecond) - benchmark_started) / 1_000_000.0
  puts format(
    "resources wall_ms=%.2f cpu_ms=%.2f peak_rss_bytes=%s iterations=%d warmup=%d",
    wall_milliseconds, cpu_milliseconds, ResourceUsage.peak_rss_bytes, ITERATIONS, WARMUP
  )
end
