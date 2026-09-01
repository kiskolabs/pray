# frozen_string_literal: true

module Pray
  module RegistryIntegrity
    def validate_and_unpack(cache_directory, declaration, selected, artifact_bytes, source_url: nil)
      require_integrity_fields!(declaration.name, selected)
      artifact_hash = Hashing.sha256_prefixed(artifact_bytes)
      if artifact_hash != selected.artifact_hash
        raise Error.integrity(
          "package artifact hash mismatch for #{declaration.name} #{selected.version}"
        )
      end

      verify_registry_signature!(declaration, selected, artifact_bytes)
      Trust.verify_publisher_fingerprint!(source_url, selected) if source_url
      Archive.unpack_praypkg(artifact_bytes, cache_directory)
      spec_path = Resolve.find_prayspec_file(cache_directory)
      spec = Pray.parse_package_spec(File.read(spec_path)).canonicalized
      validate_package_identity!(cache_directory, declaration, selected, spec)
      actual_tree_hash = spec.tree_hash_for_root(cache_directory)
      if actual_tree_hash != selected.tree_hash
        raise Error.integrity(
          "package tree hash mismatch for #{declaration.name} #{selected.version}"
        )
      end
    end

    def require_integrity_fields!(package_name, selected)
      if selected.artifact_hash.to_s.empty?
        raise Error.integrity("package #{package_name} #{selected.version} is missing artifact_hash")
      end
      if selected.tree_hash.to_s.empty?
        raise Error.integrity("package #{package_name} #{selected.version} is missing tree_hash")
      end
    end

    def cache_ready?(cache_directory, selected)
      return false unless File.directory?(cache_directory)

      spec_path = Resolve.find_prayspec_file(cache_directory)
      spec = Pray.parse_package_spec(File.read(spec_path)).canonicalized
      spec.version == selected.version && !selected.tree_hash.to_s.empty? &&
        spec.tree_hash_for_root(cache_directory) == selected.tree_hash
    rescue Error, SystemCallError
      false
    end

    private

    def validate_package_identity!(cache_directory, declaration, selected, spec)
      if spec.name != declaration.name
        raise Error.resolution(
          "package path #{cache_directory.inspect} declares #{spec.name.inspect}, expected #{declaration.name.inspect}"
        )
      end
      return if spec.version == selected.version

      raise Error.resolution(
        "package #{declaration.name} version #{spec.version} does not match registry version #{selected.version}"
      )
    end
  end
end
