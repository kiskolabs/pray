# frozen_string_literal: true

require "json"
require "fileutils"
require_relative "trust"

module Pray
  SessionFile = Struct.new(
    :server_url, :email, :token, :kind, :signer_fingerprint,
    keyword_init: true
  )

  module Session
    module_function

    def session_file_path(_root)
      File.join(Trust.trust_home, "session.json")
    end

    def persist(root, session)
      path = session_file_path(root)
      FileUtils.mkdir_p(File.dirname(path))
      migrate_legacy_session(root, path)
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
      write_document(path, document)
      session
    end

    def load_latest(root)
      path = session_file_path(root)
      migrate_legacy_session(root, path)
      sessions = load_sessions(path)
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

    def migrate_legacy_session(root, path)
      legacy_path = File.join(root, ".pray", "session.json")
      return unless File.file?(legacy_path)
      return if File.expand_path(legacy_path) == File.expand_path(path)

      sessions = load_sessions(path)
      load_sessions(legacy_path).each do |legacy|
        sessions << legacy unless sessions.any? { |entry| entry.server_url == legacy.server_url }
      end
      document = (sessions.length == 1) ? session_to_hash(sessions.first) : {
        "sessions" => sessions.map { |entry| session_to_hash(entry) }
      }
      FileUtils.mkdir_p(File.dirname(path))
      write_document(path, document)
      File.delete(legacy_path)
    end

    def write_document(path, document)
      temporary_path = "#{path}.tmp-#{Process.pid}"
      File.open(temporary_path, File::WRONLY | File::CREAT | File::TRUNC, 0o600) do |file|
        file.write("#{JSON.pretty_generate(document)}\n")
        file.flush
        file.fsync
      end
      File.chmod(0o600, temporary_path)
      File.rename(temporary_path, path)
    end
  end
end
