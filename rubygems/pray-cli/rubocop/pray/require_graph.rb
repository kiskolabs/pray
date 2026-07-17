# frozen_string_literal: true

require "pathname"

module RuboCop
  module Pray
    module RequireGraph
      module_function

      def cycles(lib_root)
        graph = build_graph(lib_root)
        find_cycles(graph)
      end

      def expand_path(path)
        Pathname.new(path).expand_path.realpath.to_s
      rescue Errno::ENOENT
        Pathname.new(path).expand_path.to_s
      end

      def build_graph(lib_root)
        root = Pathname.new(lib_root).expand_path
        graph = {}

        root.glob("**/*.rb").sort.each do |file_path|
          absolute_path = expand_path(file_path)
          graph[absolute_path] = require_edges(file_path)
        end

        graph
      end

      def require_edges(file_path)
        content = file_path.read(encoding: Encoding::UTF_8)
        directory = file_path.dirname
        edges = []

        content.scan(/^\s*require_relative\s+["']([^"']+)["']/) do |match|
          relative = match.first
          required = if relative.end_with?(".rb")
            directory.join(relative)
          else
            directory.join("#{relative}.rb")
          end
          next unless required.file?

          edges << expand_path(required)
        end

        edges
      end

      def find_cycles(graph)
        cycles = []
        visited = {}

        depth_first_search = lambda do |node, stack, stack_set|
          return if visited[node] == :closed

          if stack_set[node]
            cycle_start = stack.index(node)
            cycles << (stack[cycle_start..] + [node])
            return
          end

          return if visited[node]

          visited[node] = :open
          stack << node
          stack_set[node] = true

          graph.fetch(node, []).each do |neighbor|
            depth_first_search.call(neighbor, stack, stack_set)
          end

          stack.pop
          stack_set.delete(node)
          visited[node] = :closed
        end

        graph.each_key do |node|
          depth_first_search.call(node, [], {})
        end

        cycles
      end
    end
  end
end
