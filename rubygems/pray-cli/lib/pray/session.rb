# frozen_string_literal: true

require "json"
require "fileutils"

module Pray
  SessionFile = Struct.new(
    :server_url, :email, :token, :kind, :signer_fingerprint,
    keyword_init: true
  )

  module Session
    module_function

    def session_file_path(root)
      File.join(root, ".pray", "session.json")
    end

    def persist(root, session)
      path = session_file_path(root)
      FileUtils.mkdir_p(File.dirname(path))
      sessions = load_sessions(path)
      existing = sessions.find { |entry| entry.server_url == session.server_url }
      if existing
        existing.email = session.email
        existing.token = session.token
        existing.kind = session.kind
        existing.signer_fingerprint = session.signer_fingerprint
      else
        sessions << session
      end
      document = if sessions.length == 1
        session_to_hash(sessions.first)
      else
        {"sessions" => sessions.map { |entry| session_to_hash(entry) }}
      end
      File.write(path, JSON.pretty_generate(document))
      session
    end

    def load_latest(root)
      sessions = load_sessions(session_file_path(root))
      sessions.reverse.find { |session| !session.email.to_s.strip.empty? }
    end

    def current_signer(root)
      session = load_latest(root)
      return session.email if session && !session.email.to_s.strip.empty?

      "local"
    end

    def current_signer_fingerprint(root)
      session = load_latest(root)
      fingerprint = session&.signer_fingerprint.to_s.strip
      fingerprint.empty? ? nil : fingerprint
    end

    def load_sessions(path)
      return [] unless File.file?(path)

      data = JSON.parse(File.read(path))
      entries = (data.is_a?(Hash) && data.key?("sessions")) ? data["sessions"] : [data]
      Array(entries).map { |entry| session_from_hash(entry) }
    rescue JSON::ParserError => error
      raise Error.parse("session file", error.message)
    end

    def session_to_hash(session)
      hash = {
        "server_url" => session.server_url,
        "email" => session.email,
        "token" => session.token,
        "kind" => session.kind
      }
      hash["signer_fingerprint"] = session.signer_fingerprint if session.signer_fingerprint
      hash
    end

    def session_from_hash(entry)
      SessionFile.new(
        server_url: entry["server_url"],
        email: entry["email"],
        token: entry["token"],
        kind: entry["kind"],
        signer_fingerprint: entry["signer_fingerprint"]
      )
    end
  end
end
