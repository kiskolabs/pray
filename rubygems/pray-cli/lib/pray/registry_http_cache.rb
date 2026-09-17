# frozen_string_literal: true

module Pray
  module RegistryHttpCache
    module_function

    def fetch_package_metadata(source_url, package_name)
      @cache ||= {}
      key = [source_url, package_name]
      return @cache[key] if @cache.key?(key)

      PathSafety.reject_unsafe_package_name!(package_name)
      response = Registry.http_get(Registry.join_url(source_url, "v1/packages/#{package_name}.json"))
      @cache[key] = Registry.parse_metadata(response)
    end
  end
end
