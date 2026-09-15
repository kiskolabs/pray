# frozen_string_literal: true

module Pray
  module Upstream
    module_function

    def identity_path?(path)
      File.extname(path.to_s) == ".prayspec"
    end

    def content_paths(files)
      files.reject { |path| identity_path?(path) }
    end

    def clean_replica?(old_content, local_content)
      local_content == old_content
    end

    def try_merge_content_files(old_content, new_content, local_content)
      return [new_content.dup, []] if clean_replica?(old_content, local_content)

      merged = {}
      conflicts = []
      (old_content.keys + new_content.keys + local_content.keys).uniq.sort.each do |path|
        result = merge_one_content_path(old_content[path], new_content[path], local_content[path])
        if result == :conflict
          conflicts << path
        elsif result != :omit
          merged[path] = result
        end
      end
      [merged, conflicts]
    end

    def merge_one_content_path(old, new, local)
      case [old.nil?, new.nil?, local.nil?]
      when [false, false, false]
        merge_all_present(old, new, local)
      when [false, true, false]
        (local == old) ? :omit : :conflict
      when [false, false, true]
        (old == new) ? :omit : new
      when [true, false, true]
        new
      when [true, true, false]
        local
      when [true, false, false]
        (local == new) ? new : :conflict
      else
        :conflict
      end
    end

    def merge_all_present(old, new, local)
      return new if local == new || local == old
      return local if old == new

      :conflict
    end

    def merge_content_files(old_content, new_content, local_content)
      merged, conflicts = try_merge_content_files(old_content, new_content, local_content)
      return merged if conflicts.empty?

      raise Error.resolution("upstream merge conflict in #{conflicts.join(", ")}")
    end

    def merge_conflict_message(fork, upstream_name, old_version, new_version, paths)
      "upstream merge conflict in #{fork} while refreshing #{upstream_name} " \
        "#{old_version} to #{new_version}: #{paths.join(", ")}"
    end

    def next_upstream_constraint(current, new_version)
      trimmed = current.to_s.strip
      return current if trimmed == "*" || trimmed.start_with?("~>", "^", ">=", ">", "<=", "<")

      "= #{new_version}"
    end
    private_class_method :merge_one_content_path, :merge_all_present
  end
end
