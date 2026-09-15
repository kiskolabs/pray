# frozen_string_literal: true

module Pray
  module DependencyGraph
    module_function

    def find_dependency_cycle(edges)
      visiting = {}
      visited = {}
      stack = []
      edges.keys.sort.each do |name|
        next if visited[name]

        cycle = depth_first_search(name, edges, visiting, visited, stack)
        return cycle if cycle
      end
      nil
    end

    def depth_first_search(name, edges, visiting, visited, stack)
      return nil if visited[name]
      if visiting[name]
        start = stack.index(name)
        return nil unless start

        return stack[start..] + [name]
      end

      visiting[name] = true
      stack << name
      Array(edges[name]).each do |dependency|
        next unless edges.key?(dependency)

        cycle = depth_first_search(dependency, edges, visiting, visited, stack)
        return cycle if cycle
      end
      stack.pop
      visiting.delete(name)
      visited[name] = true
      nil
    end
  end
end
