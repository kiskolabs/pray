# frozen_string_literal: true

module Pray
  module Render
    module_function

    def layout_rendered_targets(project, rendered)
      rendered.map do |target|
        PathSafety.validate_destination_path!(target.path)
        RenderDest.ensure_safe_destination_ancestors!(project.project_root, target.path, target.path)
        path = File.join(project.project_root, target.path)
        content = RenderDest.layout_rendered_content(path, target.path, target.content)
        RenderedTarget.new(
          path: target.path,
          content: content,
          managed_spans: RenderPatch.relocate_managed_spans(content, target.managed_spans)
        )
      end
    end
  end
end
