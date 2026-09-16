# frozen_string_literal: true

module Pray
  class PackageSpec
    LOCAL_VERSION = "local"

    def has_release_version?
      !version.to_s.empty? && version != LOCAL_VERSION
    end

    def recorded_version
      has_release_version? ? version : LOCAL_VERSION
    end

    def require_release_version!
      return if has_release_version?

      raise Error.manifest("package #{name} needs a version before it can be packaged")
    end

    def satisfy_constraint!(constraint)
      unless has_release_version?
        normalized = Constraint.normalize_version_constraint(constraint)
        return if normalized.empty? || normalized == "*"

        raise Error.resolution("package #{name} has no version; add spec.version or omit the constraint")
      end
      return if Constraint.version_satisfies(version, constraint)

      raise Error.resolution("package #{name} version #{version} does not satisfy constraint #{constraint}")
    end
  end
end
