# frozen_string_literal: true

module Pray
  DEFAULT_LOCAL_PRAYER_NAME = "project"
  DEFAULT_PATH_SOURCE_NAME = "local"
  DEFAULT_PATH_SOURCE_DIRECTORY = "prayers"

  module LocalPrayer
    module_function

    def validate_name!(name)
      trimmed = name.to_s.strip
      raise Error.usage("prayer name is missing") if trimmed.empty?
      if trimmed.include?("/") || trimmed.include?("\\") || trimmed == "." || trimmed == ".."
        raise Error.usage("prayer name must be a single folder name")
      end
      if reserved_distribution_layout_name?(trimmed)
        raise Error.usage("#{trimmed} is reserved for the distribution layout")
      end

      trimmed
    end

    def path_source_package_directory(source_name, package_name)
      prefix = "#{source_name}/"
      rest = package_name.delete_prefix(prefix)
      if rest != package_name && !rest.empty? && !rest.include?("/") && !rest.include?("\\")
        return rest
      end

      package_name.to_s.tr("/", "-").tr("\\", "-")
    end

    def reserved_distribution_layout_name?(name)
      name.match?(/\Av[0-9]+\z/)
    end
  end
end
