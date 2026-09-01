# frozen_string_literal: true

require "spec_helper"
require "fileutils"

RSpec.describe Pray::Session do
  let(:workspace) { Dir.mktmpdir("pray-session-") }

  around do |example|
    original_home = ENV["PRAY_HOME"]
    ENV["PRAY_HOME"] = File.join(workspace, "user-home")
    example.run
  ensure
    original_home ? ENV["PRAY_HOME"] = original_home : ENV.delete("PRAY_HOME")
    FileUtils.rm_rf(workspace)
  end

  it "upserts sessions by server url and keeps the latest signer" do
    first = described_class.persist(
      workspace,
      Pray::SessionFile.new(
        server_url: "http://a", email: "a@example.com", token: "t1", kind: "passkey"
      )
    )
    expect(first.email).to eq("a@example.com")

    described_class.persist(
      workspace,
      Pray::SessionFile.new(
        server_url: "http://b", email: "b@example.com", token: "t2", kind: "ssh_key",
        signer_fingerprint: "SHA256:x"
      )
    )
    described_class.persist(
      workspace,
      Pray::SessionFile.new(
        server_url: "http://a", email: "a2@example.com", token: "t3", kind: "passkey"
      )
    )

    document = JSON.parse(File.read(described_class.session_file_path(workspace)))
    expect(document["sessions"].length).to eq(2)
    expect(document["sessions"].find { |entry| entry["server_url"] == "http://a" }["email"]).to eq(
      "a2@example.com"
    )
    expect(described_class.current_signer(workspace)).to eq("b@example.com")
    expect(described_class.current_signer_fingerprint(workspace)).to eq("SHA256:x")
  end
end
