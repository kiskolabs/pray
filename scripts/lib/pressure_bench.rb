# frozen_string_literal: true

require "json"

module PressureBench
  SAMPLE_KEYS = %w[wall_ms cpu_ms peak_rss_bytes resolve_p95_us].freeze
  DEFAULT_IMPLS = %w[rust ruby npm].freeze

  module_function

  def parse_fields(line)
    fields = {}
    line.to_s.scan(/(\w+)=([0-9.]+|sha256:[0-9a-f]+)/) do |key, value|
      fields[key] = if value.start_with?("sha256:")
        value
      elsif value.include?(".")
        Float(value)
      else
        Integer(value)
      end
    end
    fields
  end

  def parse_compat_output(text)
    # Ruby /m makes "." match newlines; keep each series on one physical line.
    resources = parse_fields(text[/^resources ([^\n]+)/m, 1])
    resolve = parse_fields(text[/^resolve ([^\n]+)/m, 1])
    fixture = parse_fields(text[/^fixture ([^\n]+)/m, 1])
    raise "missing resources line" if resources.empty?
    raise "missing resolve line" if resolve.empty?

    {
      "wall_ms" => resources.fetch("wall_ms"),
      "cpu_ms" => resources.fetch("cpu_ms"),
      "peak_rss_bytes" => resources.fetch("peak_rss_bytes"),
      "resolve_p95_us" => resolve.fetch("p95_us"),
      "output_sha256" => fixture["output_sha256"]
    }
  end

  def median(values)
    ordered = values.sort
    middle = ordered.length / 2
    return ordered[middle] if ordered.length.odd?

    (ordered[middle - 1] + ordered[middle]) / 2.0
  end

  def summarize(samples)
    SAMPLE_KEYS.to_h do |key|
      values = samples.map { |sample| sample.fetch(key) }
      [key, {"n" => values.length, "median" => median(values), "raw" => values}]
    end
  end

  def propose_ceilings(summaries, headroom:, previous: nil)
    loads = summaries.to_h do |load, summary|
      ceilings = SAMPLE_KEYS.to_h do |key|
        basis = if key == "peak_rss_bytes"
          summary.fetch(key).fetch("median")
        else
          summary.fetch(key).fetch("raw").max
        end
        proposed = (basis * headroom).ceil
        if previous && previous.dig("loads", load.to_s, key)
          prior = previous["loads"][load.to_s][key]
          proposed = [proposed, prior].min
        end
        [key, proposed]
      end
      [load.to_s, ceilings]
    end
    {"version" => 1, "headroom" => headroom, "loads" => loads}
  end

  def propose_multi(impl_summaries, headroom:, previous: nil)
    implementations = impl_summaries.to_h do |impl, summaries|
      prior = previous&.dig("implementations", impl) ||
        (impl == "rust" && previous&.key?("loads") ? previous : nil)
      [impl, propose_ceilings(summaries, headroom: headroom, previous: prior)]
    end
    {"version" => 2, "headroom" => headroom, "implementations" => implementations}
  end

  def gate_failures(summaries, ceilings)
    failures = []
    ceilings.fetch("loads").each do |load, limits|
      summary = summaries[load] || summaries[load.to_i] || summaries[load.to_s]
      raise "no samples for load #{load}" unless summary

      limits.each do |key, limit|
        measured = summary.fetch(key).fetch("median")
        next if measured <= limit

        failures << {
          "load" => load,
          "series" => key,
          "measured" => measured,
          "ceiling" => limit
        }
      end
    end
    failures
  end

  def gate_multi(impl_summaries, ceilings)
    failures = []
    impl_ceilings = ceilings["implementations"]
    raise "ceilings missing implementations" unless impl_ceilings

    impl_summaries.each do |impl, summaries|
      limits = impl_ceilings.fetch(impl)
      gate_failures(summaries, limits).each do |failure|
        failures << failure.merge("implementation" => impl)
      end
    end
    failures
  end

  def assert_digest_parity!(impl_samples_by_load)
    impl_samples_by_load.each do |load, by_impl|
      digests = by_impl.filter_map do |impl, sample|
        digest = sample["output_sha256"]
        next unless digest

        [impl, digest]
      end
      next if digests.length < 2

      reference = digests.first.fetch(1)
      digests.each do |impl, digest|
        next if digest == reference

        raise "digest mismatch at load=#{load}: #{digests.map { |name, value| "#{name}=#{value}" }.join(" ")}"
      end
    end
  end
end
