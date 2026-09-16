# frozen_string_literal: true

module Pray
  module Resolve
    module_function

    def missing_local_embed_guidance(path)
      "Prayfile lists `local \"#{path}\"` but the file does not exist. " \
        "Create the file or remove the entry from Prayfile, then run `pray install`."
    end

    def resolve_local_file(project_root, declaration)
      path = File.join(project_root, declaration.path)
      unless File.exist?(path)
        if declaration.optional
          return ResolvedLocalFile.new(
            path: path,
            manifest_path: declaration.path,
            content: "",
            source_checksum: Hashing.sha256_prefixed(""),
            position: declaration.position,
            optional: true
          )
        end
        raise Error.resolution(missing_local_embed_guidance(declaration.path))
      end

      content = Hashing.normalize_line_endings(File.read(path))
      ResolvedLocalFile.new(
        path: path,
        manifest_path: declaration.path,
        content: content,
        source_checksum: Hashing.sha256_prefixed(content),
        position: declaration.position,
        optional: declaration.optional
      )
    end
  end
end
