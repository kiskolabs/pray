# frozen_string_literal: true

require "socket"
require "json"

module HttpFixtureServer
  module_function

  def start(routes)
    server = TCPServer.new("127.0.0.1", 0)
    port = server.addr[1]
    thread = Thread.new do
      loop do
        socket = server.accept
        Thread.new { handle(socket, routes) }
      rescue => _error
        break
      end
    end
    thread.abort_on_exception = true
    {server: server, thread: thread, port: port, url: "http://127.0.0.1:#{port}"}
  end

  def stop(fixture)
    fixture[:server].close
    fixture[:thread].kill
  rescue IOError
    nil
  end

  def handle(socket, routes)
    request_line = socket.gets
    return unless request_line

    method, path, = request_line.split
    headers = {}
    loop do
      line = socket.gets
      break if line.nil? || line.strip.empty?

      name, value = line.split(":", 2)
      headers[name.strip.downcase] = value.strip if name && value
    end
    body_length = headers["content-length"].to_i
    body = body_length.positive? ? socket.read(body_length) : ""
    key = "#{method} #{path.split("?", 2).first}"
    response = routes[key]
    response = response.call(body, headers) if response.respond_to?(:call)
    response ||= ["404 Not Found", "text/plain", "not found"]
    status, content_type, payload = response
    payload = payload.to_s
    socket.print(
      "HTTP/1.1 #{status}\r\nContent-Type: #{content_type}\r\n" \
      "Content-Length: #{payload.bytesize}\r\nConnection: close\r\n\r\n#{payload}"
    )
  ensure
    socket.close unless socket.closed?
  end
end
