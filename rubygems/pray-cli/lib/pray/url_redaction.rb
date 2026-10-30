# frozen_string_literal: true

require "uri"

module Pray
  module UrlRedaction
    module_function

    def redact_userinfo(url)
      uri = URI(url)
      return url unless uri.user || uri.password

      uri.user = "***"
      uri.password = nil
      uri.to_s
    rescue URI::InvalidURIError
      url
    end
  end
end
