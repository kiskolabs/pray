# frozen_string_literal: true

module Pray
  module ResolveDeps
    module_function

    def reject_dependency_cycles(packages)
      edges = {}
      packages.each do |package|
        edges[package.declaration.name] = package.spec.dependencies.map(&:name)
      end
      cycle = DependencyGraph.find_dependency_cycle(edges)
      return unless cycle

      raise Error.resolution("dependency cycle detected: #{cycle.join(" -> ")}")
    end
  end
end
