# frozen_string_literal: true

require "json"
require "base64"
require "openssl"
require_relative "session"
require_relative "ssh_agent"

module Pray
  module AuthClient
    module_function

    def login_with_passkey(server_url, credential_id, private_key_path, session_root, email:)
      challenge = post_json(
        "#{trim_slash(server_url)}/v1/auth/passkeys/challenge",
        {"credential_id" => credential_id}
      )
      seed = File.binread(private_key_path)
      unless seed.bytesize == 32
        raise Error.unsupported("passkey private key must be 32 raw bytes")
      end

      signature = Base64.strict_encode64(sign_ed25519(seed, challenge["challenge"].to_s))
      response = post_json(
        "#{trim_slash(server_url)}/v1/auth/passkeys/login",
        {
          "credential_id" => credential_id,
          "challenge_id" => challenge["challenge_id"],
          "signature" => signature
        }
      )
      persist_login(server_url, response, email, "passkey", nil, session_root)
    end

    def login_with_ssh_agent(server_url, public_key_path, session_root, email:)
      public_key = File.read(public_key_path).strip
      challenge = post_json(
        "#{trim_slash(server_url)}/v1/auth/ssh-keys/challenge",
        {"public_key" => public_key}
      )
      signature = SshAgent.sign(public_key, challenge["challenge"].to_s)
      response = post_json(
        "#{trim_slash(server_url)}/v1/auth/ssh-keys/login",
        {
          "public_key" => public_key,
          "challenge_id" => challenge["challenge_id"],
          "signature" => signature
        }
      )
      persist_login(
        server_url, response, email, "ssh_key", challenge["fingerprint"], session_root
      )
    end

    def persist_login(server_url, response, email, kind, fingerprint, session_root)
      response_email = response["email"].to_s
      unless response_email == email
        raise Error.resolution(
          "login email mismatch: expected #{email}, got #{response_email}"
        )
      end

      Session.persist(
        session_root,
        SessionFile.new(
          server_url: server_url,
          email: response_email,
          token: response["token"].to_s,
          kind: kind,
          signer_fingerprint: fingerprint
        )
      )
    end

    def sign_ed25519(seed, message)
      unless OpenSSL::PKey.respond_to?(:generate_key)
        raise Error.unsupported("OpenSSL Ed25519 support is required for passkey login")
      end

      key = OpenSSL::PKey.new_raw_private_key("ED25519", seed)
      key.sign(nil, message)
    rescue OpenSSL::PKey::PKeyError => error
      raise Error.unsupported("OpenSSL Ed25519 signing failed: #{error.message}")
    end

    def post_json(url, body)
      response = Registry.http_post(url, "application/json", JSON.generate(body))
      JSON.parse(response)
    rescue JSON::ParserError => error
      raise Error.parse("auth response", error.message)
    end

    def trim_slash(value)
      value.to_s.sub(%r{/+\z}, "")
    end
  end
end
