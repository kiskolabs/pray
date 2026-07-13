# frozen_string_literal: true

require "socket"
require "fileutils"
require "pathname"
require_relative "path_safety"

module Pray
  module Serve
    DEFAULT_MAX_CONNECTIONS = 8

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
      handle_connection(root, socket)
    ensure
      connection_slots << true unless connection_slots.closed?
      socket.close unless socket.closed?
    end

    def handle_connection(root, socket)
      request_line = socket.gets
      return unless request_line

      method, path, = request_line.split
      headers = read_headers(socket)
      body_length = headers["content-length"].to_i
      socket.read(body_length) if body_length.positive?

      response = dispatch_request(root, method, path)
      socket.print(response)
    end

    def read_headers(socket)
      headers = {}
      loop do
        line = socket.gets
        break if line.nil? || line.strip.empty?

        name, value = line.split(":", 2)
        headers[name.strip.downcase] = value.strip if name && value
      end
      headers
    end

    def dispatch_request(root, method, path)
      path = path.split("?", 2).first
      return not_found unless method == "GET"

      if path == "/"
        return html_response("<h1>Pray distribution</h1><p>Root: #{root}</p>")
      end

      file_path = PathSafety.join_under_root(root, path.delete_prefix("/"))
      return not_found unless file_path
      return not_found unless File.file?(file_path)

      content_type = content_type_for(file_path)
      body = File.binread(file_path)
      ok_response(content_type, body)
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
  end
end
