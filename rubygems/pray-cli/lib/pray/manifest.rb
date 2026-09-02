# frozen_string_literal: true

require_relative "manifest_json"
require_relative "manifest_formatter"
require_relative "manifest_parser_helpers"
require_relative "manifest_parser_blocks"
require_relative "manifest_parser"
require_relative "path_safety"

module Pray
  RenderPolicy = Struct.new(
    :mode, :conflict, :churn, :header
  ) do
    def self.default
      new(
        mode: "managed",
        conflict: "fail",
        churn: "minimal",
        header: true
      )
    end
  end

  ManifestSource = Struct.new(:name, :kind, :url, :subdir, :rev, :tag)
  ManifestTarget = Struct.new(
    :name, :outputs, :skills, :commands, :rules, :max_bytes, :mode, :scoped, :entries, :header
  ) do
    def initialize(
      name:, outputs: [], skills: [], commands: [], rules: [], max_bytes: nil,
      mode: "legacy", scoped: false, entries: [], header: nil
    )
      super
    end
  end

  ManifestPackage = Struct.new(
    :name, :constraint, :source, :exports, :targets, :features, :groups, :optional,
    :path, :git, :tag, :rev, :tarball, :oci, :file, :roles, :bound
  ) do
    def initialize(
      name:, constraint: "*", source: nil, exports: [], targets: [], features: [], groups: [],
      optional: false, path: nil, git: nil, tag: nil, rev: nil, tarball: nil, oci: nil,
      file: nil, roles: [], bound: false
    )
      super
    end
  end

  ManifestLocal = Struct.new(:path, :position, :optional, :bound) do
    def initialize(path:, position: "after", optional: false, bound: false)
      super
    end
  end

  Manifest = Struct.new(
    :prayfile_version, :sources, :targets, :packages, :local, :symbols, :render,
    :deprecated_keywords,
    keyword_init: true
  ) do
    def initialize(
      prayfile_version: "",
      sources: [],
      targets: [],
      packages: [],
      local: [],
      symbols: {},
      render: RenderPolicy.default,
      deprecated_keywords: []
    )
      super
    end

    def note_deprecated_keyword(keyword)
      return unless %w[target output agent skills].include?(keyword)
      self.deprecated_keywords ||= []
      deprecated_keywords << keyword unless deprecated_keywords.include?(keyword)
    end

    def deprecation_warnings
      replacements = {
        "target" => "compose` / `tree",
        "output" => "compose",
        "agent" => "pray",
        "skills" => "tree` / `folder",
        "skill" => "folder",
        "spec.skills" => "a folder export"
      }
      (deprecated_keywords || []).filter_map do |keyword|
        replacement = replacements[keyword]
        next unless replacement

        "warning: `#{keyword}` is deprecated and will be removed in version 2; prefer `#{replacement}`"
      end
    end

    def canonicalized
      dup.tap do |copy|
        copy.sources = sources.sort_by(&:name)
        copy.targets = targets.sort_by(&:name)
        copy.packages = packages.sort_by { |package| [package.name, package.source.to_s, package.constraint] }
        copy.local = local.sort_by(&:path)
        copy.deprecated_keywords = []
      end
    end

    def manifest_hash
      bytes = ManifestJson.encode_compact(canonicalized)
      Hashing.sha256_prefixed(bytes)
    end
  end

  module ManifestMethods
    module_function

    def read_manifest_text(manifest_path)
      File.read(manifest_path)
    rescue Errno::ENOENT
      raise Error.manifest("missing #{manifest_path}; run pray init to create one")
    end

    def parse_manifest(text)
      lines = Literal.prepare_parser_lines(text)
      manifest = BlockParser.new(lines).parse_root
      validate_manifest_paths!(manifest)
      manifest
    end

    def validate_manifest_paths!(manifest)
      manifest.targets.each do |target|
        (target.outputs + target.skills + target.commands + target.rules).each do |path|
          PathSafety.validate_destination_path!(path)
        end
      end
      manifest.packages.each do |package|
        PathSafety.validate_project_relative_path!(package.path) if package.path
        PathSafety.validate_destination_path!(package.file) if package.file
      end
      manifest.local.each { |local| PathSafety.validate_project_relative_path!(local.path) }
    end
  end

  extend ManifestMethods
end
