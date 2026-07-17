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

    it "rejects unknown commands with usage errors" do
      expect { described_class.parse_command(["not-a-command"]) }
        .to raise_error(Pray::Error, /usage error:.*unknown command/)
    end

    it "parses trust subcommands" do
      expect(described_class.parse_command(["trust", "list"])).to eq([:trust_list])
      expect(described_class.parse_command(["trust", "show", "https://registry.example"])).to eq(
        [:trust_show, "https://registry.example"]
      )
    end
  end
end
