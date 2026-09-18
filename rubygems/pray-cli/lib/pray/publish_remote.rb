# frozen_string_literal: true

module Pray
  ManifestPublishRemote = Struct.new(:name, :path, :url, :packages) do
    def initialize(name:, path: nil, url: nil, packages: [])
      super
    end
  end

  module PublishRemote
    module_function

    def validate_publish_remotes!(manifest)
      seen = {}
      (manifest.publish_remotes || []).each do |remote|
        raise Error.parse("manifest", "publish requires a name") if remote.name.to_s.strip.empty?
        if seen[remote.name]
          raise Error.manifest("duplicate publish remote: #{remote.name}")
        end
        seen[remote.name] = true
        if remote.path && remote.url
          raise Error.parse("manifest", "publish \"#{remote.name}\" must set path: or a URL, not both")
        end
        if remote.path.nil? && remote.url.nil?
          raise Error.parse("manifest", "publish \"#{remote.name}\" requires path: or a URL")
        end
        PathSafety.validate_project_relative_path!(remote.path) if remote.path
        if remote.url && !publish_url?(remote.url)
          raise Error.parse(
            "manifest",
            "publish \"#{remote.name}\" URL must be https://, http://, pray+ssh://, or ssh+pray://"
          )
        end
        validate_remote_packages!(manifest, remote)
      end
    end

    def validate_remote_packages!(manifest, remote)
      remote.packages.each do |package_name|
        package = manifest.packages.find { |entry| entry.name == package_name }
        unless package
          raise Error.manifest("publish \"#{remote.name}\" lists unknown package #{package_name}")
        end
        next if package_is_path_owned?(package, manifest.sources)

        raise Error.manifest(
          "publish \"#{remote.name}\" lists #{package_name}, which is not a path package"
        )
      end
    end

    def package_is_path_owned?(package, sources)
      return true if package.path
      return false if package.git || package.tarball || package.oci

      map = sources.to_h { |source| [source.name, source] }
      name = ResolveSource.implied_source_name(package, map)
      name && map[name]&.kind == "path"
    rescue Error
      false
    end

    def path_owned_package_names(manifest)
      manifest.packages.select { |package| package_is_path_owned?(package, manifest.sources) }
        .map(&:name)
    end

    def allowed_publish_names(manifest, listed)
      listed.empty? ? path_owned_package_names(manifest) : listed
    end

    def publish_url?(url)
      url.start_with?("https://", "http://", "pray+ssh://", "ssh+pray://")
    end
  end
end
