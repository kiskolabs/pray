# frozen_string_literal: true

require "json"

module Pray
  RegistrySearchHit = Struct.new(:name, :summary, keyword_init: true)

  module RegistrySearch
    module_function

    def search_local_registry(root, query, include_summary: true)
      index = parse_index(File.read(File.join(root, "v1", "index.json"), encoding: "UTF-8"))
      search_index_names(index, query).map do |name|
        metadata_path = Registry.registry_metadata_path(root, name)
        metadata = Registry.parse_metadata(File.read(metadata_path, encoding: "UTF-8")) if include_summary && File.file?(metadata_path)
        search_hit(name, metadata)
      end
    end

    def search_http_registry(url, query, include_summary: true)
      index = parse_index(Registry.http_get(Registry.join_url(url, "v1/index.json")))
      search_index_names(index, query).map do |name|
        PathSafety.reject_unsafe_package_name!(name)
        metadata = fetch_http_metadata(url, name) if include_summary
        search_hit(name, metadata)
      end
    end

    def search_index_names(index, query)
      needle = query.strip.downcase
      index.fetch("packages").select do |name|
        raise Error.parse("registry index", "package names must be strings") unless name.is_a?(String)

        PathSafety.reject_unsafe_package_name!(name)
        name.downcase.include?(needle)
      end.sort
    end

    def latest_non_yanked_summary(metadata)
      return unless metadata

      latest = metadata.versions.reject(&:yanked).max_by { |version| Gem::Version.new(version.version) }
      summary = latest&.derived_metadata&.fetch("summary", nil).to_s.strip
      summary unless summary.empty?
    end

    def parse_index(text)
      index = JSON.parse(text)
      unless index["spec"] == "prayfile-distribution-1" && index["packages"].is_a?(Array)
        raise Error.parse("registry index", "expected prayfile-distribution-1 with a packages array")
      end
      index
    rescue JSON::ParserError => error
      raise Error.parse("registry index", error.message)
    end

    def fetch_http_metadata(url, name)
      path = "v1/packages/#{name}.json"
      Registry.parse_metadata(Registry.http_get(Registry.join_url(url, path)))
    rescue Error => error
      raise unless error.category == :resolution

      nil
    end

    def search_hit(name, metadata)
      if metadata && metadata.name != name
        raise Error.integrity("registry metadata name mismatch for #{name}")
      end
      RegistrySearchHit.new(name: name, summary: latest_non_yanked_summary(metadata))
    end
  end
end
