# frozen_string_literal: true

module Pray
  module CLI
    def login_command(servers:, email:, mode:, passkey_key: nil, credential_id: nil, public_key: nil)
      session_root = Dir.pwd
      servers.each do |server_url|
        session = case mode
        when :passkey
          AuthClient.login_with_passkey(
            server_url, credential_id, passkey_key, session_root, email: email
          )
        when :ssh_agent
          AuthClient.login_with_ssh_agent(
            server_url, public_key, session_root, email: email
          )
        else
          raise Error.unsupported("unknown login mode: #{mode}")
        end
        puts "logged in as #{session.email} via #{session.kind} on #{server_url}"
      end
    end

    def confess_command(package:, from_lock:, version:, accepted:, rejected:, note:, url:)
      Confess.submit(
        package: package, from_lock: from_lock, version: version,
        accepted: accepted, rejected: rejected, note: note, url: url
      )
    end

    def sync_command(root:, peers:)
      peer_sources = if peers.empty?
        Sync.load_sync_peers(root).map { |peer| peer["url"] }
      else
        peers
      end
      summary = Sync.synchronize_registry(root, peer_sources)
      puts "Synchronized #{summary[:packages]} package(s) from #{summary[:peers]} peer(s); " \
           "learned #{summary[:known_peers]} peer(s)"
    end
  end
end
