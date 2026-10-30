# frozen_string_literal: true

module Pray
  module CLI
    def search_command(query:, source:, root:, url:)
      hits = if root
        RegistrySearch.search_local_registry(root, query)
      elsif url
        RegistrySearch.search_http_registry(url, query)
      else
        search_declared_source(query, source)
      end
      if hits.empty?
        puts "no packages matched"
      else
        hits.each { |hit| puts(hit.summary ? "#{hit.name} — #{hit.summary}" : hit.name) }
      end
    end

    def search_declared_source(query, source_name)
      manifest = Pray.parse_manifest(Pray.read_manifest_text(manifest_path))
      source = source_name ? manifest.sources.find { |entry| entry.name == source_name } : searchable_source(manifest)
      raise Error.resolution("unknown source: #{source_name}") if source_name && !source
      unless source
        raise Error.resolution("Prayfile has no searchable sources; pass --root PATH or --url URL")
      end

      case source.kind
      when "path"
        root = File.expand_path(source.url, File.dirname(manifest_path))
        RegistrySearch.search_local_registry(root, query)
      when "registry", "static index"
        RegistrySearch.search_http_registry(source.url, query)
      else
        raise Error.unsupported("search does not support source kind #{source.kind}")
      end
    end

    def searchable_source(manifest)
      manifest.sources.find { |source| %w[registry static\ index].include?(source.kind) } || manifest.sources.first
    end
  end
end
