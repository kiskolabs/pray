# frozen_string_literal: true

module Pray
  module Environment
    module_function

    def package_matches_environment?(groups, environment)
      return true if groups.empty?

      return false if environment.nil?

      groups.any? { |group| group == environment }
    end

    def collect_group_names(manifest)
      manifest.packages.flat_map(&:groups).uniq.sort
    end

    def validate_environment(manifest, environment)
      return if environment.nil?

      if environment.empty?
        raise Error.resolution("environment name cannot be empty")
      end

      known_groups = collect_group_names(manifest)
      if known_groups.empty?
        raise Error.resolution("unknown environment #{environment}; Prayfile defines no groups")
      end
      return if known_groups.include?(environment)

      raise Error.resolution(
        "unknown environment #{environment}; available groups are #{known_groups.join(", ")}"
      )
    end

    def should_render_package?(declaration, environment)
      package_matches_environment?(declaration.groups, environment)
    end
  end
end
