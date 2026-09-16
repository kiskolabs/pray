# frozen_string_literal: true

module Pray
  module CLI
    def resolve_local_prayer_directory(requested)
      manifest = Pray.parse_manifest(Pray.read_manifest_text(manifest_path))
      path_sources = path_sources_for(manifest)
      if requested
        directory = requested.to_s.strip
        raise Error.usage("--path requires a directory") if directory.empty?

        matching = path_sources.find { |source| source.url == directory }
        return [matching.name, matching.url] if matching
        if path_sources.length == 1
          raise Error.usage("path source already uses #{path_sources.first.url}")
        end

        name = ensure_path_source(directory)
        return [name, directory]
      end

      case path_sources.length
      when 0
        name = ensure_path_source(DEFAULT_PATH_SOURCE_DIRECTORY)
        [name, DEFAULT_PATH_SOURCE_DIRECTORY]
      when 1
        [path_sources.first.name, path_sources.first.url]
      else
        raise Error.usage("say which directory with --path")
      end
    end

    def declare_local_prayer(package_name)
      manifest_text = Pray.read_manifest_text(manifest_path)
      manifest = Pray.parse_manifest(manifest_text)
      return if manifest.packages.any? { |package| package.name == package_name }

      statement = "pray \"#{package_name}\""
      updated = insert_into_first_compose(manifest_text, statement) ||
        insert_manifest_statement(manifest_text, statement)
      Transaction.write_file(manifest_path, updated)
    end

    def path_sources_for(manifest)
      manifest.sources.select { |source| source.kind == "path" }
    end

    def ensure_path_source(directory)
      manifest_text = Pray.read_manifest_text(manifest_path)
      manifest = Pray.parse_manifest(manifest_text)
      existing = manifest.sources.find { |source| source.kind == "path" && source.url == directory }
      return existing.name if existing

      name = unused_source_name(manifest, directory)
      statement = "source \"#{name}\", path: \"#{directory}\""
      Transaction.write_file(manifest_path, insert_source_statement(manifest_text, statement))
      name
    end

    def unused_source_name(manifest, directory)
      taken = manifest.sources.map(&:name)
      return DEFAULT_PATH_SOURCE_NAME unless taken.include?(DEFAULT_PATH_SOURCE_NAME)

      fallback = directory.to_s.split(%r{[/\\]}).reject(&:empty?).last || DEFAULT_PATH_SOURCE_NAME
      raise Error.manifest("source name #{fallback} is already used") if taken.include?(fallback)

      fallback
    end

    def insert_source_statement(text, statement)
      lines = text.lines.map(&:chomp)
      last_source = lines.rindex { |line| line.lstrip.start_with?("source ") }
      insertion_index = if last_source
        last_source + 1
      else
        prayfile = lines.index { |line| line.lstrip.start_with?("prayfile ") }
        prayfile ? prayfile + 1 : 0
      end
      lines.insert(insertion_index, statement)
      join_manifest_lines(lines)
    end

    def insert_into_first_compose(text, statement)
      lines = text.lines.map(&:chomp)
      index = lines.index do |line|
        trimmed = line.lstrip
        trimmed.start_with?("compose ") && trimmed.end_with?(" do")
      end
      return unless index

      indent = lines[index][/\A\s*/].to_s
      lines.insert(index + 1, "#{indent}  #{statement}")
      join_manifest_lines(lines)
    end

    def join_manifest_lines(lines)
      output = lines.join("\n")
      output += "\n" unless output.end_with?("\n")
      output
    end
  end
end
