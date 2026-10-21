# frozen_string_literal: true

require_relative "path_safety"

module Pray
  class PackageSpec
    def validate_relative_paths!
      files.each { |file| PathSafety.validate_archive_member_path!(file) }
      exports.each_value do |export|
        PathSafety.validate_archive_member_path!(export.path)
        PathSafety.validate_archive_member_path!(export.default_path) if export.default_path
      end
      skills.each_value { |skill| PathSafety.validate_archive_member_path!(skill.path) }
      templates.each_value { |template| PathSafety.validate_archive_member_path!(template.path) }
      self
    end
  end
end
