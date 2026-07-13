# frozen_string_literal: true

module Pray
  class Error < StandardError
    def initialize(category, message, parse_kind: nil)
      @category = category
      @parse_kind = parse_kind
      super(message)
    end

    attr_reader :category

    def exit_code
      case category
      when :parse then 2
      when :manifest, :io then 1
      when :resolution then 3
      when :integrity then 4
      when :render then 5
      when :verify then 6
      when :unsupported then 8
      else 1
      end
    end

    def to_s
      prefix = case category
               when :parse then "#{@parse_kind} parse error"
               when :manifest then "manifest error"
               when :resolution then "resolution error"
               when :integrity then "integrity error"
               when :render then "render error"
               when :verify then "verify error"
               when :unsupported then "unsupported feature"
               when :io then "I/O error"
               else category.to_s
               end
      "#{prefix}: #{super}"
    end

    def self.parse(kind, message)
      new(:parse, message, parse_kind: kind)
    end

    def self.manifest(message) = new(:manifest, message)
    def self.resolution(message) = new(:resolution, message)
    def self.integrity(message) = new(:integrity, message)
    def self.render(message) = new(:render, message)
    def self.verify(message) = new(:verify, message)
    def self.unsupported(message) = new(:unsupported, message)
    def self.io(error) = new(:io, error.message)
  end
end
