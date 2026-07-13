# frozen_string_literal: true

require_relative "manifest_json"

module Pray
  RenderPolicy = Struct.new(
    :mode, :conflict, :churn, :header, :section_markers, :line_endings,
    keyword_init: true
  ) do
    def self.default
      new(
        mode: "managed",
        conflict: "fail",
        churn: "minimal",
        header: true,
        section_markers: true,
        line_endings: "lf"
      )
    end
  end

  ManifestSource = Struct.new(:name, :kind, :url, :subdir, :rev, :tag, keyword_init: true)
  ManifestTarget = Struct.new(
    :name, :outputs, :skills, :commands, :rules, :max_bytes,
    keyword_init: true
  ) do
    def initialize(name:, outputs: [], skills: [], commands: [], rules: [], max_bytes: nil)
      super
    end
  end

  ManifestPackage = Struct.new(
    :name, :constraint, :source, :exports, :targets, :features, :optional,
    :path, :git, :tag, :rev, :tarball, :oci,
    keyword_init: true
  ) do
    def initialize(
      name:, constraint: "*", source: nil, exports: [], targets: [], features: [],
      optional: false, path: nil, git: nil, tag: nil, rev: nil, tarball: nil, oci: nil
    )
      super
    end
  end

  ManifestLocal = Struct.new(:path, :position, :optional, keyword_init: true) do
    def initialize(path:, position: "after", optional: false)
      super
    end
  end

  Manifest = Struct.new(
    :prayfile_version, :sources, :targets, :packages, :local, :render,
    keyword_init: true
  ) do
    def initialize(
      prayfile_version: "",
      sources: [],
      targets: [],
      packages: [],
      local: [],
      render: RenderPolicy.default
    )
      super
    end

    def canonicalized
      dup.tap do |copy|
        copy.sources = sources.sort_by(&:name)
        copy.targets = targets.sort_by(&:name)
        copy.packages = packages.sort_by { |package| [package.name, package.source.to_s, package.constraint] }
        copy.local = local.sort_by(&:path)
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
      BlockParser.new(lines).parse_root
    end

    def format_package_declaration(package)
      parts = ["agent \"#{package.name}\""]
      parts << "\"#{package.constraint}\"" unless package.constraint == "*"
      parts << "path: \"#{package.path}\"" if package.path
      parts << "source: \"#{package.source}\"" if package.source
      parts << "git: \"#{package.git}\"" if package.git
      parts << "tag: \"#{package.tag}\"" if package.tag
      parts << "rev: \"#{package.rev}\"" if package.rev
      parts << "tarball: \"#{package.tarball}\"" if package.tarball
      parts << "oci: \"#{package.oci}\"" if package.oci
      parts << "exports: [#{format_string_keyword_list(package.exports)}]" unless package.exports.empty?
      parts << "targets: [#{format_string_keyword_list(package.targets)}]" unless package.targets.empty?
      parts << "features: [#{format_string_keyword_list(package.features)}]" unless package.features.empty?
      parts << "optional: true" if package.optional
      parts.join(", ")
    end

    def replace_package_declaration(text, package)
      name = package.name
      package_prefix = "agent \"#{name}\""
      alternate_prefix = "agent '#{name}'"
      lines = text.lines.map(&:chomp)
      index = lines.index { |line| trimmed = line.lstrip; trimmed.start_with?(package_prefix) || trimmed.start_with?(alternate_prefix) }
      raise Error.manifest("package #{name} not found in manifest") unless index

      lines[index] = format_package_declaration(package)
      output = lines.join("\n")
      output += "\n" if text.end_with?("\n") && !output.end_with?("\n")
      output
    end

    def format_string_keyword_list(values)
      values.map { |value| "\"#{value}\"" }.join(", ")
    end

    class BlockParser
      def initialize(lines)
        @lines = lines
        @cursor = 0
      end

      def parse_root
        manifest = Manifest.new
        while (statement = next_statement)
          raise Error.parse("manifest", "unexpected 'end'") if statement == "end"

          apply_statement(manifest, statement, false)
        end
        raise Error.manifest("missing prayfile version") if manifest.prayfile_version.empty?

        manifest
      end

      def parse_nested(manifest, stop_on_end:)
        while (statement = next_statement)
          return if statement == "end" && stop_on_end
          raise Error.parse("manifest", "unexpected 'end'") if statement == "end"

          apply_statement(manifest, statement, true)
        end
        raise Error.parse("manifest", "missing 'end'") if stop_on_end
      end

      def apply_statement(manifest, statement, allow_target)
        case statement
        when /\Aprayfile (.+)\z/
          manifest.prayfile_version = string_from_literal(Regexp.last_match(1))
        when /\Asource (.+)\z/
          manifest.sources << parse_source(Regexp.last_match(1))
        when /\Atarget (.+)\z/
          raise Error.parse("manifest", "target must use a block") if !allow_target && !statement.end_with?(" do")

          target, is_block = parse_target_header(Regexp.last_match(1))
          manifest.targets << target
          parse_target_block(manifest, manifest.targets.length - 1) if is_block
        when /\Agroup (.+)\z/
          _, is_block = parse_group_header(Regexp.last_match(1))
          parse_nested(manifest, stop_on_end: true) if is_block
        when /\Aagent (.+)\z/
          manifest.packages << parse_package_decl(Regexp.last_match(1))
        when /\Alocal (.+)\z/
          manifest.local << parse_local_decl(Regexp.last_match(1))
        when /\Arender (.+)\z/
          manifest.render = parse_render_policy(Regexp.last_match(1))
        else
          raise Error.parse("manifest", "unrecognized statement: #{statement}")
        end
      end

      def parse_target_block(manifest, target_index)
        while (statement = next_statement)
          return if statement == "end"

          target = manifest.targets[target_index]
          raise Error.manifest("target index out of range") unless target

          apply_target_statement(target, statement)
        end
        raise Error.parse("manifest", "missing 'end' for target block")
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

    module ParserHelpers
      module_function

      def parse_call(rest)
        positional = []
        keywords = {}
        Literal.split_top_level(rest.strip.sub(/,\z/, ""), ",").each do |segment|
          if (keyword = parse_keyword_segment(segment))
            keywords[keyword[0]] = keyword[1]
          elsif !segment.empty?
            positional << Literal.parse_literal(segment)
          end
        end
        [positional, keywords]
      end

      def parse_keyword_segment(segment)
        if (index = Literal.find_top_level(segment, "=>"))
          key = string_from_literal(segment[0...index].strip)
          return [key, Literal.parse_literal(segment[(index + 2)..].strip)]
        end
        if (index = Literal.find_top_level(segment, ":"))
          left = segment[0...index].strip
          right = segment[(index + 1)..].strip
          return nil if left.empty?

          return [left.delete_prefix(":"), Literal.parse_literal(right)]
        end
        nil
      end

      def keyword_array(keywords, key)
        keywords[key]&.as_array&.filter_map(&:as_string) || []
      end

      def string_from_value(value)
        text = value.as_string
        raise Error.parse("manifest", "expected string-like literal, found #{value.inspect}") unless text

        text
      end

      def string_from_literal(input)
        string_from_value(Literal.parse_literal(input))
      end

      def parse_source(rest)
        values, keywords = parse_call(rest)
        raise Error.parse("manifest", "source requires a name") if values.empty?
        if values.length < 2 && !keywords.key?("path") && !keywords.key?("git")
          raise Error.parse("manifest", "source requires a name and url, path:, or git:")
        end

        name = string_from_value(values.first)
        if keywords["path"]
          kind = "path"
          url = string_from_value(keywords["path"])
        elsif keywords["git"]
          kind = "git"
          url = string_from_value(keywords["git"])
          url = "git+#{url}" unless url.start_with?("git+")
        else
          url = string_from_value(values[1])
          kind = if url.start_with?("git+")
                   "git"
                 elsif url.start_with?("pray+ssh://", "ssh+pray://")
                   "pray_ssh"
                 else
                   "registry"
                 end
        end

        ManifestSource.new(
          name: name,
          kind: kind,
          url: url,
          subdir: keywords["subdir"]&.then { |value| string_from_value(value) } ||
            keywords["distribution"]&.then { |value| string_from_value(value) },
          rev: keywords["rev"]&.then { |value| string_from_value(value) },
          tag: keywords["tag"]&.then { |value| string_from_value(value) }
        )
      end

      def parse_target_header(rest)
        is_block = rest.rstrip.end_with?("do")
        header = rest.sub(/\s*do\z/, "").strip
        values, keywords = parse_call(header)
        name = string_from_value(values.first)
        outputs = keyword_array(keywords, "output")
        folders = keyword_array(keywords, "folder") + keyword_array(keywords, "skills")
        target = ManifestTarget.new(
          name: name,
          outputs: outputs,
          skills: folders,
          commands: keyword_array(keywords, "commands"),
          rules: keyword_array(keywords, "rules"),
          max_bytes: keywords["max_bytes"]&.as_integer
        )
        [target, is_block]
      end

      def parse_group_header(rest)
        is_block = rest.rstrip.end_with?("do")
        header = rest.sub(/\s*do\z/, "").strip
        values, = parse_call(header)
        [string_from_value(values.first), is_block]
      end

      def parse_package_decl(rest)
        values, keywords = parse_call(rest)
        raise Error.parse("manifest", "agent missing name") if values.empty?

        name = string_from_value(values[0])
        constraint = if values[1]
                       Constraint.normalize_version_constraint(string_from_value(values[1]))
                     else
                       "*"
                     end
        ManifestPackage.new(
          name: name,
          constraint: constraint,
          source: keywords["source"]&.as_string,
          exports: keyword_array(keywords, "exports"),
          targets: keyword_array(keywords, "targets"),
          features: keyword_array(keywords, "features"),
          optional: keywords["optional"]&.as_bool || false,
          path: keywords["path"]&.as_string,
          git: keywords["git"]&.as_string,
          tag: keywords["tag"]&.as_string,
          rev: keywords["rev"]&.as_string,
          tarball: keywords["tarball"]&.as_string,
          oci: keywords["oci"]&.as_string
        )
      end

      def parse_local_decl(rest)
        values, keywords = parse_call(rest)
        ManifestLocal.new(
          path: string_from_value(values.first),
          position: keywords["position"]&.as_string || "after",
          optional: keywords["optional"]&.as_bool || false
        )
      end

      def parse_render_policy(rest)
        _, keywords = parse_call(rest)
        RenderPolicy.new(
          mode: keywords["mode"]&.as_string || "managed",
          conflict: keywords["conflict"]&.as_string || "fail",
          churn: keywords["churn"]&.as_string || "minimal",
          header: keyword_bool(keywords, "header", true),
          section_markers: keyword_bool(keywords, "section_markers", true),
          line_endings: keywords["line_endings"]&.as_string || "lf"
        )
      end

      def keyword_bool(keywords, key, default)
        value = keywords[key]
        return default if value.nil?

        value.as_bool
      end

      def apply_target_statement(target, statement)
        case statement
        when /\Aoutput (.+)\z/
          target.outputs << string_from_literal(Regexp.last_match(1))
        when /\Afolder (.+)\z/, /\Askills (.+)\z/
          target.skills << string_from_literal(Regexp.last_match(1))
        when /\Acommands (.+)\z/
          target.commands << string_from_literal(Regexp.last_match(1))
        when /\Arules (.+)\z/
          target.rules << string_from_literal(Regexp.last_match(1))
        when /\Amax_bytes (.+)\z/
          target.max_bytes = Literal.parse_literal(Regexp.last_match(1).strip).as_integer
        else
          raise Error.parse("manifest", "unrecognized target statement: #{statement}")
        end
      end
    end

    class BlockParser
      include ParserHelpers
    end
  end

  extend ManifestMethods
end
