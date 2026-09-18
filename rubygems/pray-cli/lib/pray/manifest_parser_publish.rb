# frozen_string_literal: true

module Pray
  module ManifestMethods
    module ParserPublish
      def apply_publish(manifest, rest)
        is_block = rest.rstrip.end_with?(" do")
        header = rest.sub(/\s+do\z/, "").strip
        remote = parse_publish_header(header)
        parse_publish_block(remote) if is_block
        manifest.publish_remotes << remote
      end

      def parse_publish_header(rest)
        values, keywords = parse_call(rest)
        raise Error.parse("manifest", "publish requires a name") if values.empty?

        name = string_from_value(values.first)
        if %w[git source signing_key token].any? { |key| keywords.key?(key) }
          raise Error.parse(
            "manifest",
            "publish \"#{name}\" does not take git:, source:, signing_key:, or token:"
          )
        end
        ManifestPublishRemote.new(
          name: name,
          path: keyword_string(keywords, "path"),
          url: values[1] && string_from_value(values[1]),
          packages: []
        )
      end

      def parse_publish_block(remote)
        while (statement = next_statement)
          return if statement == "end"

          pray_rest = statement[/\A(?:pray|use|include|agent|package) (.+)\z/, 1]
          unless pray_rest
            raise Error.parse("manifest", "publish blocks only support pray package names: #{statement}")
          end

          package = parse_package_decl(pray_rest)
          if remote.packages.include?(package.name)
            raise Error.parse("manifest", "duplicate package #{package.name} in publish \"#{remote.name}\"")
          end
          remote.packages << package.name
        end
        raise Error.parse("manifest", "missing 'end' for publish block")
      end
    end
  end
end
