# frozen_string_literal: true

module Pray
  module ManifestMethods
    class BlockParser
      include ParserHelpers
      include ParserBlocks

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
          apply_legacy_target(manifest, statement, Regexp.last_match(1), allow_target)
        when /\Agroup (.+)\z/
          apply_group(manifest, Regexp.last_match(1))
        when "pray do", "template do"
          parse_symbols_block(manifest)
        when /\A(?:compose|output) (.+)\z/
          apply_compose_or_output(manifest, statement, Regexp.last_match(1))
        when /\A(?:tree|folder|skills) (.+)\z/
          apply_tree_or_folder(manifest, statement, Regexp.last_match(1))
        when /\Afile (.+)\z/
          parse_file_block(manifest, Regexp.last_match(1))
        when /\Aagent (.+)\z/
          apply_agent_package(manifest, Regexp.last_match(1))
        when /\Apackage (.+)\z/
          Destination.upsert_package(manifest, parse_package_with_groups(Regexp.last_match(1)))
        when /\A(?:pray|use|include) (.+)\z/
          apply_pray_statement(manifest, Regexp.last_match(1), nil)
        when /\Alocal (.+)\z/
          apply_unbound_local(manifest, Regexp.last_match(1))
        when /\Arender (.+)\z/
          manifest.render = parse_render_policy(Regexp.last_match(1))
        else
          raise Error.parse("manifest", "unrecognized statement: #{statement}")
        end
      end

      def apply_legacy_target(manifest, statement, rest, allow_target)
        raise Error.parse("manifest", "target must use a block") if !allow_target && !statement.end_with?(" do")

        manifest.note_deprecated_keyword("target")
        target, is_block = parse_target_header(rest)
        manifest.targets << target
        parse_target_block(manifest, manifest.targets.length - 1) if is_block
      end

      def apply_group(manifest, rest)
        group_names, is_block = parse_group_header(rest)
        raise Error.parse("manifest", "group must use a block") unless is_block
        raise Error.parse("manifest", "nested group blocks are not supported") if @group_stack.any?

        @group_stack.push(group_names)
        parse_group_block(manifest)
        @group_stack.pop
      end

      def apply_compose_or_output(manifest, statement, rest)
        if statement.start_with?("output ")
          unless statement.end_with?(" do")
            raise Error.parse(
              "manifest",
              "top-level output must use a compose block (output \"path\" do ... end)"
            )
          end
          manifest.note_deprecated_keyword("output")
        end
        parse_destination_block(manifest, rest, "compose")
      end

      def apply_tree_or_folder(manifest, statement, rest)
        if statement.start_with?("folder ", "skills ") && !statement.end_with?(" do")
          raise Error.parse("manifest", "top-level folder/skills must use a tree block")
        end
        parse_destination_block(manifest, rest, "tree")
      end

      def apply_agent_package(manifest, rest)
        manifest.note_deprecated_keyword("agent")
        Destination.upsert_package(manifest, parse_package_with_groups(rest))
      end

      def apply_unbound_local(manifest, rest)
        local = parse_local_decl(rest)
        local.bound = false
        Destination.upsert_local(manifest, local)
      end

      def parse_package_with_groups(rest)
        package = parse_package_decl(rest)
        package.groups = @group_stack.last&.dup || []
        package
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
  end
end
