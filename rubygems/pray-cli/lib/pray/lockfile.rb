# frozen_string_literal: true

require "toml-rb"

require_relative "lockfile_serialize"

module Pray
  LockSource = Struct.new(:name, :kind, :url, :revision, :host_key_fingerprint, keyword_init: true)
  LockedPackage = Struct.new(
    :name, :version, :source, :path, :tree_hash, :artifact_hash, :artifact,
    :exports, :dependencies, :signer_fingerprint,
    keyword_init: true
  ) do
    def initialize(exports: [], dependencies: [], signer_fingerprint: nil, source: nil, **kwargs)
      super(**kwargs, exports: exports, dependencies: dependencies, signer_fingerprint: signer_fingerprint, source: source)
    end
  end

  LockedTarget = Struct.new(:name, :outputs, keyword_init: true)
  ManagedSpanRecord = Struct.new(
    :id, :target, :open_line, :close_line, :ideal_checksum, :package, :export,
    :source_checksum, :silenced,
    keyword_init: true
  )

  Lockfile = Struct.new(
    :prayfile_lock, :spec, :generated_by, :manifest_hash, :environment, :source, :package,
    :target, :managed_span,
    keyword_init: true
  ) do
    def initialize(
      prayfile_lock: "1", spec: "0.1", generated_by: Pray::GENERATED_BY,
      manifest_hash: "", environment: nil, source: [], package: [], target: [], managed_span: []
    )
      super
    end

    def canonicalized
      dup.tap do |copy|
        copy.source = source.sort_by(&:name)
        copy.package = package.sort_by { |entry| [entry.name, entry.source.to_s, entry.version] }
        copy.target = target.sort_by(&:name)
        copy.managed_span = managed_span.sort_by { |span| [span.target, span.open_line, span.id] }
      end
    end

    def serialized
      LockfileIO.lockfile_to_toml(canonicalized)
    end

    def file_hash
      Hashing.sha256_prefixed(serialized)
    end

    def equivalent_to?(other)
      canonicalized == other.canonicalized
    end
  end

  module LockfileIO
    module_function

    def lockfile_to_toml(lockfile)
      LockfileSerialize.lockfile_to_toml(lockfile)
    end

    def lockfile_to_hash(lockfile)
      {
        "prayfile_lock" => lockfile.prayfile_lock,
        "spec" => lockfile.spec,
        "generated_by" => lockfile.generated_by,
        "manifest_hash" => lockfile.manifest_hash,
        "source" => lockfile.source.map { |entry| source_to_hash(entry) },
        "package" => lockfile.package.map { |entry| package_to_hash(entry) },
        "target" => lockfile.target.map { |entry| { "name" => entry.name, "outputs" => entry.outputs } },
        "managed_span" => lockfile.managed_span.map { |entry| managed_span_to_hash(entry) }
      }
    end

    def source_to_hash(entry)
      hash = { "name" => entry.name, "kind" => entry.kind, "url" => entry.url }
      hash["revision"] = entry.revision if entry.revision
      hash["host_key_fingerprint"] = entry.host_key_fingerprint if entry.host_key_fingerprint
      hash
    end

    def package_to_hash(entry)
      hash = {
        "name" => entry.name,
        "version" => entry.version,
        "path" => entry.path,
        "tree_hash" => entry.tree_hash,
        "artifact_hash" => entry.artifact_hash,
        "artifact" => entry.artifact,
        "exports" => entry.exports,
        "dependencies" => entry.dependencies
      }
      hash["source"] = entry.source unless entry.source.nil?
      hash["signer_fingerprint"] = entry.signer_fingerprint if entry.signer_fingerprint
      hash
    end

    def managed_span_to_hash(entry)
      {
        "id" => entry.id,
        "target" => entry.target,
        "open_line" => entry.open_line,
        "close_line" => entry.close_line,
        "ideal_checksum" => entry.ideal_checksum,
        "package" => entry.package,
        "export" => entry.export,
        "source_checksum" => entry.source_checksum,
        "silenced" => entry.silenced
      }
    end

    def parse_lockfile(text)
      data = TomlRB.parse(text)
      from_hash(data)
    rescue TomlRB::ParseError => error
      raise Error.parse("lockfile", error.message)
    end

    def read_lockfile(path)
      parse_lockfile(File.read(path))
    end

    def serialize_lockfile(lockfile)
      lockfile_to_toml(lockfile.canonicalized)
    end

    def lockfile_hash(lockfile)
      Hashing.sha256_prefixed(serialize_lockfile(lockfile))
    end

    def write_lockfile(path, lockfile)
      File.write(path, serialize_lockfile(lockfile))
    end

    def write_lockfile_if_changed(path, lockfile)
      serialized = serialize_lockfile(lockfile)
      if File.exist?(path) && File.binread(path) == serialized
        return
      end

      File.write(path, serialized)
    end

    def lockfiles_equivalent?(left, right)
      left.equivalent_to?(right)
    end

    def build_lockfile(manifest_hash, environment, project_root, manifest_sources, manifest_targets, rendered, packages, source_revisions, source_host_keys)
      Lockfile.new(
        manifest_hash: manifest_hash,
        environment: environment,
        source: manifest_sources.map do |source|
          LockSource.new(
            name: source.name,
            kind: source.kind,
            url: source.url,
            revision: source_revisions[source.name],
            host_key_fingerprint: source_host_keys[source.name]
          )
        end,
        package: packages.map do |package|
          LockedPackage.new(
            name: package.declaration.name,
            version: package.spec.version,
            source: package.declaration.source,
            path: relative_lockfile_path(project_root, package.root),
            tree_hash: package.tree_hash,
            artifact_hash: package.artifact_hash,
            artifact: normalize_lockfile_artifact(project_root, package.artifact, package.root),
            exports: package.selected_exports,
            dependencies: package.spec.dependencies.map(&:name),
            signer_fingerprint: package.signer_fingerprint
          )
        end,
        target: manifest_targets.map { |target| LockedTarget.new(name: target.name, outputs: target.outputs) },
        managed_span: rendered.flat_map(&:managed_spans)
      ).canonicalized
    end

    def from_hash(data)
      Lockfile.new(
        prayfile_lock: data["prayfile_lock"],
        spec: data["spec"],
        generated_by: data["generated_by"],
        manifest_hash: data["manifest_hash"],
        environment: data["environment"],
        source: Array(data["source"]).map do |entry|
          LockSource.new(
            name: entry["name"],
            kind: entry["kind"],
            url: entry["url"],
            revision: entry["revision"],
            host_key_fingerprint: entry["host_key_fingerprint"]
          )
        end,
        package: Array(data["package"]).map do |entry|
          LockedPackage.new(
            name: entry["name"],
            version: entry["version"],
            source: entry["source"],
            path: entry["path"],
            tree_hash: entry["tree_hash"],
            artifact_hash: entry["artifact_hash"],
            artifact: entry["artifact"],
            exports: entry["exports"] || [],
            dependencies: entry["dependencies"] || [],
            signer_fingerprint: entry["signer_fingerprint"]
          )
        end,
        target: Array(data["target"]).map { |entry| LockedTarget.new(name: entry["name"], outputs: entry["outputs"] || []) },
        managed_span: Array(data["managed_span"]).map do |entry|
          ManagedSpanRecord.new(
            id: entry["id"],
            target: entry["target"],
            open_line: entry["open_line"],
            close_line: entry["close_line"],
            ideal_checksum: entry["ideal_checksum"],
            package: entry["package"],
            export: entry["export"],
            source_checksum: entry["source_checksum"],
            silenced: entry["silenced"]
          )
        end
      )
    end

    def relative_lockfile_path(project_root, path)
      absolute = Pathname(path).absolute? ? Pathname(path) : Pathname(project_root).join(path)
      normalized_root = Pathname(project_root).cleanpath
      normalized_absolute = absolute.cleanpath
      relative = normalized_absolute.relative_path_from(normalized_root)
      format_relative_lockfile_path(relative)
    rescue ArgumentError
      format_relative_lockfile_path(Pathname(path).cleanpath)
    end

    def format_relative_lockfile_path(relative)
      text = relative.to_s.tr("\\", "/")
      text == "." || text.start_with?("./") ? text : "./#{text}"
    end

    def normalize_lockfile_artifact(project_root, artifact, package_root)
      return artifact unless artifact.start_with?("path:")

      path_text = artifact.delete_prefix("path:")
      path = Pathname(path_text)
      relative = if path.absolute?
                   relative_lockfile_path(project_root, path)
                 else
                   relative_lockfile_path(project_root, package_root)
                 end
      "path:#{relative}"
    end
  end

  extend LockfileIO
end
