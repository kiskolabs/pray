# frozen_string_literal: true

module Pray
  ResolvedProject = Struct.new(
    :manifest_path, :project_root, :manifest, :manifest_hash, :packages,
    :local_files, :source_revisions, :source_host_keys,
    keyword_init: true
  ) do
    def lockfile_hash
      manifest_hash
    end
  end

  ResolvedPackage = Struct.new(
    :declaration, :root, :spec, :tree_hash, :artifact_hash, :artifact,
    :selected_exports, :source_checksum, :export_bodies, :skill_files,
    :signer_fingerprint, :registry_latest_version,
    keyword_init: true
  )

  ResolvedLocalFile = Struct.new(
    :path, :manifest_path, :content, :position, :optional,
    keyword_init: true
  )

  module Resolve
    module_function

    def project_root_from_manifest(manifest_path)
      parent = File.dirname(manifest_path)
      parent.empty? || parent == "." ? "." : parent
    end

    def canonical_project_root(manifest_path)
      root = project_root_from_manifest(manifest_path)
      return Pathname(root).cleanpath.to_s if Pathname(root).absolute?

      File.expand_path(root)
    end

    def resolve_project(manifest_path, offline: false, refresh: false)
      user_config = Config.load_user_config
      project_root = canonical_project_root(manifest_path)
      lockfile_path = File.join(project_root, "Prayfile.lock")
      lockfile_hints = File.exist?(lockfile_path) ? Pray.read_lockfile(lockfile_path) : nil
      manifest_text = Pray.read_manifest_text(manifest_path)
      manifest = Pray.parse_manifest(manifest_text)
      manifest_hash = manifest.manifest_hash
      sources = source_map(manifest.sources)
      git_sources = GitSources.prepare_git_sources(
        project_root,
        manifest.sources,
        lockfile_hints,
        refresh: refresh
      )
      source_revisions = git_sources.transform_values(&:revision)
      source_host_keys = Trust.prepare_source_host_keys(manifest.sources)

      packages = []
      seen = {}
      errors = []
      manifest.packages.each do |declaration|
        begin
          package = resolve_package(
            project_root,
            sources,
            git_sources,
            user_config,
            declaration,
            lockfile_hints,
            offline: offline
          )
          if seen[package.declaration.name]
            raise Error.resolution("duplicate package declaration: #{package.declaration.name}")
          end

          seen[package.declaration.name] = true
          packages << package
        rescue Error => error
          errors << "#{declaration.name}: #{error.message}"
        end
      end
      raise Error.resolution(errors.join("\n")) unless errors.empty?

      local_files = []
      local_errors = []
      manifest.local.each do |local|
        begin
          local_files << resolve_local_file(project_root, local)
        rescue Error => error
          local_errors << "local #{local.path}: #{error.message}"
        end
      end
      raise Error.resolution(local_errors.join("\n")) unless local_errors.empty?

      ResolvedProject.new(
        manifest_path: manifest_path,
        project_root: project_root,
        manifest: manifest,
        manifest_hash: manifest_hash,
        packages: packages,
        local_files: local_files,
        source_revisions: source_revisions,
        source_host_keys: source_host_keys
      )
    end

    def missing_local_embed_guidance(path)
      "Prayfile lists `local \"#{path}\"` but the file does not exist. " \
        "Create the file or remove the entry from Prayfile, then run `pray install`."
    end

    def resolve_package(project_root, sources, git_sources, user_config, declaration, lockfile, offline: false)
      root, registry_latest_version = resolve_package_root_with_metadata(
        project_root, sources, git_sources, user_config, declaration, lockfile, offline: offline
      )
      spec_path = find_prayspec_file(root)
      spec_text = File.read(spec_path)
      spec = Pray.parse_package_spec(spec_text).canonicalized
      if spec.name != declaration.name
        raise Error.resolution(
          "package path #{root.inspect} declares #{spec.name.inspect}, expected #{declaration.name.inspect}"
        )
      end
      unless Constraint.version_satisfies(spec.version, declaration.constraint)
        raise Error.resolution(
          "package #{declaration.name} version #{spec.version} does not satisfy constraint #{declaration.constraint}"
        )
      end

      selected_exports = select_exports(declaration, spec)
      file_bytes = load_package_file_bytes(root, spec)
      tree_hash = PackageSpec.tree_hash_from_file_bytes(file_bytes)
      export_bodies = load_export_bodies(file_bytes, spec, selected_exports)
      skill_files = build_skill_file_index(spec)

      ResolvedPackage.new(
        declaration: declaration,
        root: root,
        spec: spec,
        tree_hash: tree_hash,
        artifact_hash: tree_hash,
        artifact: "path:#{File.dirname(spec_path)}",
        selected_exports: selected_exports,
        source_checksum: tree_hash,
        export_bodies: export_bodies,
        skill_files: skill_files,
        signer_fingerprint: nil,
        registry_latest_version: registry_latest_version
      )
    end

    def resolve_package_root_with_metadata(
      project_root, sources, git_sources, user_config, declaration, lockfile, offline: false
    )
      if (local_path = user_config.local.package[declaration.name])
        return [File.expand_path(local_path, project_root), nil]
      end
      return [File.expand_path(declaration.path, project_root), nil] if declaration.path

      if declaration.source
        source = sources[declaration.source]
        raise Error.resolution("unknown source: #{declaration.source}") unless source

        if (local_path = user_config.local.source[declaration.source])
          source_root = File.expand_path(local_path, project_root)
          resolved = Registry.resolve_local_registry_package_root(
            project_root,
            "local:#{declaration.source}",
            source_root,
            declaration,
            preferred_version: lockfile_preferred_version(lockfile, declaration.name),
            offline: offline
          )
          return [resolved.root, resolved.registry_latest_version]
        end

        case source.kind
        when "path"
          return [File.join(project_root, source.url, declaration.name.tr("/", "-")), nil]
        when "registry", "static index"
          resolved = Registry.resolve_registry_package_root(
            project_root,
            source.url,
            declaration,
            preferred_version: lockfile_preferred_version(lockfile, declaration.name),
            offline: offline
          )
          return [resolved.root, resolved.registry_latest_version]
        when "pray_ssh"
          raise Error.unsupported("pray_ssh sources are not implemented yet in pray-cli Ruby")
        when "git"
          checkout = git_sources[source.name]
          raise Error.resolution("git source #{source.name} was not prepared") unless checkout

          clone_url = source.url.delete_prefix("git+")
          distribution_root = GitSources.resolve_distribution_root(checkout.cache_directory, checkout.subdir)
          source_key = checkout.revision.to_s.empty? ? clone_url : "#{clone_url}@#{checkout.revision}"
          resolved = Registry.resolve_local_registry_package_root(
            project_root,
            source_key,
            distribution_root,
            declaration,
            preferred_version: lockfile_preferred_version(lockfile, declaration.name),
            offline: offline
          )
          return [resolved.root, resolved.registry_latest_version]
        else
          raise Error.unsupported("source kind #{source.kind} not implemented yet")
        end
      end

      if declaration.git || declaration.tarball || declaration.oci
        raise Error.unsupported("remote sources are not implemented yet")
      end

      [File.join(project_root, declaration.name.tr("/", "-")), nil]
    end

    def resolve_package_root(project_root, sources, git_sources, user_config, declaration, lockfile = nil)
      resolve_package_root_with_metadata(project_root, sources, git_sources, user_config, declaration, lockfile).first
    end

    def lockfile_preferred_version(lockfile, package_name)
      return nil unless lockfile

      lockfile.package.find { |entry| entry.name == package_name }&.version
    end

    def resolve_local_file(project_root, declaration)
      path = File.join(project_root, declaration.path)
      unless File.exist?(path)
        if declaration.optional
          return ResolvedLocalFile.new(
            path: path,
            manifest_path: declaration.path,
            content: "",
            position: declaration.position,
            optional: true
          )
        end
        raise Error.resolution(missing_local_embed_guidance(declaration.path))
      end

      ResolvedLocalFile.new(
        path: path,
        manifest_path: declaration.path,
        content: Hashing.normalize_line_endings(File.read(path)),
        position: declaration.position,
        optional: declaration.optional
      )
    end

    def find_prayspec_file(root)
      files = Dir.children(root).filter_map do |entry|
        path = File.join(root, entry)
        File.file?(path) && File.extname(entry) == ".prayspec" ? path : nil
      end
      case files.length
      when 1 then files.first
      when 0 then raise Error.resolution("no prayspec file found in #{root.inspect}")
      else raise Error.resolution("multiple prayspec files found in #{root.inspect}")
      end
    end

    def source_map(sources)
      sources.to_h { |source| [source.name, source] }
    end

    def select_exports(declaration, spec)
      return spec.exports.keys.sort if declaration.exports.empty?

      declaration.exports.each do |export|
        unless spec.exports.key?(export)
          raise Error.resolution("package #{declaration.name} does not export #{export}")
        end
      end
      declaration.exports
    end

    def load_package_file_bytes(root, spec)
      file_bytes = {}
      spec.files.each do |file|
        path = File.join(root, file)
        raise Error.integrity("package file missing: #{file}") unless File.exist?(path)
        raise Error.integrity("package file is a directory: #{file}") if File.directory?(path)

        file_bytes[file] = File.binread(path)
      end
      file_bytes
    end

    def load_export_bodies(file_bytes, spec, selected_exports)
      export_bodies = {}
      selected_exports.each do |export_name|
        entry = spec.exports[export_name]
        raise Error.resolution("package #{spec.name} is missing export #{export_name}") unless entry
        next unless entry.kind == "fragment"

        bytes = file_bytes[entry.path]
        raise Error.integrity("package file missing for export #{export_name}: #{entry.path}") unless bytes

        export_bodies[export_name] = Hashing.normalize_line_endings(bytes.force_encoding(Encoding::UTF_8))
      end
      export_bodies
    end

    def build_skill_file_index(spec)
      index = {}
      spec.exports.each do |export_name, export|
        next unless %w[folder skill].include?(export.kind)

        folder_prefix = export.path.delete_suffix("/")
        files = indexed_files_under_prefix(spec.files, folder_prefix)
        index[export_name] = files unless files.empty?
      end
      spec.skills.each do |skill_name, skill|
        next if index.key?(skill_name)

        skill_prefix = skill.path.delete_suffix("/")
        files = indexed_files_under_prefix(spec.files, skill_prefix)
        index[skill_name] = files unless files.empty?
      end
      index
    end

    def indexed_files_under_prefix(files, prefix)
      files.filter_map { |file| skill_relative_file(file, prefix) }
    end

    def skill_relative_file(file, skill_prefix)
      return nil unless file.start_with?(skill_prefix)

      relative = file.delete_prefix(skill_prefix).delete_prefix("/")
      return nil if relative.empty? || file == skill_prefix

      relative
    end
  end
end
