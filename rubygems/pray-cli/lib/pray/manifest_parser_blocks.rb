# frozen_string_literal: true

module Pray
  module ManifestMethods
    module ParserBlocks
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
          if (match = statement.match(/\Aagent (.+)\z/))
            manifest.note_deprecated_keyword("agent")
            Destination.upsert_package(manifest, parse_package_with_groups(match[1]))
          elsif (match = statement.match(/\A(?:package|pray|use|include) (.+)\z/))
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
          label = (mode == "compose") ? "compose" : "tree"
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

          if (match = statement.match(/\Aagent (.+)\z/))
            manifest.note_deprecated_keyword("agent")
            apply_pray_statement(manifest, match[1], index)
            next
          end
          if (match = statement.match(/\A(?:pray|use|include|package) (.+)\z/))
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
          if (match = statement.match(/\Aagent (.+)\z/))
            manifest.note_deprecated_keyword("agent")
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
          if (match = statement.match(/\A(?:pray|use|include|package) (.+)\z/))
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

          manifest.note_deprecated_keyword("output") if statement.start_with?("output ")
          apply_target_statement(target, statement)
        end
        raise Error.parse("manifest", "missing 'end' for target block")
      end
    end
  end
end
