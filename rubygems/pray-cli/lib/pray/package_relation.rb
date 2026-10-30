# frozen_string_literal: true

module Pray
  module PackageRelation
    module_function

    def canonicalize(references)
      Array(references).map(&:strip).uniq.sort.tap do |canonical|
        invalid = canonical.find { |reference| !valid?(reference) }
        raise Error.parse("prayspec", "invalid related prayer reference") if invalid
      end
    end

    def valid?(reference)
      return false if reference.empty? || reference.match?(/[\\[:cntrl:]]/)
      if reference.match?(/\A[A-Za-z][A-Za-z0-9+.-]*:.+\z/)
        return !uri_has_userinfo?(reference)
      end
      if reference.start_with?("/")
        path = locator_path(reference)
        return path.length > 1 && !path.start_with?("//")
      end
      if reference.start_with?("./", "../")
        return locator_path(reference).split("/").any? do |segment|
          !segment.empty? && ![".", ".."].include?(segment)
        end
      end

      return false if reference.match?(/[?#]/)

      segments = reference.split("/", -1)
      segments.length == 2 && segments.none? { |segment| segment.empty? || [".", ".."].include?(segment) }
    end

    def locator_path(reference)
      reference.split(/[?#]/, 2).first
    end

    def uri_has_userinfo?(reference)
      after_scheme = reference.split("://", 2)[1]
      after_scheme && after_scheme.split(%r{[/?#]}, 2).first.include?("@")
    end
  end
end
