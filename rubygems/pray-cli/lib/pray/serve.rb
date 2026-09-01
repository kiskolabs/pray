# frozen_string_literal: true

require "socket"
require "fileutils"
require "pathname"
require "json"
require "timeout"
require_relative "path_safety"
require_relative "resource_limits"
require_relative "serve_federation"

module Pray
  module Serve
    DEFAULT_MAX_CONNECTIONS = ResourceLimits::MAX_SERVE_CONCURRENT_CONNECTIONS

    module_function

    def run_server(root:, host: "127.0.0.1", port: 7429, max_connections: DEFAULT_MAX_CONNECTIONS)
      root = File.expand_path(root)
      server = TCPServer.new(host, port)
      connection_slots = SizedQueue.new(max_connections)
      max_connections.times { connection_slots << true }
      puts "Serving #{root} on http://#{host}:#{port}"

      loop do
        socket = server.accept
        begin
          connection_slots.pop(true)
        rescue ThreadError
          socket.print(service_unavailable)
          socket.close
          next
        end

        Thread.new { serve_connection(root, socket, connection_slots) }
      rescue Interrupt
        break
      end
    ensure
      server&.close
    end

    def serve_connection(root, socket, connection_slots)
      Timeout.timeout(ResourceLimits::SERVE_SOCKET_TIMEOUT_SECONDS) do
        handle_connection(root, socket)
      end
    rescue Timeout::Error
      socket.print(request_timeout) unless socket.closed?
    rescue Error => error
      response = error.message.include?("request body exceeds") ? payload_too_large : bad_request
      socket.print(response) unless socket.closed?
    ensure
      connection_slots << true unless connection_slots.closed?
      socket.close unless socket.closed?
    end

    def handle_connection(root, socket)
      request_line = socket.gets
      return unless request_line

      method, path, = request_line.split
      headers = read_headers(socket)
      body_length = Integer(headers.fetch("content-length", "0"), exception: false)
      raise Error.parse("request", "invalid content length") unless body_length
      validate_body_length!(body_length)
      body = body_length.positive? ? socket.read(body_length) : ""

      response = dispatch_request(root, method, path, body)
      socket.print(response)
    end

    def read_headers(socket)
      headers = {}
      header_bytes = 0
      loop do
        line = socket.gets
        break if line.nil? || line.strip.empty?
        header_bytes += line.bytesize
        if header_bytes > ResourceLimits::MAX_SERVE_HEADER_BYTES
          raise Error.parse("request", "request headers exceed server limit")
        end

        name, value = line.split(":", 2)
        headers[name.strip.downcase] = value.strip if name && value
      end
      headers
    end

    def validate_body_length!(body_length)
      return if body_length <= ResourceLimits::MAX_SERVE_BODY_BYTES

      raise Error.unsupported(
        "request body exceeds #{ResourceLimits::MAX_SERVE_BODY_BYTES} bytes"
      )
    end

    def dispatch_request(root, method, path, body = "")
      path = path.split("?", 2).first

      case [method, path]
      when ["GET", "/health"]
        return ok_response("text/plain", "ok")
      when ["GET", "/.well-known/pray-federation.json"]
        return ServeFederation.discovery_response(root)
      when ["GET", "/v1/sync/index"]
        return ServeFederation.index_response(root)
      when ["POST", "/v1/confessions"]
        return ServeFederation.append_confession(root, body)
      end

      if method == "GET" && path.start_with?("/v1/sync/package/")
        return ServeFederation.package_response(root, path)
      end

      return not_found unless method == "GET"

      if path == "/"
        return html_response("<h1>Pray distribution</h1>")
      end

      file_path = PathSafety.join_under_root(root, path.delete_prefix("/"))
      return not_found unless file_path
      return not_found unless File.file?(file_path)
      return payload_too_large if File.size(file_path) > ResourceLimits::MAX_HTTP_RESPONSE_BYTES

      content_type = content_type_for(file_path)
      file_body = File.binread(file_path)
      ok_response(content_type, file_body)
    end

    def content_type_for(path)
      case File.extname(path)
      when ".json" then "application/json"
      when ".praypkg" then "application/octet-stream"
      else "text/plain"
      end
    end

    def ok_response(content_type, body)
      "HTTP/1.1 200 OK\r\nContent-Type: #{content_type}\r\nContent-Length: #{body.bytesize}\r\nConnection: close\r\n\r\n#{body}"
    end

    def html_response(body)
      ok_response("text/html", body)
    end

    def not_found
      body = "not found"
      "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\nContent-Length: #{body.bytesize}\r\nConnection: close\r\n\r\n#{body}"
    end

    def service_unavailable
      body = "too many connections"
      "HTTP/1.1 503 Service Unavailable\r\nContent-Type: text/plain\r\nContent-Length: #{body.bytesize}\r\nConnection: close\r\n\r\n#{body}"
    end

    def request_timeout
      body = "request timed out"
      "HTTP/1.1 408 Request Timeout\r\nContent-Type: text/plain\r\nContent-Length: #{body.bytesize}\r\nConnection: close\r\n\r\n#{body}"
    end

    def payload_too_large
      body = "request exceeds server limit"
      "HTTP/1.1 413 Payload Too Large\r\nContent-Type: text/plain\r\nContent-Length: #{body.bytesize}\r\nConnection: close\r\n\r\n#{body}"
    end

    def bad_request
      body = "bad request"
      "HTTP/1.1 400 Bad Request\r\nContent-Type: text/plain\r\nContent-Length: #{body.bytesize}\r\nConnection: close\r\n\r\n#{body}"
    end
  end
end
