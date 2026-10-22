# frozen_string_literal: true

require "digest"
require "tmpdir"
require_relative "hashing"

module Pray
  module DerivedMetadata
    module_function

    EMBEDDING_MODEL = "pray-hash-bucket-v1"
    EMBEDDING_DIMENSIONS = 16
    MAX_SUMMARY_PARTS = 2
    MAX_TOKEN_COUNT = 512
    MAX_SNIPPET_LENGTH = 120
    STOPWORDS = %w[
      a an and are as at be by for from has have in is it its of on or package the this to was we
      with you your
    ].freeze

    def from_package_root(root, spec)
      summary_candidates = []
      push_candidate(summary_candidates, spec.summary)
      push_candidate(summary_candidates, spec.description)
      spec.exports.each_value do |export|
        push_candidate(summary_candidates, export.summary)
      end

      analyzed_text = +""
      sample_lines = []
      file_count = 0
      character_count = 0
      Array(spec.files).each do |relative|
        path = File.join(root, relative)
        next unless File.file?(path)

        normalized = Hashing.normalize_line_endings(File.read(path, encoding: "UTF-8"))
        line = first_meaningful_line(normalized)
        push_candidate(sample_lines, line) if line
        file_count += 1
        character_count += normalized.chars.count
        analyzed_text << normalized
        analyzed_text << "\n"
      end

      sample_lines.each { |line| push_candidate(summary_candidates, line) }
      if summary_candidates.empty?
        top_topics(analyzed_text).each { |token| push_candidate(summary_candidates, token) }
      end

      summary = build_summary(summary_candidates)
      tokens = tokenize(analyzed_text)
      topics = top_topics(analyzed_text)
      categories = infer_categories("#{summary_candidates.join(" \n")} #{topics.join(' ')}")
      {
        "summary" => summary,
        "topics" => topics,
        "categories" => categories,
        "possible_effects" => infer_possible_effects(categories),
        "possible_side_effects" => infer_possible_side_effects(analyzed_text, topics),
        "embeddings" => [{"model" => EMBEDDING_MODEL, "vector" => hashed_embedding(tokens)}],
        "file_count" => file_count,
        "character_count" => character_count,
        "token_count" => tokens.length
      }
    end

    def from_archive_bytes(archive_bytes)
      Dir.mktmpdir("pray-derived-metadata-") do |directory|
        Archive.unpack_praypkg(archive_bytes, directory)
        spec_path = Resolve.find_prayspec_file(directory)
        spec = Pray.parse_package_spec(File.read(spec_path, encoding: "UTF-8")).canonicalized
        from_package_root(directory, spec)
      end
    end

    def build_summary(candidates)
      summary = []
      seen = {}
      candidates.each do |candidate|
        normalized = candidate.to_s.strip
        next if normalized.empty?

        key = normalized.downcase
        next if seen[key]

        seen[key] = true
        summary << truncate(normalized, MAX_SNIPPET_LENGTH)
        break if summary.length == MAX_SUMMARY_PARTS
      end
      return "Package metadata" if summary.empty?

      summary.join(" — ")
    end
    private_class_method :build_summary

    def top_topics(text)
      counts = Hash.new(0)
      tokenize(text).each { |token| counts[token] += 1 }
      counts.sort_by { |token, count| [-count, token] }.first(5).map(&:first)
    end
    private_class_method :top_topics

    def infer_categories(text)
      lowered = text.downcase
      categories = []
      categories << "documentation" if contains_any?(lowered, %w[guide guidance readme documentation doc])
      categories << "testing" if contains_any?(lowered, %w[test testing fixture spec])
      categories << "security" if contains_any?(lowered, %w[auth security ssh key sign trust])
      categories << "automation" if contains_any?(lowered, %w[automation workflow render pipeline agent])
      categories << "general" if categories.empty?
      categories.sort.uniq
    end
    private_class_method :infer_categories

    def infer_possible_effects(categories)
      effects = []
      effects << "clarifies usage" if categories.include?("documentation")
      effects << "improves validation" if categories.include?("testing")
      effects << "surfaces trust-sensitive behavior" if categories.include?("security")
      effects << "reduces manual steps" if categories.include?("automation")
      effects << "improves package discovery" if effects.empty?
      effects
    end
    private_class_method :infer_possible_effects

    def infer_possible_side_effects(text, topics)
      side_effects = []
      if contains_any?(text, %w[write overwrite replace render inject delete])
        side_effects << "may change generated output or managed files"
      end
      if contains_any?(topics.join(" "), %w[publish release deploy])
        side_effects << "may affect downstream package distribution"
      end
      side_effects.sort.uniq
    end
    private_class_method :infer_possible_side_effects

    def hashed_embedding(tokens)
      vector = Array.new(EMBEDDING_DIMENSIONS, 0)
      tokens.first(MAX_TOKEN_COUNT).each do |token|
        digest = Digest::SHA256.digest(token)
        bucket = digest.getbyte(0) % EMBEDDING_DIMENSIONS
        vector[bucket] += 1
      end
      vector
    end
    private_class_method :hashed_embedding

    def tokenize(text)
      tokens = []
      text.to_s.split(/[^[:alnum:]]+/).each do |raw|
        token = raw.strip.downcase
        next if token.length < 4 || STOPWORDS.include?(token)

        tokens << token
        break if tokens.length == MAX_TOKEN_COUNT
      end
      tokens
    end
    private_class_method :tokenize

    def contains_any?(text, needles)
      needles.any? { |needle| text.include?(needle) }
    end
    private_class_method :contains_any?

    def push_candidate(candidates, candidate)
      trimmed = candidate.to_s.strip
      return if trimmed.empty?

      candidates << truncate(trimmed, MAX_SNIPPET_LENGTH)
    end
    private_class_method :push_candidate

    def first_meaningful_line(text)
      text.each_line.map(&:strip).find { |line| !line.empty? }&.then { |line| truncate(line, MAX_SNIPPET_LENGTH) }
    end
    private_class_method :first_meaningful_line

    def truncate(text, limit)
      text.chars.take(limit).join
    end
    private_class_method :truncate
  end
end
