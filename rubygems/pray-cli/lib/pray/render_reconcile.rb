# frozen_string_literal: true

module Pray
  module RenderReconcile
    module_function

    def current?(path, display, fresh, retired = {})
      kind = RenderDest.destination_kind(path)
      return false if kind == :missing

      RenderDest.fail_if_symlink!(path, display)
      unless kind == :regular
        raise Error.render("refusing to read `#{display}`; destination is not a regular file")
      end
      existing = RenderDest.decode_utf8(RenderDest.read_regular_bytes(path, display), display)
      existing == RenderPatch.patch_rendered_content(existing, fresh, retired)
    end

    def retired_span_checksums(previous_lockfile, target)
      current_ids = target.managed_spans.to_h { |span| [span.id, true] }
      Array(previous_lockfile&.managed_span).filter_map do |span|
        next unless span.target == target.path
        next if current_ids[span.id]

        [span.id, span.ideal_checksum]
      end.to_h
    end

    def reject_managed_span_conflicts!(project, rendered, previous_lockfile)
      active_by_target = rendered.to_h do |target|
        [target.path, target.managed_spans.to_h { |span| [span.id, true] }]
      end
      Array(previous_lockfile.managed_span).group_by(&:target).each do |target_path, spans|
        active = active_by_target[target_path]
        next unless active

        PathSafety.validate_destination_path!(target_path)
        RenderDest.ensure_safe_destination_ancestors!(project.project_root, target_path, target_path)
        path = File.join(project.project_root, target_path)
        next unless File.file?(path)

        content = RenderDest.decode_utf8(RenderDest.read_regular_bytes(path, target_path), target_path)
        bodies = RenderPatch.managed_bodies(RenderPatch.split_segments(content))
        spans.each do |span|
          next unless active[span.id] && bodies.key?(span.id)
          next if Hashing.checksum_managed_span_content(bodies[span.id]) == span.ideal_checksum

          raise Error.render(
            "conflict: managed span `#{span.id}` in `#{target_path}` was edited " \
            "(`#{span.package}::#{span.export}`)"
          )
        end
      end
    end
  end
end
