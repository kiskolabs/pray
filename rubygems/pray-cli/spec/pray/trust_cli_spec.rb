# frozen_string_literal: true

require "spec_helper"
require "fileutils"

RSpec.describe "trust CLI" do
  let(:workspace) { Dir.mktmpdir("pray-trust-cli-") }

  after { FileUtils.rm_rf(workspace) }

  around do |example|
    original = ENV["PRAY_HOME"]
    ENV["PRAY_HOME"] = workspace
    example.run
  ensure
    original ? ENV["PRAY_HOME"] = original : ENV.delete("PRAY_HOME")
  end

  it "adds and lists signing keys in policy" do
    Pray::CLI.trust_add_key_command("sha256:abc", match_prefix: "https://example.com")
    output = Pray::Trust.list_policy(scope: :all)
    expect(output).to include("https://example.com")
    expect(output).to include("SHA256:ABC")
    expect(File.read(File.join(workspace, "trust.toml"))).to include("require_signed_commit")
  end

  it "imports publishers from a local registry root" do
    root = File.join(workspace, "dist")
    FileUtils.mkdir_p(File.join(root, "v1"))
    File.write(
      File.join(root, "v1", "ssh_publishers.json"),
      JSON.generate({"publishers" => [{"fingerprint" => "SHA256:pub1"}]})
    )
    result = Pray::Trust.import_registry(root)
    expect(result[:publishers_added]).to eq(1)
    expect(Pray::Trust.load_policy_or_default.rules.first.allowed_publishers).to include("SHA256:PUB1")
  end

  it "detects compromised trusted keys from a local feed" do
    Pray::Trust.add_allowed_signing_key("sha256:dead")
    feed = File.join(workspace, "compromised.txt")
    File.write(feed, "sha256:dead # leaked\n")
    source, hits = Pray::Trust.check_compromised(feed)
    expect(source).to end_with("compromised.txt")
    expect(hits.length).to eq(1)
    expect(hits.first[0]).to eq("SHA256:DEAD")
  end

  it "mutates allow signed commit and removes keys" do
    Pray::Trust.set_allow("https://example.com", false)
    Pray::Trust.set_require_signed_commit("https://example.com", true)
    Pray::Trust.add_allowed_signing_key("sha256:one", match_prefix: "https://example.com")
    Pray::Trust.remove_allowed_signing_key("sha256:one", match_prefix: "https://example.com")
    rule = Pray::Trust.load_policy_or_default.rules.first
    expect(rule.allow).to be(false)
    expect(rule.require_signed_commit).to be(true)
    expect(rule.allowed_signing_keys).to be_empty
    expect(Pray::Trust.show_policy_toml).to include("match_prefix")
    listed = Pray::Trust.list_policy(scope: :all, source_url: "https://example.com/x")
    expect(listed).to include("scope: local")
    expect(listed).to include("effective_scope")
    expect(Pray::Trust.list_policy(scope: :global)).to include("scope: global")
    expect(Pray::Trust.list_policy(scope: :local)).to include("https://example.com")
    empty_home = File.join(workspace, "empty-home")
    FileUtils.mkdir_p(empty_home)
    expect(Pray::Trust.list_policy(scope: :all, home: empty_home)).to include("(no rules)")
    feed = File.join(workspace, "clean.toml")
    File.write(feed, "[[keys]]\nvalue = \"sha256:other\"\n")
    source, hits = Pray::Trust.check_compromised(feed, home: empty_home)
    expect(hits).to be_empty
    expect(source).to end_with("clean.toml")
  end
end
