# frozen_string_literal: true

require_relative "manifest_json"

module Pray
  RenderPolicy = Struct.new(
    :mode, :conflict, :churn, :header, :section_markers, :line_endings
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
    :name, :outputs, :skills, :commands, :rules, :max_bytes, :mode, :scoped, :entries,
    keyword_init: true
  ) do
    def initialize(
      name:, outputs: [], skills: [], commands: [], rules: [], max_bytes: nil,
      mode: "legacy", scoped: false, entries: []
    )
      super
    end
  end

  ManifestPackage = Struct.new(
    :name, :constraint, :source, :exports, :targets, :features, :groups, :optional,
    :path, :git, :tag, :rev, :tarball, :oci, :file, :roles, :bound,
    keyword_init: true
  ) do
    def initialize(
      name:, constraint: "*", source: nil, exports: [], targets: [], features: [], groups: [],
      optional: false, path: nil, git: nil, tag: nil, rev: nil, tarball: nil, oci: nil,
      file: nil, roles: [], bound: false
    )
      super
    end
  end

  ManifestLocal = Struct.new(:path, :position, :optional, :bound, keyword_init: true) do
    def initialize(path:, position: "after", optional: false, bound: false)
      super
    end
  end

  Manifest = Struct.new(
    :prayfile_version, :sources, :targets, :packages, :local, :symbols, :render,
    keyword_init: true
  ) do
    def initialize(
      prayfile_version: "",
      sources: [],
      targets: [],
      packages: [],
      local: [],
      symbols: {},
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
      parts = ["pray \"#{package.name}\""]
      parts << "\"#{package.constraint}\"" unless package.constraint == "*"
      parts << "path: \"#{package.path}\"" if package.path
      parts << "source: \"#{package.source}\"" if package.source
      parts << "git: \"#{package.git}\"" if package.git
      parts << "tag: \"#{package.tag}\"" if package.tag
      parts << "rev: \"#{package.rev}\"" if package.rev
      parts << "tarball: \"#{package.tarball}\"" if package.tarball
      parts << "oci: \"#{package.oci}\"" if package.oci
      parts << "file: \"#{package.file}\"" if package.file
      unless package.exports.empty?
        parts << if package.exports.length == 1
          "export: \"#{package.exports.first}\""
        else
          "exports: [#{format_string_keyword_list(package.exports)}]"
        end
      end
      parts << "targets: [#{format_string_keyword_list(package.targets)}]" unless package.targets.empty?
      parts << "features: [#{format_string_keyword_list(package.features)}]" unless package.features.empty?
      parts << "optional: true" if package.optional
      parts.join(", ")
    end

    def replace_package_declaration(text, package)
      name = package.name
      prefixes = [
        "pray \"#{name}\"", "pray '#{name}'",
        "use \"#{name}\"", "include \"#{name}\"",
        "agent \"#{name}\"", "agent '#{name}'",
        "package \"#{name}\"", "package '#{name}'"
      ]
      lines = text.lines.map(&:chomp)
      index = lines.index { |line|
        trimmed = line.lstrip
        prefixes.any? { |prefix| trimmed.start_with?(prefix) }
      }
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
        @group_stack = []
        @surface = StatementSurface::Reader.new
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
          group_names, is_block = parse_group_header(Regexp.last_match(1))
          raise Error.parse("manifest", "group must use a block") unless is_block
          if @group_stack.any?
            raise Error.parse("manifest", "nested group blocks are not supported")
          end

          @group_stack.push(group_names)
          parse_group_block(manifest)
          @group_stack.pop
        when "pray do", "template do"
          parse_symbols_block(manifest)
        when /\A(?:compose|output) (.+)\z/
          if statement.start_with?("output ") && !statement.end_with?(" do")
            raise Error.parse(
              "manifest",
              "top-level output must use a compose block (output \"path\" do ... end)"
            )
          end
          parse_destination_block(manifest, Regexp.last_match(1), "compose")
        when /\A(?:tree|folder|skills) (.+)\z/
          if (statement.start_with?("folder ") || statement.start_with?("skills ")) &&
              !statement.end_with?(" do")
            raise Error.parse("manifest", "top-level folder/skills must use a tree block")
          end
          parse_destination_block(manifest, Regexp.last_match(1), "tree")
        when /\Afile (.+)\z/
          parse_file_block(manifest, Regexp.last_match(1))
        when /\A(?:agent|package) (.+)\z/
          Destination.upsert_package(manifest, parse_package_with_groups(Regexp.last_match(1)))
        when /\A(?:pray|use|include) (.+)\z/
          apply_pray_statement(manifest, Regexp.last_match(1), nil)
        when /\Alocal (.+)\z/
          local = parse_local_decl(Regexp.last_match(1))
          local.bound = false
          Destination.upsert_local(manifest, local)
        when /\Arender (.+)\z/
          manifest.render = parse_render_policy(Regexp.last_match(1))
        else
          raise Error.parse("manifest", "unrecognized statement: #{statement}")
        end
      end

      def parse_symbols_block(manifest)
        while (statement = next_statement)
          return if statement == "end"

          assignment = StatementSurface.split_symbol_assignment(statement)
          unless assignment
            raise Error.parse(
              "manifest",
              "unsupported statement inside pray/template block: #{statement}"
            )
          end

          key, value_literal = assignment
          unless Substitute.pray_symbol_key?(key)
            raise Error.parse("manifest", "invalid pray symbol key `#{key}`")
          end
          if manifest.symbols.key?(key)
            raise Error.parse("manifest", "duplicate pray symbol `#{key}`")
          end

          manifest.symbols[key] = string_from_literal(value_literal)
        end
        raise Error.parse("manifest", "missing 'end' for pray/template block")
      end

      def parse_group_block(manifest)
        while (statement = next_statement)
          return if statement == "end"
          if (match = statement.match(/\A(?:agent|package|pray|use|include) (.+)\z/))
            Destination.upsert_package(manifest, parse_package_with_groups(match[1]))
          elsif /\Agroup (.+)\z/.match?(statement)
            raise Error.parse("manifest", "nested group blocks are not supported")
          else
            raise Error.parse(
              "manifest",
              "group blocks only support agent, package, or pray declarations: #{statement}"
            )
          end
        end
        raise Error.parse("manifest", "missing 'end' for group block")
      end

      def parse_destination_block(manifest, rest, mode)
        unless rest.rstrip.end_with?("do")
          label = mode == "compose" ? "compose" : "tree"
          raise Error.parse("manifest", "#{label} must use a block")
        end

        header = rest.sub(/\s*do\z/, "").strip
        values, = parse_call(header)
        raise Error.parse("manifest", "destination missing path") if values.empty?

        path = string_from_value(values.first)
        manifest.targets << Destination.new_destination_target(mode, path)
        index = manifest.targets.length - 1

        while (statement = next_statement)
          return if statement == "end"

          if (match = statement.match(/\A(?:pray|use|include|agent|package) (.+)\z/))
            apply_pray_statement(manifest, match[1], index)
            next
          end
          if mode == "compose" && (match = statement.match(/\Alocal (.+)\z/))
            local = parse_local_decl(match[1])
            local.bound = true
            Destination.bind_local_entry(manifest.targets[index], local.path)
            Destination.upsert_local(manifest, local)
            next
          end
          raise Error.parse("manifest", "unsupported statement inside destination block: #{statement}")
        end
        raise Error.parse("manifest", "missing 'end' for destination block")
      end

      def parse_file_block(manifest, rest)
        unless rest.rstrip.end_with?("do")
          raise Error.parse("manifest", "file must use a block (or use pray ..., file: \"path\")")
        end

        header = rest.sub(/\s*do\z/, "").strip
        values, = parse_call(header)
        raise Error.parse("manifest", "file block missing path") if values.empty?

        file_path = string_from_value(values.first)
        saw_package = false
        while (statement = next_statement)
          if statement == "end"
            unless saw_package
              raise Error.parse("manifest", "file block requires a pray package declaration")
            end
            return
          end
          if (match = statement.match(/\A(?:pray|use|include|agent|package) (.+)\z/))
            package = parse_package_with_groups(match[1])
            if package.file
              raise Error.parse("manifest", "file: keyword is invalid inside a file block")
            end
            package.file = file_path
            package.bound = true
            package.roles << "file" unless package.roles.include?("file")
            Destination.upsert_package(manifest, package)
            saw_package = true
            next
          end
          raise Error.parse("manifest", "unsupported statement inside file block: #{statement}")
        end
        raise Error.parse("manifest", "missing 'end' for file block")
      end

      def apply_pray_statement(manifest, rest, destination_index)
        values, keywords = parse_call(rest)
        raise Error.parse("manifest", "pray missing package or path") if values.empty?

        first = string_from_value(values.first)
        has_package_signal = values.length > 1 ||
          %w[source export exports file optional path git tag rev tarball oci targets features]
            .any? { |key| keywords.key?(key) }

        in_compose = destination_index &&
          manifest.targets[destination_index]&.mode == "compose"

        if !has_package_signal && Destination.local_path_form?(first)
          unless in_compose
            raise Error.parse("manifest", "local pray paths are only valid inside compose blocks")
          end
          local = ManifestLocal.new(path: first, position: "after", optional: false, bound: true)
          Destination.bind_local_entry(manifest.targets[destination_index], local.path)
          Destination.upsert_local(manifest, local)
          return
        end

        package = parse_package_with_groups(rest)
        if package.file
          if destination_index
            raise Error.parse("manifest", "file: is mutually exclusive with compose/tree nesting")
          end
          package.bound = true
          package.roles << "file" unless package.roles.include?("file")
        end
        if destination_index
          mode = manifest.targets[destination_index].mode
          package.bound = true
          role = Destination.role_for_destination(mode)
          package.roles << role if role && !package.roles.include?(role)
          Destination.bind_package_entry(manifest.targets[destination_index], package.name)
        end
        Destination.upsert_package(manifest, package)
      end

      def parse_package_with_groups(rest)
        package = parse_package_decl(rest)
        package.groups = @group_stack.last&.dup || []
        package
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
        pending = @surface.next
        return pending if pending

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
          @surface.push_raw(statement)
          pending = @surface.next
          return pending if pending
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
        raise Error.parse("manifest", "group missing name") if values.empty?

        names = values.map { |value| string_from_value(value) }
        [names, is_block]
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
        exports = keyword_array(keywords, "exports")
        if (export = keywords["export"]&.as_string)
          exports << export unless exports.include?(export)
        end
        file = keywords["file"]&.as_string
        roles = []
        roles << "file" if file
        ManifestPackage.new(
          name: name,
          constraint: constraint,
          source: keywords["source"]&.as_string,
          exports: exports,
          targets: keyword_array(keywords, "targets"),
          features: keyword_array(keywords, "features"),
          optional: keywords["optional"]&.as_bool || false,
          path: keywords["path"]&.as_string,
          git: keywords["git"]&.as_string,
          tag: keywords["tag"]&.as_string,
          rev: keywords["rev"]&.as_string,
          tarball: keywords["tarball"]&.as_string,
          oci: keywords["oci"]&.as_string,
          file: file,
          roles: roles,
          bound: false
        )
      end

      def parse_local_decl(rest)
        values, keywords = parse_call(rest)
        ManifestLocal.new(
          path: string_from_value(values.first),
          position: keywords["position"]&.as_string || keywords["at"]&.as_string || "after",
          optional: keywords["optional"]&.as_bool || false,
          bound: false
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
