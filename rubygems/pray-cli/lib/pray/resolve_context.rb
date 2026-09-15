# frozen_string_literal: true

module Pray
  ResolveOptions = Struct.new(
    :offline, :refresh, :unlocked_packages, :refresh_source_revisions,
    :ignore_locked_versions, :environment
  ) do
    def initialize(
      offline: false,
      refresh: false,
      unlocked_packages: Set.new,
      refresh_source_revisions: false,
      ignore_locked_versions: false,
      environment: nil
    )
      super
    end

    def preferred_lock_version(lockfile, package_name)
      return nil unless lockfile
      return nil if ignore_locked_versions || unlocked_packages.include?(package_name)

      lockfile.package.find { |entry| entry.name == package_name }&.version
    end
  end
end
