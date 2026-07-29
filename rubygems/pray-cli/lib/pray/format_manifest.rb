# frozen_string_literal: true

module Pray
  PackageFormatHint = Struct.new(:roles, :file_path, :exports) do
    def initialize(roles: [], file_path: nil, exports: [])
      super
    end
  end

  module FormatManifest
    module_function

    def uses_destination_dsl?(manifest)
      manifest.targets.any? { |target| target.scoped || target.mode != "legacy" } ||
        manifest.packages.any? { |package| package.bound || package.file } ||
        manifest.local.any?(&:bound)
    end

    def classify_format_hints(project)
      hints = {}
      project.packages.each do |package|
        roles = []
        file_path = package.declaration.file
        package.selected_exports.each do |export_name|
          export = package.spec.exports[export_name]
          next unless export

          %w[fragment folder file].each do |role|
            if Destination.export_kind_matches_role?(export.kind, role) && !roles.include?(role)
              roles << role
            end
          end
          if file_path.nil? && export.kind == "file"
            file_path = export.default_path || export_name
          end
        end
        hints[package.declaration.name] = PackageFormatHint.new(
          roles: roles,
          file_path: file_path,
          exports: ambiguous_exports_for_roles(package.selected_exports, package.spec.exports, roles)
        )
      end
      hints
    end

    def recommend_manifest(manifest, hints)
      recommended = if has_migratable_legacy_targets?(manifest)
        migrate_legacy_manifest(manifest, hints)
      else
        deep_clone_manifest(manifest)
      end
      omit_context_resolved_exports(recommended)
      omit_default_sources(recommended)
      recommended
    end

    def format_recommended(manifest, hints)
      recommended = recommend_manifest(manifest, hints)
      text = FormatSerialize.serialize_recommended(recommended)
      reparsed = Pray.parse_manifest(text)
      if reparsed.manifest_hash != recommended.manifest_hash
        raise Error.manifest("formatted Prayfile did not round-trip to an equivalent manifest")
      end

      text
    end

    def has_migratable_legacy_targets?(manifest)
      manifest.targets.any? do |target|
        !target.scoped && target.mode == "legacy" &&
          (!target.outputs.empty? || !target.skills.empty?)
      end
    end

    def migrate_legacy_manifest(manifest, hints)
      next_manifest = Manifest.new(
        prayfile_version: manifest.prayfile_version,
        sources: manifest.sources.map { |source| source.dup },
        targets: [],
        packages: manifest.packages.map { |package| clone_package(package) },
        local: manifest.local.map { |local| local.dup },
        symbols: manifest.symbols.dup,
        render: manifest.render.dup
      )

      apply_format_hints(next_manifest.packages, hints)

      compose_paths = unique_paths(
        manifest.targets.flat_map { |target| target.outputs.map { |path| [path, target.name] } }
      )
      tree_paths = unique_paths(
        manifest.targets.flat_map { |target| target.skills.map { |path| [path, target.name] } }
      )

      compose_paths.each do |path, target_names|
        target = Destination.new_destination_target("compose", path)
        locals_for_compose(next_manifest.local).each do |local|
          Destination.bind_local_entry(target, local.path)
          entry = next_manifest.local.find { |candidate| candidate.path == local.path }
          entry.bound = true if entry
        end
        packages_for_role(next_manifest.packages, "fragment", target_names).each do |package|
          Destination.bind_package_entry(target, package.name)
          mark_package_bound(next_manifest.packages, package.name, "fragment")
        end
        next_manifest.targets << target
      end

      tree_paths.each do |path, target_names|
        target = Destination.new_destination_target("tree", path)
        packages_for_role(next_manifest.packages, "folder", target_names).each do |package|
          Destination.bind_package_entry(target, package.name)
          mark_package_bound(next_manifest.packages, package.name, "folder")
        end
        next_manifest.targets << target
      end

      next_manifest.packages.each do |package|
        next unless package.file

        package.bound = true
        package.roles << "file" unless package.roles.include?("file")
      end
      next_manifest.local.each do |local|
        local.position = "after" if local.bound
      end

      manifest.targets.each do |target|
        next unless FormatSerialize.target_has_extras?(target)

        next_manifest.targets << ManifestTarget.new(
          name: target.name,
          commands: target.commands.dup,
          rules: target.rules.dup,
          max_bytes: target.max_bytes,
          mode: "legacy",
          scoped: false
        )
      end

      next_manifest
    end

    def apply_format_hints(packages, hints)
      packages.each do |package|
        hint = hints[package.name]
        if hint
          hint.roles.each do |role|
            package.roles << role unless package.roles.include?(role)
          end
          package.file ||= hint.file_path
          if package.exports.empty? && !hint.exports.empty?
            package.exports = hint.exports.dup
          end
        end
        if package.file && !package.roles.include?("file")
          package.roles << "file"
        end
      end
    end

    def omit_context_resolved_exports(manifest)
      manifest.packages.each do |package|
        package.exports.clear if package.bound && package.exports.length <= 1
      end
    end

    def omit_default_sources(manifest)
      sole_source = (manifest.sources.length == 1) ? manifest.sources.first.name : nil
      source_names = manifest.sources.map(&:name).to_set
      manifest.packages.each do |package|
        source = package.source
        next unless source

        matches_sole = sole_source == source
        namespace = package.name.split("/", 2).first
        matches_namespace = namespace == source && source_names.include?(source)
        package.source = nil if matches_sole || matches_namespace
      end
    end

    def unique_paths(items)
      map = {}
      items.each do |path, target_name|
        map[path] ||= Set.new
        map[path] << target_name
      end
      map.sort.map { |path, names| [path, names] }
    end

    def locals_for_compose(locals)
      before = []
      after = []
      locals.each do |local|
        next if local.bound

        if %w[start before].include?(local.position)
          before << local
        else
          after << local
        end
      end
      before + after
    end

    def packages_for_role(packages, role, target_names)
      packages.select do |package|
        next false if package.file
        if !package.targets.empty? && package.targets.none? { |name| target_names.include?(name) }
          next false
        end

        package.roles.include?(role)
      end
    end

    def mark_package_bound(packages, name, role)
      package = packages.find { |entry| entry.name == name }
      return unless package

      package.bound = true
      package.roles << role unless package.roles.include?(role)
    end

    def ambiguous_exports_for_roles(selected_exports, exports, roles)
      ambiguous = []
      roles.each do |role|
        matching = selected_exports.select do |export_name|
          export = exports[export_name]
          export && Destination.export_kind_matches_role?(export.kind, role)
        end
        next unless matching.length > 1

        matching.each do |export_name|
          ambiguous << export_name unless ambiguous.include?(export_name)
        end
      end
      ambiguous
    end

    def deep_clone_manifest(manifest)
      Marshal.load(Marshal.dump(manifest))
    end

    def clone_package(package)
      package.dup.tap do |copy|
        copy.exports = package.exports.dup
        copy.targets = package.targets.dup
        copy.features = package.features.dup
        copy.groups = package.groups.dup
        copy.roles = package.roles.dup
      end
    end
  end
end
