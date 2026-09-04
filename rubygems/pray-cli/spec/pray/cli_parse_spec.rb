# frozen_string_literal: true

require "spec_helper"

RSpec.describe Pray::CLI do
  describe ".parse_command" do
    it "requires a command when none is given" do
      expect { described_class.parse_command([]) }
        .to raise_error(Pray::Error, /pray requires a command/)
    end

    it "parses install flags" do
      expect(described_class.parse_command(["install", "--locked", "--offline"])).to eq(
        [:install, {check: false, strict: false, semantic: false, locked: true, frozen: false, offline: true, targets: []}]
      )
    end

    it "parses add with path" do
      expect(described_class.parse_command(["add", "demo/pkg", "~> 1.0", "--path", "packages/demo"])).to eq(
        [:add, {name: "demo/pkg", constraint: "~> 1.0", path: "packages/demo"}]
      )
    end

    it "parses publish destinations" do
      expect(described_class.parse_command(["publish", "--root", "dist", "--server", "https://registry.example"])).to eq(
        [:publish, {roots: ["dist"], servers: ["https://registry.example"]}]
      )
    end

    it "strictly parses clean arguments" do
      expect(described_class.parse_command(["clean"])).to eq([:clean, {unused: false}])
      expect(described_class.parse_command(["clean", "--unused"])).to eq([:clean, {unused: true}])
      expect { described_class.parse_command(["clean", "--other"]) }
        .to raise_error(Pray::Error, /unknown clean flag/)
      expect { described_class.parse_command(["clean", "unused"]) }
        .to raise_error(Pray::Error, /unexpected clean argument/)
    end

    it "rejects unknown commands with usage errors" do
      expect { described_class.parse_command(["not-a-command"]) }
        .to raise_error(Pray::Error, /usage error:.*unknown command/)
    end

    it "parses trust subcommands" do
      expect(described_class.parse_command(["trust", "list"])).to eq(
        [:trust_list, {scope: :all, source_url: nil}]
      )
      expect(described_class.parse_command(["trust", "show"])).to eq([:trust_show])
      expect(described_class.parse_command(["trust", "add-key", "sha256:abc", "--match-prefix", "https://x"])).to eq(
        [:trust_add_key, {key: "sha256:abc", match_prefix: "https://x"}]
      )
    end

    it "parses login confess and sync" do
      expect(described_class.parse_command([
        "login", "--server", "http://127.0.0.1:9", "--email", "a@b.c",
        "--passkey-key", "key", "--credential-id", "cred"
      ])).to eq([
        :login,
        {servers: ["http://127.0.0.1:9"], email: "a@b.c", mode: :passkey,
         passkey_key: "key", credential_id: "cred"}
      ])
      expect(described_class.parse_command(["confess", "sample/base", "--accepted"])).to eq([
        :confess,
        {package: "sample/base", from_lock: nil, version: nil, accepted: true,
         rejected: false, note: nil, url: nil}
      ])
      expect(described_class.parse_command(["sync", "--root", "dist", "--peer", "http://x"])).to eq(
        [:sync, {root: "dist", peers: ["http://x"]}]
      )
    end
  end
end
