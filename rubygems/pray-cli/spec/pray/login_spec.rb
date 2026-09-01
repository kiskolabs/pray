# frozen_string_literal: true

require "spec_helper"
require "fileutils"
require "openssl"
require_relative "../support/http_fixture_server"

RSpec.describe Pray::AuthClient do
  let(:workspace) { Dir.mktmpdir("pray-login-") }
  let(:pray_home) { File.join(workspace, "user-home") }

  around do |example|
    original_home = ENV["PRAY_HOME"]
    ENV["PRAY_HOME"] = pray_home
    example.run
  ensure
    original_home ? ENV["PRAY_HOME"] = original_home : ENV.delete("PRAY_HOME")
    FileUtils.rm_rf(workspace)
  end

  it "persists a passkey session after challenge and login" do
    seed = OpenSSL::Random.random_bytes(32)
    key_path = File.join(workspace, "passkey.key")
    File.binwrite(key_path, seed)

    fixture = HttpFixtureServer.start(
      "POST /v1/auth/passkeys/challenge" => [
        "200 OK", "application/json",
        JSON.generate(
          "credential_id" => "cred-1",
          "challenge_id" => "chal-1",
          "challenge" => "login-challenge"
        )
      ],
      "POST /v1/auth/passkeys/login" => [
        "200 OK", "application/json",
        JSON.generate("email" => "dev@example.com", "token" => "tok-1")
      ]
    )

    session = described_class.login_with_passkey(
      fixture[:url], "cred-1", key_path, workspace, email: "dev@example.com"
    )
    expect(session.email).to eq("dev@example.com")
    expect(session.kind).to eq("passkey")
    session_path = File.join(pray_home, "session.json")
    expect(File.read(session_path)).to include("tok-1")
    expect(File.stat(session_path).mode & 0o777).to eq(0o600)
  ensure
    HttpFixtureServer.stop(fixture) if fixture
  end

  it "rejects login when the server email does not match" do
    seed = OpenSSL::Random.random_bytes(32)
    key_path = File.join(workspace, "passkey.key")
    File.binwrite(key_path, seed)
    fixture = HttpFixtureServer.start(
      "POST /v1/auth/passkeys/challenge" => [
        "200 OK", "application/json",
        JSON.generate(
          "credential_id" => "cred-1", "challenge_id" => "chal-1", "challenge" => "x"
        )
      ],
      "POST /v1/auth/passkeys/login" => [
        "200 OK", "application/json",
        JSON.generate("email" => "other@example.com", "token" => "tok-1")
      ]
    )

    expect do
      described_class.login_with_passkey(
        fixture[:url], "cred-1", key_path, workspace, email: "dev@example.com"
      )
    end.to raise_error(Pray::Error, /email mismatch/)
  ensure
    HttpFixtureServer.stop(fixture) if fixture
  end
end
