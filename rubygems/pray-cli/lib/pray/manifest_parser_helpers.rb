# frozen_string_literal: true

module Pray
  module ManifestMethods
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

        kind, url = source_kind_and_url(values, keywords)
        ManifestSource.new(
          name: string_from_value(values.first),
          kind: kind,
          url: url,
          subdir: keyword_string(keywords, "subdir") || keyword_string(keywords, "distribution"),
          rev: keyword_string(keywords, "rev"),
          tag: keyword_string(keywords, "tag")
        )
      end

      def source_kind_and_url(values, keywords)
        if keywords["path"]
          return ["path", string_from_value(keywords["path"])]
        end
        if keywords["git"]
          url = string_from_value(keywords["git"])
          url = "git+#{url}" unless url.start_with?("git+")
          return ["git", url]
        end

        url = string_from_value(values[1])
        [infer_source_kind(url), url]
      end

      def infer_source_kind(url)
        return "git" if url.start_with?("git+")
        return "pray_ssh" if url.start_with?("pray+ssh://", "ssh+pray://")

        "registry"
      end

      def keyword_string(keywords, key)
        keywords[key]&.then { |value| string_from_value(value) }
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

        file = keywords["file"]&.as_string
        ManifestPackage.new(
          name: string_from_value(values[0]),
          constraint: package_constraint(values),
          source: keywords["source"]&.as_string,
          exports: package_exports(keywords),
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
          roles: file ? ["file"] : [],
          bound: false
        )
      end

      def package_constraint(values)
        return "*" unless values[1]

        Constraint.normalize_version_constraint(string_from_value(values[1]))
      end

      def package_exports(keywords)
        exports = keyword_array(keywords, "exports")
        if (export = keywords["export"]&.as_string)
          exports << export unless exports.include?(export)
        end
        exports
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
        conflict = keywords["conflict"]&.as_string || "fail"
        unless conflict == "fail"
          raise Error.unsupported(
            "render conflict :#{conflict} is not implemented; only :fail is supported"
          )
        end

        RenderPolicy.new(
          mode: keywords["mode"]&.as_string || "managed",
          conflict: conflict,
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
  end
end
