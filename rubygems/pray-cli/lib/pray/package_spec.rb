# frozen_string_literal: true

module Pray
  PackageExport = Struct.new(:kind, :path, :summary, keyword_init: true)
  PackageSkill = Struct.new(:path, :summary, keyword_init: true)
  PackageTemplate = Struct.new(:path, :summary, keyword_init: true)
  PackageDependency = Struct.new(:name, :constraint, :optional, keyword_init: true)

  PackageSpec = Struct.new(
    :name, :version, :summary, :description, :authors, :license, :homepage,
    :source_code_uri, :changelog_uri, :prayfile_version, :files, :exports,
    :skills, :templates, :adapters, :targets, :dependencies, :metadata,
    keyword_init: true
  ) do
    def initialize(
      name: "", version: "", summary: nil, description: nil, authors: [], license: nil,
      homepage: nil, source_code_uri: nil, changelog_uri: nil, prayfile_version: nil,
      files: [], exports: {}, skills: {}, templates: {}, adapters: {}, targets: [],
      dependencies: [], metadata: {}
    )
      super
    end

    def canonicalized
      dup.tap do |copy|
        copy.files = files.sort
        copy.authors = authors.sort
        copy.targets = targets.sort
        copy.dependencies = dependencies.sort_by { |dependency| [dependency.name, dependency.constraint, dependency.optional] }
      end
    end

    def tree_hash_for_root(root)
      file_bytes = {}
      files.each do |file|
        path = File.join(root, file)
        raise Error.integrity("package file missing: #{file}") unless File.exist?(path)
        raise Error.integrity("package file is a directory: #{file}") if File.directory?(path)

        file_bytes[file] = File.binread(path)
      end
      self.class.tree_hash_from_file_bytes(file_bytes)
    end

    def self.tree_hash_from_file_bytes(file_bytes)
      entries = file_bytes.map { |path, bytes| [path, Hashing.sha256_prefixed(bytes)] }.sort_by(&:first)
      serialized = entries.map do |path, hash|
        "file\0regular\0#{path}\0#{hash}\n"
      end.join
      Hashing.sha256_prefixed(serialized)
    end
  end

  module PackageSpecParser
    module_function

    module ParserHelpers
      include ManifestMethods::ParserHelpers

      def parse_dependency(rest, optional)
        values, keywords = parse_call(rest)
        name = string_from_value(values.first)
        constraint = values[1] ? string_from_value(values[1]) : "*"
        PackageDependency.new(
          name: name,
          constraint: constraint,
          optional: keywords["optional"]&.as_bool || optional
        )
      end

      def parse_exports(value)
        map = Literal.parse_literal_map(value)
        exports = {}
        map.each do |name, literal|
          entry = literal.as_map
          raise Error.parse("prayspec", "export #{name} must be a map") unless entry

          exports[name] = PackageExport.new(
            kind: map_string(entry, "type") || "fragment",
            path: map_string(entry, "path") || raise(Error.parse("prayspec", "export #{name} missing path")),
            summary: map_string(entry, "summary")
          )
        end
        exports
      end

      def parse_skills(value)
        map = Literal.parse_literal_map(value)
        map.transform_values do |literal|
          entry = literal.as_map
          raise Error.parse("prayspec", "skill must be a map") unless entry

          PackageSkill.new(
            path: map_string(entry, "path") || raise(Error.parse("prayspec", "skill missing path")),
            summary: map_string(entry, "summary")
          )
        end
      end

      def parse_templates(value)
        map = Literal.parse_literal_map(value)
        map.transform_values do |literal|
          entry = literal.as_map
          raise Error.parse("prayspec", "template must be a map") unless entry

          PackageTemplate.new(
            path: map_string(entry, "path") || raise(Error.parse("prayspec", "template missing path")),
            summary: map_string(entry, "summary")
          )
        end
      end

      def parse_string_map(value)
        Literal.parse_literal_map(value).transform_values { |literal| string_from_value(literal) }
      end

      def map_string(map, key)
        map[key]&.as_string
      end

      def array_of_strings(value)
        array = Literal.parse_literal(value)
        values = array.as_array
        raise Error.parse("prayspec", "expected array") unless values

        values.map { |entry| string_from_value(entry) }
      end

      def string_from_value(value)
        text = value.as_string
        raise Error.parse("prayspec", "expected string-like literal, found #{value.inspect}") unless text

        text
      end

      def string_from_literal(value)
        string_from_value(Literal.parse_literal(value))
      end
    end

    def parse_package_spec(text)
      lines = Literal.prepare_parser_lines(text)
      BlockParser.new(lines).parse_root
    end

    class BlockParser
      include ParserHelpers

      def initialize(lines)
        @lines = lines
        @cursor = 0
      end

      def parse_root
        expect_start!
        spec = PackageSpec.new
        while (statement = next_statement)
          return spec.canonicalized if statement == "end"

          apply_statement(spec, statement)
        end
        raise Error.parse("prayspec", "missing 'end'")
      end

      def expect_start!
        statement = next_statement
        raise Error.parse("prayspec", "empty package spec") unless statement
        unless statement.start_with?("Package::Specification.new")
          raise Error.parse("prayspec", "expected Package::Specification.new")
        end
      end

      def apply_statement(spec, statement)
        case statement
        when /\Aspec\.add_dependency (.+)\z/
          spec.dependencies << parse_dependency(Regexp.last_match(1), false)
        when /\Aspec\.add_optional_dependency (.+)\z/
          spec.dependencies << parse_dependency(Regexp.last_match(1), true)
        when /\Aspec\.(.+) = (.+)\z/
          apply_assignment(spec, Regexp.last_match(1).strip, Regexp.last_match(2).strip)
        else
          raise Error.parse("prayspec", "unrecognized statement: #{statement}")
        end
      end

      def apply_assignment(spec, field, value)
        case field
        when "name" then spec.name = string_from_literal(value)
        when "version" then spec.version = string_from_literal(value)
        when "summary" then spec.summary = string_from_literal(value)
        when "description" then spec.description = string_from_literal(value)
        when "authors" then spec.authors = array_of_strings(value)
        when "license" then spec.license = string_from_literal(value)
        when "homepage" then spec.homepage = string_from_literal(value)
        when "source_code_uri" then spec.source_code_uri = string_from_literal(value)
        when "changelog_uri" then spec.changelog_uri = string_from_literal(value)
        when "prayfile_version" then spec.prayfile_version = string_from_literal(value)
        when "files" then spec.files = array_of_strings(value)
        when "targets" then spec.targets = array_of_strings(value)
        when "exports" then spec.exports = parse_exports(value)
        when "skills" then spec.skills = parse_skills(value)
        when "templates" then spec.templates = parse_templates(value)
        when "adapters" then spec.adapters = parse_string_map(value)
        when "metadata" then spec.metadata = Literal.parse_literal_map(value)
        else
          raise Error.parse("prayspec", "unsupported assignment: #{field}")
        end
      end

      def next_statement
        while @cursor < @lines.length
          statement = @lines[@cursor].strip
          @cursor += 1
          next if statement.empty?

          while !statement.end_with?(" do") && statement != "end" && @cursor < @lines.length &&
                (statement.rstrip.end_with?(",") || !Literal.is_balanced?(statement))
            next_line = @lines[@cursor].strip
            @cursor += 1
            next if next_line.empty?

            statement = "#{statement} #{next_line}"
          end
          return statement
        end
        nil
      end
    end
  end

  extend PackageSpecParser
end
