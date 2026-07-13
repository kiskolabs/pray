# frozen_string_literal: true

module Pray
  module Constraint
    module_function

    def normalize_version_constraint(constraint)
      trimmed = constraint.strip
      return trimmed if trimmed.empty? || trimmed == "*"
      return trimmed if operator_prefixed?(trimmed)

      if bare_semver?(trimmed)
        "=#{trimmed}"
      else
        trimmed
      end
    end

    def version_satisfies(version, constraint)
      normalized = normalize_version_constraint(constraint)
      return true if normalized.empty? || normalized == "*"

      requirement = Gem::Requirement.new(normalized.strip)
      requirement.satisfied_by?(Gem::Version.new(version))
    rescue ArgumentError => error
      raise Error.resolution(error.message)
    end

    def pessimistic_constraint_for_version(version)
      parsed = Gem::Version.new(version)
      if parsed.segments[1].to_i.zero? && parsed.segments[2].to_i.zero?
        "~> #{parsed.segments[0]}.0"
      else
        "~> #{parsed.segments[0]}.#{parsed.segments[1]}"
      end
    end

    def latest_constraint_for_package(current_constraint, latest_version)
      normalized = normalize_version_constraint(current_constraint)
      return "*" if normalized == "*"
      return pessimistic_constraint_for_version(latest_version) if normalized.start_with?("~")

      if normalized.start_with?("^")
        parsed = Gem::Version.new(latest_version)
        return "^#{parsed.segments[0]}.#{parsed.segments[1]}"
      end

      if normalized.start_with?("=") || bare_semver?(current_constraint.strip)
        return "=#{latest_version}"
      end

      pessimistic_constraint_for_version(latest_version)
    end

    def ruby_pessimistic_to_semver(constraint)
      text = constraint.strip.sub(/\A~>\s*/, "")
      parts = text.split(".")
      raise Error.resolution("unsupported Ruby pessimistic constraint: #{constraint}") if parts.empty? || parts.length > 3

      numbers = [parts[0].to_i, (parts[1] || 0).to_i, (parts[2] || 0).to_i]
      lower = numbers.join(".")
      upper = case parts.length
              when 1 then "#{numbers[0] + 1}.0.0"
              when 2 then "#{numbers[0]}.#{numbers[1] + 1}.0"
              else "#{numbers[0]}.#{numbers[1] + 1}.0"
              end
      ">= #{lower}, < #{upper}"
    end

    def operator_prefixed?(text)
      text.start_with?("~>", "~", "^", "=", ">", "<") || text.include?("*")
    end

    def bare_semver?(text)
      Gem::Version.correct?(text)
    rescue StandardError
      false
    end
  end
end
