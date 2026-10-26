# frozen_string_literal: true

module Pray
  module Render
    module_function

    def render_project(project)
      project.manifest.targets.flat_map do |target|
        target.outputs.map do |output|
          rendered = render_target(project, target, output)
          if target.max_bytes && rendered.content.bytesize > target.max_bytes
            raise Error.render(
              "target `#{target.name}` output `#{output}` is #{rendered.content.bytesize} bytes; " \
              "max_bytes is #{target.max_bytes}"
            )
          end
          rendered
        end
      end
    end
  end
end
