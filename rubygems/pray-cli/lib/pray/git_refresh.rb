# frozen_string_literal: true

module Pray
  module GitRefresh
    module_function

    def resolution_may_benefit_from_git_source_refresh?(error)
      error.category == :resolution && catalog_miss_message?(error.message)
    end

    def catalog_miss_message?(message)
      message.include?("no registry version") ||
        message.include?("not found in distribution") ||
        message.include?("not found in git source") ||
        message.include?("v1/packages/")
    end

    def annotate_missing_git_catalog(error, package_name:, source_name:, revision:)
      return error unless error.category == :resolution
      return error if revision.to_s.empty?
      return error unless catalog_miss_message?(error.message)
      return error if error.message.include?("not found in git source")

      Error.resolution(
        "package #{package_name} was not found in git source #{source_name} at revision #{revision}. " \
        "Run `pray update` to advance the source pin."
      )
    end

    def annotate_failed_refresh(lockfile_path, error)
      return error unless error.category == :resolution
      return error unless catalog_miss_message?(error.message)

      pins = locked_git_revision_guidance(lockfile_path)
      return error if pins.nil?

      package = package_name_from_catalog_miss(error.message) || "the declared package"
      Error.resolution(
        "package #{package} was not found in locked git source (#{pins}). " \
        "Run `pray update` to advance the source pin."
      )
    end

    def locked_git_revision_guidance(lockfile_path)
      return unless File.exist?(lockfile_path)

      lockfile = LockfileIO.read_lockfile(lockfile_path)
      pins = lockfile.source.filter_map do |source|
        next unless source.kind == "git" && source.revision

        "#{source.name} at #{source.revision}"
      end
      pins.empty? ? nil : pins.join(", ")
    rescue Error
      nil
    end

    def package_name_from_catalog_miss(message)
      rest = message.delete_prefix("package ")
      return if rest == message

      ending = rest.index(" was not found") || rest.index(" not found")
      return unless ending

      name = rest[0, ending].strip
      name.empty? ? nil : name
    end
  end
end
