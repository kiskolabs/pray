# frozen_string_literal: true

require "socket"
require "base64"

module Pray
  module SshAgent
    module_function

    def sign(public_key, message)
      agent_socket = ENV["SSH_AUTH_SOCK"]
      raise Error.unsupported("SSH_AUTH_SOCK is not set") if agent_socket.to_s.empty?

      raw_key_bytes = parse_ssh_ed25519_public_key(public_key)
      public_key_blob = +""
      write_ssh_string(public_key_blob, "ssh-ed25519")
      write_ssh_string(public_key_blob, raw_key_bytes)

      payload = +""
      write_ssh_string(payload, public_key_blob)
      write_ssh_string(payload, message)
      write_u32(payload, 0)

      UNIXSocket.open(agent_socket) do |stream|
        write_ssh_message(stream, 13, payload)
        message_type, response = read_ssh_message(stream)
        unless message_type == 14
          raise Error.resolution("ssh agent returned unexpected message type: #{message_type}")
        end

        signature_blob = read_ssh_string_bytes(response)
        parse_ssh_signature_blob(signature_blob)
      end
    end

    def parse_ssh_ed25519_public_key(public_key)
      fields = public_key.split(/\s+/)
      algorithm = fields[0]
      raise Error.unsupported("public key must include an algorithm") unless algorithm
      unless algorithm == "ssh-ed25519"
        raise Error.unsupported("unsupported public key algorithm: #{algorithm}")
      end

      key_value = fields[1]
      raise Error.unsupported("public key must include key bytes") unless key_value

      blob = Base64.decode64(key_value)
      cursor = blob.b
      blob_algorithm = read_ssh_string!(cursor)
      unless blob_algorithm == "ssh-ed25519"
        raise Error.parse("public key", "ed25519 public key blob must start with ssh-ed25519")
      end

      key_bytes = read_ssh_string!(cursor)
      unless key_bytes.bytesize == 32
        raise Error.parse("public key", "ed25519 public key must be 32 bytes")
      end

      key_bytes
    end

    def parse_ssh_signature_blob(signature_blob)
      cursor = signature_blob.b
      algorithm = read_ssh_string!(cursor)
      unless algorithm == "ssh-ed25519"
        raise Error.unsupported("unsupported ssh signature algorithm: #{algorithm}")
      end

      Base64.strict_encode64(read_ssh_string!(cursor))
    end

    def write_ssh_message(stream, message_type, payload)
      buffer = message_type.chr + payload
      stream.write([buffer.bytesize].pack("N"))
      stream.write(buffer)
    end

    def read_ssh_message(stream)
      length = stream.read(4)&.unpack1("N")
      raise Error.resolution("empty ssh agent response") unless length

      buffer = stream.read(length)
      raise Error.resolution("truncated ssh agent response") if buffer.nil? || buffer.bytesize < 1

      [buffer.getbyte(0), buffer.byteslice(1, buffer.bytesize - 1).to_s]
    end

    def write_ssh_string(buffer, bytes)
      bytes = bytes.b
      buffer << [bytes.bytesize].pack("N")
      buffer << bytes
    end

    def write_u32(buffer, value)
      buffer << [value].pack("N")
    end

    def read_ssh_string!(cursor)
      raise Error.resolution("truncated ssh agent response") if cursor.bytesize < 4

      length = cursor.byteslice(0, 4).unpack1("N")
      cursor.slice!(0, 4)
      raise Error.resolution("truncated ssh agent response") if cursor.bytesize < length

      value = cursor.byteslice(0, length)
      cursor.slice!(0, length)
      value
    end

    def read_ssh_string_bytes(buffer)
      cursor = buffer.b
      read_ssh_string!(cursor)
    end
  end
end
