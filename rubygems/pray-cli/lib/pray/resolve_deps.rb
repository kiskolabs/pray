# frozen_string_literal: true

module Pray
  module ResolveDeps
    module_function

    def reject_dependency_cycles(packages)
      edges = {}
      packages.each do |package|
        edges[package.declaration.name] = package.spec.dependencies.reject(&:optional).map(&:name)
      end
      cycle = DependencyGraph.find_dependency_cycle(edges)
      return unless cycle

      raise Error.resolution("dependency cycle detected: #{cycle.join(" -> ")}")
    end

    def validate_optional_referrals(packages)
      packages_by_name = packages.to_h { |package| [package.declaration.name, package] }
      packages.each do |package|
        package.spec.dependencies.select(&:optional).each do |dependency|
          target = packages_by_name[dependency.name]
          next unless target

          target.spec.satisfy_constraint!(dependency.constraint)
        rescue Error
          raise Error.resolution(
            "optional referral #{package.declaration.name} -> #{dependency.name} requires " \
              "#{dependency.constraint}, but selected version is #{target.spec.recorded_version}"
          )
        end
      end
    end
  end
end
